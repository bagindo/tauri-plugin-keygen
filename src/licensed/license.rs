use super::types::*;
use crate::{
    err::Error,
    machine::types::{MachineLicense, MachineLicenseIncluded},
    Result,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct License {
    #[serde(skip_serializing)]
    pub id: String,
    pub policy_id: String,
    pub key: String,
    pub code: String,
    pub detail: String,
    pub expiry: Option<String>,
    pub entitlements: Vec<String>,
    pub metadata: serde_json::Value,
    pub valid: bool,
}

impl License {
    pub(crate) fn should_maintain_access(&self) -> bool {
        self.valid && self.code == "EXPIRED"
    }

    pub(crate) fn has_expired(&self) -> bool {
        self.expiry
            .clone()
            .and_then(|expiry| DateTime::parse_from_rfc3339(&expiry).ok())
            .map_or(true, |expiry_date| {
                expiry_date.signed_duration_since(Utc::now()).num_minutes() <= 0
            })
    }

    pub(crate) fn from_license_response(lic_res: LicenseResponse) -> Option<Self> {
        match lic_res.data {
            Some(lic_data) => {
                // get license policy id
                let policy_id = lic_data.relationships.policy.data.id;

                // get license entitlements
                let entitlements: Vec<String> = lic_res
                    .meta
                    .scope
                    .and_then(|scope| {
                        scope.get("entitlements").and_then(|entitlements_val| {
                            serde_json::from_value::<Vec<String>>(entitlements_val.clone()).ok()
                        })
                    })
                    .unwrap_or_default();

                Some(Self {
                    id: lic_data.id,
                    policy_id,
                    key: lic_data.attributes.key,
                    code: lic_res.meta.code,
                    detail: lic_res.meta.detail,
                    expiry: lic_data.attributes.expiry,
                    metadata: lic_data.attributes.metadata,
                    entitlements,
                    valid: lic_res.meta.valid,
                })
            }
            None => None,
        }
    }

    pub(crate) fn from_machine_license(machine_license: MachineLicense) -> Result<Option<Self>> {
        // if machine file expiry exists, check validity
        if let Some(expiry) = machine_license.meta.expiry {
            if Self::has_machine_file_expired(machine_license.meta.issued, expiry)? {
                return Ok(None);
            }
        }

        // get included license
        let included_lic = machine_license.included.iter().find_map(|item| match item {
            MachineLicenseIncluded::License(license) => Some(license.clone()),
            _ => None,
        });

        included_lic
            .map(|included_lic| {
                let entitlement_codes = machine_license
                    .included
                    .iter()
                    .filter_map(|item| {
                        if let MachineLicenseIncluded::Entitlement(entitlement) = item {
                            Some(entitlement.attributes.code.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                License {
                    id: included_lic.id,
                    policy_id: included_lic.relationships.policy.data.id,
                    key: included_lic.attributes.key,
                    code: "VALID".into(),
                    detail: "is valid".into(),
                    expiry: Some(included_lic.attributes.expiry),
                    entitlements: entitlement_codes,
                    metadata: included_lic.attributes.metadata,
                    valid: true,
                }
            })
            .map_or(Ok(None), |license| Ok(Some(license)))
    }

    fn has_machine_file_expired(issued: String, expiry: String) -> Result<bool> {
        let now = Utc::now();
        let issued = DateTime::parse_from_rfc3339(&issued)
            .map_err(|_| Error::ParseErr("Failed parsing machine file issued date".into()))?;
        let expiry = DateTime::parse_from_rfc3339(&expiry)
            .map_err(|_| Error::ParseErr("Failed parsing machine file expiry date".into()))?;

        // clock tampering flag
        let minutes_since_issued = now.signed_duration_since(issued).num_minutes();
        // expiration flag
        let minutes_to_expiry = expiry.signed_duration_since(now).num_minutes();

        let valid = minutes_since_issued > 0 && minutes_to_expiry > 0;

        Ok(!valid)
    }
}
