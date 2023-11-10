use super::types::*;
use crate::{err::Error, machine::types::MachineLicense, Result};
use chrono::{DateTime, Local};
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
    pub valid: bool,
}

impl License {
    pub fn should_maintain_access(&self) -> bool {
        self.valid && self.code == "EXPIRED"
    }

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
            if Self::has_machine_file_expired(machine_license.meta.issued, expiry)? {
                return Ok(None);
            }
        }

        Ok(Some(License {
            id: included_lic.id,
            policy_id: included_lic.relationships.policy.data.id,
            key: included_lic.attributes.key,
            code: "VALID".into(),
            detail: "is valid".into(),
            expiry: Some(included_lic.attributes.expiry),
            valid: true,
        }))
    }

    fn has_machine_file_expired(issued: String, expiry: String) -> Result<bool> {
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

        Ok(!valid)
    }
}
