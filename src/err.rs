use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Path Error: {0}")]
    PathErr(String),
    #[error("Http error: {0}")]
    #[allow(clippy::enum_variant_names)]
    HttpError(String),
    #[error("Failed processing a request: {0}")]
    #[allow(clippy::enum_variant_names)]
    RequestError(#[from] reqwest::Error),
    #[error("Bad response: {0}")]
    BadResponse(String),
    #[error("Bad cache: {0}")]
    BadCache(String),
    #[error("Parse Error: {0}")]
    ParseErr(String),
    #[error("License validation error: {code:?}: {detail:?}")]
    LicenseErr { code: String, detail: String },
    #[error("Keygen API Error: {code:?}: {detail:?}")]
    ApiErr { code: String, detail: String },
}

#[derive(Debug, Serialize)]
pub struct ErrorSummary {
    pub code: String,
    pub detail: String,
}

// to be sent back to js front-end
impl From<Error> for ErrorSummary {
    fn from(value: Error) -> Self {
        match value {
            Error::HttpError(detail) => Self {
                code: "HTTP_ERROR".into(),
                detail,
            },
            Error::RequestError(_) => Self {
                code: "REQUEST_ERROR".into(),
                detail: "Failed sending request: Check your internet".into(),
            },
            Error::BadResponse(detail) => Self {
                code: "BAD_RESPONSE".into(),
                detail,
            },
            Error::BadCache(detail) => Self {
                code: "BAD_CACHE".into(),
                detail,
            },
            Error::LicenseErr { code, detail } => Self { code, detail },
            Error::ApiErr { code, detail } => Self { code, detail },
            err => {
                let msg = match err {
                    Error::Io(err) => err.to_string(),
                    Error::PathErr(err) => err,
                    Error::ParseErr(err) => err,
                    _ => "".into(),
                };

                Self {
                    code: "ERR".into(),
                    detail: msg,
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct APIErrors {
    pub errors: Option<Vec<APIError>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct APIError {
    pub detail: Option<String>,
    pub code: Option<String>,
}

pub fn parse_err_json(status_code: StatusCode, err: serde_json::Value) -> Error {
    if status_code.is_client_error() {
        let api_errors: Result<APIErrors, Error> = serde_json::from_value(err)
            .map_err(|_| Error::ParseErr("Failed deserializing Keygen API error json".into()));

        match api_errors {
            Ok(api_errs) => {
                if let Some(errs) = api_errs.errors {
                    if !errs.is_empty() {
                        // just return the first item on the errors array
                        let code = errs[0].code.clone().unwrap_or_default();
                        let detail = errs[0].detail.clone().unwrap_or_default();

                        return Error::ApiErr { code, detail };
                    }
                }

                Error::ApiErr {
                    code: "UNKNOWN".into(),
                    detail: "Unknown Keygen API Error".into(),
                }
            }
            Err(err) => err,
        }
    } else {
        Error::HttpError(status_code.to_string())
    }
}
