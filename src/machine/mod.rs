pub mod types;

use crate::{
    client::KeygenClient,
    err::{parse_err_json, Error},
    licensed::LicensedState,
    Result,
};
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce, Tag};
use base64::Engine;
use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use tauri::{api::os::locale, webview_version};
use tauri::{AppHandle, Runtime};
use types::{MachineFileRes, MachineLicense};

#[cfg(target_os = "linux")]
static ENGINE_NAME: &str = "WebKit";

#[cfg(target_os = "macos")]
static ENGINE_NAME: &str = "WebKit";

#[cfg(target_os = "windows")]
static ENGINE_NAME: &str = "WebView2";

#[derive(Debug)]
pub struct Machine {
    pub fingerprint: String,
    pub name: String,
    pub hostname: String,
    pub platform: String,
    pub user_agent: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct MachineFile {
    enc: String,
    sig: String,
    alg: String,
}

impl Machine {
    pub(crate) fn new(app_name: String, app_version: String) -> Self {
        let fingerprint = machine_uid::get().unwrap_or("".into());
        let name = whoami::devicename();
        let hostname = whoami::fallible::hostname().unwrap_or("".into());

        // platform
        let os_name = format!("{}", whoami::platform());
        let os_version = whoami::distro().to_string();
        let arch = format!("{}", whoami::arch());
        let platform = format!("{} - {} - {}", os_name, os_version, arch);

        // user agent
        let engine_name = ENGINE_NAME.to_string();
        let engine_version = webview_version().unwrap_or_default();
        let locale = locale().unwrap_or_default();
        let user_agent = format!(
            "{}/{} {}/{} {}/{} {}",
            app_name, app_version, os_name, os_version, engine_name, engine_version, locale
        );

        Self {
            fingerprint,
            name,
            hostname,
            platform,
            user_agent,
        }
    }

