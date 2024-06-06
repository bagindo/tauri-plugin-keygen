pub mod sig;

use crate::{err::Error, licensed::types::LicenseResponse, KeygenVersion, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use hex::FromHex;
use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Method, RequestBuilder, Response, Url,
};
use serde::{Deserialize, Serialize};
use sig::KeygenSig;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct KeygenClient {
    custom_domain: Option<String>,
    account_id: Option<String>,
    verify_key: String,
    api_url: Option<String>,
    api_version: u8, // Keygen API major version
    http_client: reqwest::Client,
    max_clock_drift: i64, // in minutes
    cache_lifetime: i64,  // in minutes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeygenResponseCache {
    pub sig: String,
    pub target: String,
    pub host: String,
    pub date: String,
    pub body: String,
}

impl KeygenClient {
    pub(crate) fn new(
        custom_domain: Option<String>,
        api_url: Option<String>,
        account_id: Option<String>,
        verify_key: String,
        version_header: Option<KeygenVersion>,
        cache_lifetime: i64,
        user_agent: String,
    ) -> Self {
        // client with default headers
        let default_headers = Self::get_default_headers(user_agent, version_header);

        let http_client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap_or_default();

        Self {
            custom_domain,
            account_id,
            verify_key,
            api_url,
            api_version: 1,
            http_client,
            max_clock_drift: 5,
            cache_lifetime,
        }
    }

    fn get_default_headers(user_agent: String, version_header: Option<KeygenVersion>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, Self::get_header_value(&user_agent));

        if let Some(v) = version_header {
            headers.insert("Keygen-Version", Self::get_header_value(&v.to_string()));
        }

