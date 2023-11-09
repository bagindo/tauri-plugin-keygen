use crate::{client::KeygenResponseCache, err::Error, Result};
use base64::Engine;
use reqwest::{header::HeaderMap, Url};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug)]
pub struct KeygenSig {
    sig: String,
    data: KeygenSigData,
}

impl fmt::Display for KeygenSig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sig)
    }
}

#[derive(Debug)]
struct KeygenSigData {
    target: String,
    host: String,
    date: String,
    digest: String,
}

impl KeygenSig {
    pub fn target(&self) -> String {
        self.data.target.clone()
    }

    pub fn host(&self) -> String {
        self.data.host.clone()
    }

    pub fn date(&self) -> String {
        self.data.date.clone()
    }

    pub fn digest(&self) -> String {
        self.data.digest.clone()
    }

    pub fn data(&self) -> String {
        let target = format!("(request-target): {}", self.data.target);
        let host = format!("host: {}", self.data.host);
        let date = format!("date: {}", self.data.date);
        let digest = format!("digest: {}", self.data.digest);

        format!("{}\n{}\n{}\n{}", target, host, date, digest)
    }

    pub fn from_response(
        req_method: String,
        req_url: Url,
        res_headers: &HeaderMap,
        res_text: String,
    ) -> Result<Self> {
        // parse signature
        let sig = Self::parse_signature(res_headers)?;

        // build data
        let data = Self::build_signature_data(req_method, req_url, res_headers, res_text)?;

        Ok(Self { sig, data })
    }

    pub fn from_response_cache(res_cache: KeygenResponseCache) -> Self {
        // re-hash response body
        let mut hasher = Sha256::new();
        hasher.update(res_cache.body);

        let digest = format!(
            "sha-256={}",
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
        );

        Self {
            sig: res_cache.sig,
            data: KeygenSigData {
                target: res_cache.target,
                host: res_cache.host,
                date: res_cache.date,
                digest,
            },
        }
    }

    fn parse_signature(res_headers: &HeaderMap) -> Result<String> {
        // get Keygen-Signature header
        let parameterized_header = res_headers
            .get("Keygen-Signature")
            .ok_or_else(|| Error::BadResponse("Missing header: Keygen-Signature".into()))?
            .to_str()
            .map_err(|_| Error::BadResponse("Failed parsing signature string".into()))?
            .to_string();

        let (algorithm, signature) = Self::parse_header_params(parameterized_header);

        if algorithm.is_none() {
            return Err(Error::BadResponse(
                "Missing header: Keygen-Signature.algorithm".into(),
            ));
        }

        if !"ed25519".to_string().eq(&algorithm.unwrap()) {
            return Err(Error::BadResponse("Unsupported algorithm".into()));
        }

        if signature.is_none() {
            return Err(Error::BadResponse(
                "Missing header: Keygen-Signature.signature".into(),
            ));
        }

        Ok(signature.unwrap())
    }

    fn parse_header_params(parameterized_header: String) -> (Option<String>, Option<String>) {
        // parse algorithm and signature
        let mut algorithm: Option<String> = None;
        let mut signature: Option<String> = None;

        let signature_params = parameterized_header.split(", ");

        for param in signature_params {
            let mut parts = param.splitn(3, '\"');

            if let Some(key) = parts.next() {
                if let Some(val) = parts.next() {
                    if key.starts_with("algorithm") {
                        algorithm = Some(String::from(val));
                    }
                    if key.starts_with("signature") {
                        signature = Some(String::from(val));
                    }
                }
            }
        }

        (algorithm, signature)
    }

    fn build_signature_data(
        req_method: String,
        req_url: Url,
        res_headers: &HeaderMap,
        res_text: String,
    ) -> Result<KeygenSigData> {
        // get target
        let method = req_method.to_lowercase();
        let mut path = req_url.path().to_string();
        if let Some(query) = req_url.query() {
            path.push_str(format!("?{}", query).as_str())
        }
        let target = format!("{} {}", method, path);

        // get host
        let host = req_url
            .host_str()
            .ok_or_else(|| Error::BadResponse("Failed parsing host".into()))?
            .to_string();

        // get Date from headers
        let date = res_headers
            .get("Date")
            .ok_or_else(|| Error::BadResponse("Missing header: Date".into()))?
            .to_str()
            .map_err(|_| Error::BadResponse("Failed parsing signature header: date".into()))?
            .to_string();

        // hash response body
        let mut hasher = Sha256::new();
        hasher.update(res_text);

        let digest = format!(
            "sha-256={}",
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
        );

        Ok(KeygenSigData {
            target,
            host,
            date,
            digest,
        })
    }
}
