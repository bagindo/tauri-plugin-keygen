pub mod license;
pub mod types;

use crate::{
    client::{KeygenClient, KeygenResponseCache},
    err::{parse_err_json, Error},
    machine::Machine,
    Result,
};
use chrono::Local;
use license::*;
use reqwest::{Method, StatusCode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use tauri::{AppHandle, Manager, Runtime};
use types::*;

#[derive(Debug, Serialize, Default, Clone)]
pub struct LicensedState {
    license: Option<License>,
}

impl LicensedState {
    pub(crate) fn load<R: Runtime>(
        app: &AppHandle<R>,
        client: &KeygenClient,
        machine: &Machine,
    ) -> Result<Self> {
        if let Some(key) = Self::get_cached_license_key(app)? {
            // load from machine file
            match machine.load_machine_file(&key, client, app) {
                Ok(Some(machine_license)) => match License::from_machine_license(machine_license) {
                    Ok(license) => return Ok(Self { license }),
                    Err(e) => {
                        dbg!(e);
                    }
                },
                Ok(None) => {}
                Err(e) => {
                    dbg!(e);
                }
            }

            // load today's response cache
            if let Some((res_cache, cache_path)) = Self::get_response_cache(app, &key)? {
                // house keeping: delete yesterdays' cache
                match Self::clear_response_cache_except(app, cache_path) {
                    Ok(()) => {}
                    Err(err) => {
                        dbg!(err);
                    }
                };

                // verify and parse today's cache
                let lic_res = client.verify_response_cache(res_cache)?;
                if let Some(license) = License::from_license_response(lic_res) {
                    return Ok(Self {
                        license: Some(license),
                    });
                }
            }
        }

        Ok(Self { license: None })
    }

    pub(crate) fn update(&mut self, license: Option<License>) {
        self.license = license;
    }

    pub(crate) fn get_license(&self) -> Option<License> {
        self.license.clone()
    }

    pub(crate) async fn validate_key(
        &mut self,
        key: String,
        entitlements: Vec<String>,
        machine: &Machine,
        client: &KeygenClient,
    ) -> Result<(License, KeygenResponseCache)> {
        // make sure fingerprint is not an empty string
        if machine.fingerprint.is_empty() {
            return Err(Error::LicenseErr {
                code: "NO_FINGERPRINT".into(),
                detail:
                    "Can't validate license for this machine: Failed parsing machine fingerprint"
                        .into(),
            });
        }

        // prepare request
        let url = client.build_url("licenses/actions/validate-key".into(), None)?;

        let mut body = serde_json::json!({
            "meta": {
                "key": key.trim_end(),
                "scope": {
                    "fingerprint": machine.fingerprint.clone()
                }
            }
        });

        if !entitlements.is_empty() {
            body["meta"]["scope"]["entitlements"] = serde_json::json!(entitlements);
        }

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

                        // not found: no data
                        if lic_res.meta.code == "NOT_FOUND" {
                            return Err(Error::LicenseErr {
                                code: lic_res.meta.code,
                                detail: lic_res.meta.detail,
                            });
                        }

                        // license response should have data here
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

    pub(crate) fn cache_license_key<R: Runtime>(key: &String, app: &AppHandle<R>) -> Result<()> {
        let path = Self::get_license_key_cache_path(app)?;

        let mut f = File::create(path)?;
        f.write_all(key.as_bytes())?;

        Ok(())
    }

    pub(crate) fn get_cached_license_key<R: Runtime>(app: &AppHandle<R>) -> Result<Option<String>> {
        let path = Self::get_license_key_cache_path(app)?;

        // no license key
        if !path.exists() {
            return Ok(None);
        }

        let key = fs::read_to_string(path)?;

        Ok(Some(key))
    }

    pub(crate) fn remove_cached_license_key<R: Runtime>(app: &AppHandle<R>) -> Result<()> {
        let path = Self::get_license_key_cache_path(app)?;

        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }

    fn get_license_key_cache_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        // get app data dir
        let Ok(data_dir) = app.path().app_data_dir() else {
            return Err(Error::PathErr("Can't resolve app data dir".into()));
        };
        // get cache dir
        let keygen_cache_dir = data_dir.join("keygen");

        if !keygen_cache_dir.exists() {
            fs::create_dir_all(&keygen_cache_dir)?;
        }

        // dir path
        let dir_path = keygen_cache_dir.join("key");

        Ok(dir_path)
    }

    pub(crate) fn cache_response<R: Runtime>(
        app: &AppHandle<R>,
        license_key: &String,
        cache: KeygenResponseCache,
    ) -> Result<()> {
        // cache path
        let path = Self::get_response_cache_path(app, license_key)?;

        // cache content
        let cache_text = serde_json::to_string(&cache)
            .map_err(|_| Error::ParseErr("Failed parsing response cache to text".into()))?;

        let mut f = File::create(path)?;
        f.write_all(cache_text.as_bytes())?;

        Ok(())
    }

    pub(crate) fn clear_response_cache<R: Runtime>(app: &AppHandle<R>) -> Result<()> {
        // get cache dir
        let keygen_cache_dir = Self::get_response_cache_dir(app)?;

        if keygen_cache_dir.exists() {
            fs::remove_dir_all(&keygen_cache_dir)?;
        }

        Ok(())
    }

    fn clear_response_cache_except<R: Runtime>(
        app: &AppHandle<R>,
        excluded_path: PathBuf,
    ) -> Result<()> {
        // get cache dir
        let keygen_cache_dir = Self::get_response_cache_dir(app)?;

        // delete all files except the excluded_path
        if keygen_cache_dir.exists() && keygen_cache_dir.is_dir() {
            for entry in fs::read_dir(keygen_cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path != excluded_path {
                    fs::remove_file(path)?;
                }
            }
        }

        Ok(())
    }

    fn get_response_cache<R: Runtime>(
        app: &AppHandle<R>,
        license_key: &String,
    ) -> Result<Option<(KeygenResponseCache, PathBuf)>> {
        // cache path
        let path = Self::get_response_cache_path(app, license_key)?;

        // no license cache
        if !path.exists() {
            return Ok(None);
        }

        // cache content
        let cache_text = fs::read_to_string(&path)?;
        let cache_json: serde_json::Value = serde_json::from_str(&cache_text)
            .map_err(|_| Error::ParseErr("Failed parsing response cache to json".into()))?;
        let cache: KeygenResponseCache = serde_json::from_value(cache_json)
            .map_err(|_| Error::ParseErr("Failed deserializing response cache".into()))?;

        Ok(Some((cache, path)))
    }

    fn get_response_cache_path<R: Runtime>(
        app: &AppHandle<R>,
        license_key: &String,
    ) -> Result<PathBuf> {
        // get cache dir
        let keygen_cache_dir = Self::get_response_cache_dir(app)?;

        if !keygen_cache_dir.exists() {
            fs::create_dir_all(&keygen_cache_dir)?;
        }

        // get path
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", license_key, Local::now().date_naive(),));
        let path = format!("{:x}", hasher.finalize());

        // dir path
        let dir_path = keygen_cache_dir.join(path);

        Ok(dir_path)
    }

    fn get_response_cache_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        // get app data dir
        let Ok(data_dir) = app.path().app_data_dir() else {
            return Err(Error::PathErr("Can't resolve app data dir".into()));
        };

        // get cache dir
        let keygen_cache_dir = data_dir.join("keygen/validation_cache");

        Ok(keygen_cache_dir)
    }
}