        headers
    }

    fn get_header_value(input: &str) -> HeaderValue {
        // HeaderValue::from_str() only allows visible ASCII characters (32-127).
        // This might be an extra un-necessary precaution,
        // but this way it's safe to unwrap(), instead of having to return a Result type.
        let input = input
            .chars()
            .filter(|c| {
                let ascii = *c as u32;
                (32..=127).contains(&ascii)
            })
            .collect::<String>();

        HeaderValue::from_str(&input).unwrap()
    }

    pub(crate) fn post(&self, url: String) -> RequestBuilder {
        self.http_client.request(Method::POST, url)
    }

    pub(crate) fn build_url(&self, path: String, params: Option<Vec<(&str, &str)>>) -> Result<Url> {
        // get base url
        let base_url = self.get_base_url()?;

        // get full path
        let full_path = self.get_full_path(&path)?;

        let mut url = base_url
            .join(&full_path)
            .map_err(|_| Error::ParseErr("Failed to join path to base url".into()))?;

        if let Some(params) = params {
            for (key, value) in params {
                url.query_pairs_mut().append_pair(key, value);
            }
        }

        Ok(url)
    }

    fn get_base_url(&self) -> Result<Url> {
        if let Some(custom_domain) = &self.custom_domain {
            let url = Url::parse(custom_domain).map_err(|_| {
                Error::ParseErr(format!("Failed parsing base url: {}", custom_domain))
            })?;
            return Ok(url);
        }

        if let Some(api_url) = &self.api_url {
            let url = Url::parse(api_url)
                .map_err(|_| Error::ParseErr(format!("Failed parsing base url: {}", api_url)))?;
            return Ok(url);
        }

        Err(Error::ParseErr(
            "Failed getting base url: neither custom_domain nor api_url is provided".into(),
        ))
    }

    fn get_full_path(&self, path: &String) -> Result<String> {
        if self.custom_domain.is_some() {
            return Ok(format!("v{}/{}", self.api_version, path));
        }

        if self.api_url.is_some() {
            if let Some(account_id) = &self.account_id {
                return Ok(format!(
                    "v{}/accounts/{}/{}",
                    self.api_version, account_id, path
                ));
            } else {
                return Err(Error::ParseErr(
                    "Error parsing url: missing account_id".into(),
                ));
            }
        }

        Err(Error::ParseErr("Error parsing url path".into()))
    }

    // both response.text() and response.json() consumed its Self.
    // can't call .text() after calling .json() - or vice versa.
    // and since Response doesn't implement Clone..
    pub async fn res_text_json(&self, response: Response) -> Result<(String, serde_json::Value)> {
        let res_text = response
            .text()
            .await
            .map_err(|_| Error::ParseErr("Failed parsing response text".into()))?;

        let res_json: serde_json::Value = serde_json::from_str(&res_text)
            .map_err(|_| Error::ParseErr("Failed parsing response json".into()))?;

        Ok((res_text, res_json))
    }

    pub(crate) fn verify_response(
        &self,
        req_method: String,
        req_url: Url,
        res_headers: HeaderMap,
        res_text: String,
    ) -> Result<KeygenResponseCache> {
        // get signature
        let sig = KeygenSig::from_response(req_method, req_url, &res_headers, res_text.clone())?;

        // get Digest from response headers
        let res_digest = res_headers
            .get("Digest")
            .ok_or_else(|| Error::BadResponse("Missing header: Digest".into()))?;

        // verify integrity
        if !sig.digest().eq(res_digest) {
            return Err(Error::BadResponse("Digest didn't match".into()));
        }

        // get duration since response date
        let date_time = DateTime::parse_from_rfc2822(&sig.date())
            .map_err(|_| Error::BadResponse("Invalid signature date".into()))?;

        let minutes_since_response = Utc::now().signed_duration_since(date_time).num_minutes();

        // check request date
        if self.max_clock_drift >= 0 && minutes_since_response > self.max_clock_drift {
            return Err(Error::BadResponse("Request date too old".into()));
        }

        // verify signature
        match self.verify_signature(sig.data(), sig.to_string()) {
            Ok(()) => Ok(KeygenResponseCache {
                sig: sig.to_string(),
                target: sig.target(),
                host: sig.host(),
                date: sig.date(),
                body: res_text,
            }),
            Err(err) => {
                dbg!(err);
                Err(Error::BadResponse("Invalid Signature".into()))
            }
        }
    }

    pub(crate) fn verify_response_cache(
        &self,
        res_cache: KeygenResponseCache,
        cache_path: PathBuf,
    ) -> Result<LicenseResponse> {
        let res_text = res_cache.body.clone();
        let sig = KeygenSig::from_response_cache(res_cache);

        // get duration since response date
        let date_time = DateTime::parse_from_rfc2822(&sig.date())
            .map_err(|_| Error::BadCache("Failed parsing cached response date".into()))?;

        let minutes_since_response = Utc::now().signed_duration_since(date_time).num_minutes();

        // check request date
        if minutes_since_response > self.cache_lifetime {
            fs::remove_file(cache_path)?;
            return Err(Error::BadCache("Validation cache has expired".into()));
        }

        // verify signature
        match self.verify_signature(sig.data(), sig.to_string()) {
            Ok(()) => {
                // get json from res text
                let res_json: serde_json::Value = serde_json::from_str(&res_text)
                    .map_err(|_| Error::BadCache("Failed parsing cached response body".into()))?;

                // get validation
                let lic_res: LicenseResponse = serde_json::from_value(res_json)
                    .map_err(|_| Error::BadCache("Failed deserializing license response".into()))?;

                Ok(lic_res)
            }
            Err(err) => {
                dbg!(err);
                fs::remove_file(cache_path)?;
                Err(Error::BadCache("Invalid Signature".into()))
            }
        }
    }

    pub(crate) fn verify_signature(
        &self,
        data: String,
        signature: String,
    ) -> std::result::Result<(), Error> {
        // init key
        let key = match <[u8; PUBLIC_KEY_LENGTH]>::from_hex(self.verify_key.as_str()) {
            Ok(bytes) => VerifyingKey::from_bytes(&bytes),
            Err(_) => return Err(Error::ParseErr("Failed parsing verify key to bytes".into())),
        }
        .map_err(|_| Error::ParseErr("Failed parsing verifying key".into()))?;

        // decode signature
        let sig = base64::engine::general_purpose::STANDARD
            .decode(signature.as_str())
            .map_err(|_| Error::ParseErr("Failed decoding signature".into()))?;

        let sig: [u8; SIGNATURE_LENGTH] = match sig.try_into() {
            Ok(sig) => sig,
            Err(_) => return Err(Error::ParseErr("Invalid signature format".into())),
        };

        // verify
        match key.verify(data.as_bytes(), &sig.into()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::ParseErr("Invalid signature".into())),
        }
    }
}
