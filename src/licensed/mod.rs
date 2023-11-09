pub mod types;

use crate::{
    client::{KeygenClient, KeygenResponseCache},
    err::{parse_err_json, Error},
    machine::{types::MachineLicense, Machine},
    Result,
};
use chrono::{DateTime, Local};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use tauri::{AppHandle, Runtime};
use types::*;

#[derive(Debug, Serialize, Default, Clone)]
pub struct LicensedState {
    license: Option<License>,
}

impl LicensedState {
    pub fn load<R: Runtime>(
        app: &AppHandle<R>,
        client: &KeygenClient,
        machine: &Machine,
    ) -> Result<Self> {
        if let Some(key) = Self::get_cached_license_key(app)? {
            // load from response cache
            if let Some(res_cache) = Self::get_response_cache(app, key.clone())? {
                let lic_res = client.verify_response_cache(res_cache)?;
                let license = License::from_license_response(lic_res);
                return Ok(Self { license });
            }

            // load from machine file
            if let Some(machine_license) = machine.load_machine_file(key, client, app)? {
                let license = License::from_machine_license(machine_license)?;
                return Ok(Self { license });
            }
        }

        Ok(Self { license: None })
    }

    pub(crate) fn update<R: Runtime>(
        &mut self,
        license: License,
        app: &AppHandle<R>,
    ) -> Result<()> {
        // update state
        self.license = Some(license.clone());

        // save license key
        Self::cache_license_key(license.key, app)?;

        Ok(())
    }

    pub fn has_valid_license(&self) -> bool {
        match self.license.clone() {
            Some(license) => license.valid,
            None => false,
        }
    }

    pub fn get_license(&self) -> Option<License> {
        self.license.clone()
    }

    pub async fn validate(
        &mut self,
        key: String,
        machine: &Machine,
        client: &KeygenClient,
    ) -> Result<(License, KeygenResponseCache)> {
        // prepare request
        let url = client.build_url("licenses/actions/validate-key".into(), None)?;
        let body = serde_json::json!({
            "meta": {
                "key": key.trim_end(),
                "scope": {
                    "fingerprint": machine.id.clone()
                }
            }
        });

        // request validation
        let response = client
            .post(url.to_string())
            .timeout(Duration::from_secs(90))
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .json(&body)
            .send()
            .await?;

        // extract response data
        let res_status = response.status();
        let res_headers = response.headers().clone();
        let (res_text, res_json) = client.res_text_json(response).await?;

        match res_status {
            StatusCode::OK => {
                // verify signature
                match client.verify_response(Method::POST.to_string(), url, res_headers, res_text) {
                    Ok(res_cache) => {
                        // map res_json
                        let lic_res: LicenseResponse =
                            serde_json::from_value(res_json).map_err(|_| {
                                Error::ParseErr(
                                    "Failed deserializing license validation response".into(),
                                )
                            })?;

                        // not found
                        if lic_res.meta.code == "NOT_FOUND" {
                            return Err(Error::LicenseErr {
                                code: lic_res.meta.code,
                                detail: lic_res.meta.detail,
                            });
                        }

                        // lic_res should have data here
                        // but just to be safe..
                        let license = License::from_license_response(lic_res).ok_or_else(|| {
                            Error::BadResponse("Missing data on license validation response".into())
                        })?;

                        Ok((license, res_cache))
                    }
                    Err(err) => Err(err),
                }
            }
            status_code => Err(parse_err_json(status_code, res_json)),
        }
    }

    fn cache_license_key<R: Runtime>(key: String, app: &AppHandle<R>) -> Result<()> {
        let path = Self::license_key_cache_path(app)?;

        let mut f = File::create(path)?;
        f.write_all(key.as_bytes())?;

        Ok(())
    }

    pub fn get_cached_license_key<R: Runtime>(app: &AppHandle<R>) -> Result<Option<String>> {
        let path = Self::license_key_cache_path(app)?;

        // no license key
        if !path.exists() {
            return Ok(None);
        }

        let key = read_to_string(path)?;

        Ok(Some(key))
    }

    fn license_key_cache_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        // get app data dir
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or_else(|| Error::PathErr("Can't resolve app data dir".into()))?;

        // get cache dir
        let keygen_cache_dir = data_dir.join("keygen");

        if !keygen_cache_dir.exists() {
            create_dir_all(&keygen_cache_dir)?;
        }

        // dir path
        let dir_path = keygen_cache_dir.join("key");

