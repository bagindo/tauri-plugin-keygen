use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct MachineFileRes {
    pub data: MachineFileData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MachineFileData {
    pub attributes: MachineFileAttributes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MachineFileAttributes {
    pub certificate: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MachineLicense {
    pub meta: MachineMeta,
    // included array can contain different types
    // but since the request paramater on machine checkout
    // is hardcoded to only include "license",
    // this will do for now
    pub included: Vec<IncludedLicense>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MachineMeta {
    pub expiry: Option<String>,
    pub issued: String,
    pub ttl: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedLicense {
    pub id: String,
    pub attributes: LicenseAttributes,
    pub relationships: LicenseRelationships,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LicenseAttributes {
    pub name: Option<String>,
    pub key: String,
    pub expiry: String,
    pub last_validated: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseRelationships {
    pub policy: LicensePolicy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LicensePolicy {
    pub data: LicensePolicyData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LicensePolicyData {
    pub id: String,
}