    pub(crate) async fn activate(
        &self,
        licensed_state: &mut LicensedState,
        client: &KeygenClient,
    ) -> Result<()> {
        // get license
        let license = licensed_state
            .get_license()
            .ok_or_else(|| Error::LicenseErr {
                code: "NO_LICENSE".into(),
                detail: "Can't activate a machine. Current app state has no license. Call validate(key) first."
                    .into(),
            })?;

        // make sure fingerprint is not an empty string
        if self.fingerprint.is_empty() {
            return Err(Error::LicenseErr {
                code: "NO_FINGERPRINT".into(),
                detail: "Can't activate this machine. Failed parsing machine fingerprint".into(),
            });
        }

        // prepare request
        let url = client.build_url("machines".into(), None)?;
        let body = serde_json::json!({
            "data": {
                "type": "machines",
                "attributes": {
                    "fingerprint": self.fingerprint,
                    "name": self.name,
                    "platform": self.platform
                },
                "relationships": {
                    "license": {
                        "data": {
                            "type": "licenses",
                            "id": license.id
                        }
                    }
                }
            }
        });

        // request machine activation
        let response = client
            .post(url.to_string())
            .timeout(Duration::from_secs(90))
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .header("Authorization", format!("License {}", license.key))
            .json(&body)
            .send()
            .await?;

        // extract response
        let res_status = response.status();
        let res_headers = response.headers().clone();
        let (res_text, res_json) = client.res_text_json(response).await?;

        match res_status {
            StatusCode::CREATED => {
                // verify signature
                match client.verify_response(Method::POST.to_string(), url, res_headers, res_text) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            status_code => Err(parse_err_json(status_code, res_json)),
        }
    }

    pub(crate) async fn checkout<R: Runtime>(
        &self,
        licensed_state: &LicensedState,
        client: &KeygenClient,
        app: &AppHandle<R>,
        ttl: u32,
    ) -> Result<()> {
        // get license
        let license = licensed_state
            .get_license()
            .ok_or_else(|| Error::LicenseErr {
                code: "NO_LICENSE".into(),
                detail: "Can't checkout machine file. Current app state has no license. Call validateKey() first."
                .into()
            })?;

        // must have valid license
        if !license.valid {
            return Err(Error::LicenseErr {
                code: "INVALID_LICENSE".into(),
                detail: "Can't checkout machine file. Current license is invalid".into(),
            });
        }

        // expiry can't be null
        if license.expiry.is_none() {
            return Err(Error::LicenseErr {
                code: "NOT_ACTIVATED".into(),
                detail: "Can't checkout machine file. License hasn't been fully activated. Expiry still Null".into(),
            });
        }

        // build url
        let mut params = vec![
            ("encrypt", "1"),
            ("include", "license.entitlements,license"),
        ];

        // ttl should be min 1 hour max 1 year
        let ttl = std::cmp::min(ttl, 3600);
        let ttl = std::cmp::max(ttl, 31556952);

        // make sure ttl doesn't exceed license expiry
        let expiry_date = DateTime::parse_from_rfc3339(&license.expiry.clone().unwrap())
            .map_err(|_| Error::ParseErr("Failed parsing license expiry date".into()))?;
        let seconds_to_expiry = expiry_date.signed_duration_since(Utc::now()).num_seconds();
        let ttl: i64 = std::cmp::min(seconds_to_expiry, ttl.into());
        let ttl = ttl.to_string();

        // set ttl params
        if license.should_maintain_access() {
            params.push(("ttl", "")); // checkout forever for perpetual fallback license
        } else {
            params.push(("ttl", &ttl))
        }

        let url = client.build_url(
            format!("machines/{}/actions/check-out", self.fingerprint),
            Some(params),
        )?;

        // request machine checkout
        let response = client
            .post(url.to_string())
            .timeout(Duration::from_secs(90))
            .header("Accept", "application/vnd.api+json")
            .header("Authorization", format!("License {}", license.key))
            .send()
            .await?;

        // extract response
        let res_status = response.status();
        let res_headers = response.headers().clone();
        let (res_text, res_json) = client.res_text_json(response).await?;

        match res_status {
            StatusCode::OK => {
                // verify signature
                match client.verify_response(Method::POST.to_string(), url, res_headers, res_text) {
                    Ok(_) => {
                        // map res json
                        let machine_file_res: MachineFileRes = serde_json::from_value(res_json)
                            .map_err(|_| {
                                Error::ParseErr("Failed deserializing machine file response".into())
                            })?;

                        // get certificate
                        let cert = machine_file_res.data.attributes.certificate;

                        // save to '[APP_DATA]/keygen/machine.lic'
                        self.save_machine_file(cert, app)?;

                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            code => Err(parse_err_json(code, res_json)),
        }
    }

    pub(crate) fn load_machine_file<R: Runtime>(
        &self,
        license_key: &String,
        client: &KeygenClient,
        app: &AppHandle<R>,
    ) -> Result<Option<MachineLicense>> {
        // machine file path
        let path = Self::get_machine_file_path(app)?;

        // no machine file
        if !path.exists() {
            return Ok(None);
        }

        // load cert
        let cert = fs::read_to_string(path)?;

        // decrypt cert
        let machine_license = self.decrypt_machine_file(cert, license_key, client)?;

        Ok(Some(machine_license))
    }

    fn decrypt_machine_file(
        &self,
        cert: String,
        license_key: &String,
        client: &KeygenClient,
    ) -> Result<MachineLicense> {
        // Extract the encoded payload from the machine file.
        let enc = cert
            .replace("-----BEGIN MACHINE FILE-----", "")
            .replace("-----END MACHINE FILE-----", "")
            .replace('\n', "");

        // Decode the payload.
        let payload = base64::engine::general_purpose::STANDARD
            .decode(enc)
            .map_err(|_| Error::ParseErr("Failed decoding machine file".into()))?;

        let payload = String::from_utf8(payload)
            .map_err(|_| Error::ParseErr("Failed parsing decoded machine file".into()))?;

        // Parse the payload.
        let lic: MachineFile = serde_json::from_str(payload.as_str())
            .map_err(|_| Error::ParseErr("failed deserializing machine file".into()))?;

        // Assert algorithm is supported.
        if !lic.alg.eq("aes-256-gcm+ed25519") {
            return Err(Error::ParseErr("algorithm is not supported".into()));
        }

        // Verify the machine file's signature.
        let msg = format!("machine/{}", lic.enc);
        client.verify_signature(msg, lic.sig.to_string())?;

        // hash the license key and machine id to obtain decryption key
        let mut sha = Sha256::new();
        let secret = [license_key.as_bytes(), self.fingerprint.as_bytes()].concat();

        sha.update(secret);

        let digest = sha.finalize();

        // Parse the encrypted data.
        let data: Vec<_> = lic
            .enc
            .trim()
            .split('.')
            .map(|v| {
                base64::engine::general_purpose::STANDARD
                    .decode(v)
                    .expect("failed to parse encrypted data")
            })
            .collect();

        // Set up data and AES-GCM
        let mut ciphertext = Vec::from(data[0].as_slice());
        let nonce = Nonce::from_slice(data[1].as_slice());
        let tag = Tag::from_slice(data[2].as_slice());
        let key = Key::from_slice(&digest);
        let aes = Aes256Gcm::new(key);

        // Concat authentication tag with ciphertext.
        ciphertext.extend_from_slice(tag);

        // Decrypt the machine file.
        let plaintext = match aes.decrypt(nonce, ciphertext.as_ref()) {
            Ok(plaintext) => String::from_utf8(plaintext)
                .map_err(|_| Error::ParseErr("Failed decrypting machine file".into()))?,
            Err(_) => return Err(Error::ParseErr("Failed decrypting machine file".into())),
        };

        // parse to json
        let obj: serde_json::Value = serde_json::from_str(&plaintext)
            .map_err(|_| Error::ParseErr("Failed parsing decrypted machine file to json".into()))?;

        // map json
        let machine_license: MachineLicense = serde_json::from_value(obj)
            .map_err(|_| Error::ParseErr("Failed deserializing machine license".into()))?;

        Ok(machine_license)
    }

    fn save_machine_file<R: Runtime>(&self, cert: String, app: &AppHandle<R>) -> Result<()> {
        let path = Self::get_machine_file_path(app)?;

        let mut f = File::create(path)?;
        f.write_all(cert.as_bytes())?;

        Ok(())
    }

    pub(crate) fn remove_machine_file<R: Runtime>(app: &AppHandle<R>) -> Result<()> {
        let path = Self::get_machine_file_path(app)?;
        fs::remove_file(path)?;
        Ok(())
    }

    fn get_machine_file_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        // get app data dir
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or_else(|| Error::PathErr("Can't resolve app data dir".into()))?;

        // get cache dir
        let cache_dir = data_dir.join("keygen");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        // get path
        let path = cache_dir.join("machine.lic");

        Ok(path)
    }
}