        Ok(dir_path)
    }

    pub(crate) fn cache_response<R: Runtime>(
        app: &AppHandle<R>,
        license_key: String,
        cache: KeygenResponseCache,
    ) -> Result<()> {
        // cache path
        let path = Self::response_cache_path(app, license_key)?;

        // cache content
        let cache_text = serde_json::to_string(&cache)
            .map_err(|_| Error::ParseErr("Failed parsing response cache to text".into()))?;

        let mut f = File::create(path)?;
        f.write_all(cache_text.as_bytes())?;

        Ok(())
    }

    fn get_response_cache<R: Runtime>(
        app: &AppHandle<R>,
        license_key: String,
    ) -> Result<Option<KeygenResponseCache>> {
        // cache path
        let path = Self::response_cache_path(app, license_key)?;

        // no license cache
        if !path.exists() {
            return Ok(None);
        }

        // cache content
        let cache_text = read_to_string(path)?;
        let cache_json: serde_json::Value = serde_json::from_str(&cache_text)
            .map_err(|_| Error::ParseErr("Failed parsing response cache to json".into()))?;
        let cache: KeygenResponseCache = serde_json::from_value(cache_json)
            .map_err(|_| Error::ParseErr("Failed deserializing response cache".into()))?;

        Ok(Some(cache))
    }

    fn response_cache_path<R: Runtime>(app: &AppHandle<R>, license_key: String) -> Result<PathBuf> {
        // get app data dir
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or_else(|| Error::PathErr("Can't resolve app data dir".into()))?;

        // get cache dir
        let keygen_cache_dir = data_dir.join("keygen/validation_cache");

        if !keygen_cache_dir.exists() {
            create_dir_all(&keygen_cache_dir)?;
        }

        // get path
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", license_key, Local::now().date_naive(),));
        let path = format!("{:x}", hasher.finalize());

        // dir path
        let dir_path = keygen_cache_dir.join(path);

        Ok(dir_path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct License {
    #[serde(skip_serializing)]
    pub id: String,
    pub policy_id: String,
    pub key: String,
    pub code: String,
    pub detail: String,
    pub expiry: String,
    #[serde(skip_serializing)]
    pub last_validated: String,
    pub valid: bool,
}

impl License {
    pub fn from_license_response(lic_res: LicenseResponse) -> Option<Self> {
        match lic_res.data {
            Some(lic_data) => {
                let lic_policy = lic_data.relationships.policy;

                Some(Self {
                    id: lic_data.id,
                    policy_id: lic_policy.data.id,
                    key: lic_data.attributes.key,
                    code: lic_res.meta.code,
                    detail: lic_res.meta.detail,
                    expiry: lic_data.attributes.expiry,
                    last_validated: lic_data.attributes.last_validated,
                    valid: lic_res.meta.valid,
                })
            }
            None => None,
        }
    }

    pub fn from_machine_license(machine_license: MachineLicense) -> Result<Option<Self>> {
        // has license details
        if machine_license.included.is_empty() {
            return Ok(None);
        }

        // get license detail
        let included_lic = machine_license.included[0].clone();

        // if machine file expiry isn't set to null, check validity
        if let Some(expiry) = machine_license.meta.expiry {
            if !Self::is_machine_file_valid(machine_license.meta.issued, expiry)? {
                return Ok(None);
            }
        }

        Ok(Some(License {
            id: included_lic.id,
            policy_id: included_lic.relationships.policy.data.id,
            key: included_lic.attributes.key,
            code: "VALID".into(),
            detail: "is valid".into(),
            expiry: included_lic.attributes.expiry,
            last_validated: included_lic.attributes.last_validated,
            valid: true,
        }))
    }

    fn is_machine_file_valid(issued: String, expiry: String) -> Result<bool> {
        let now = Local::now();
        let issued = DateTime::parse_from_rfc3339(&issued)
            .map_err(|_| Error::ParseErr("Failed parsing machine file issued date".into()))?;
        let expiry = DateTime::parse_from_rfc3339(&expiry)
            .map_err(|_| Error::ParseErr("Failed parsing machine file expiry date".into()))?;

        // clock tampering flag
        let minutes_since_issued = now.signed_duration_since(issued).num_minutes();
        // expiration flag
        let minutes_to_expiry = expiry.signed_duration_since(now).num_minutes();

        let valid = minutes_since_issued > 0 && minutes_to_expiry > 0;

        Ok(valid)
    }

    // flag for checking update
    pub fn has_not_expired(&self) -> Result<bool> {
        let now = Local::now();
        let issued = DateTime::parse_from_rfc3339(&self.last_validated)
            .map_err(|_| Error::ParseErr("Failed parsing license lastValidated date".into()))?;
        let expiry = DateTime::parse_from_rfc3339(&self.expiry)
            .map_err(|_| Error::ParseErr("Failed parsing license expiry date".into()))?;

        // clock tampering flag
        let minutes_since_issued = now.signed_duration_since(issued).num_minutes();
        // expiration flag
        let minutes_to_expiry = expiry.signed_duration_since(now).num_minutes();

        let not_expired = minutes_since_issued > 0 && minutes_to_expiry > 0;

        Ok(not_expired)
    }

    pub fn should_maintain_access(&self) -> bool {
        self.valid && self.code == "EXPIRED"
    }
}
