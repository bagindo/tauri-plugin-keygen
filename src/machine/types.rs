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
    pub meta: MachineLicenseMeta,
    pub included: Vec<MachineLicenseIncluded>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MachineLicenseMeta {
    pub expiry: Option<String>,
    pub issued: String,
    pub ttl: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MachineLicenseIncluded {
    #[serde(rename = "licenses")]
    License(IncludedLicense),
    #[serde(rename = "entitlements")]
    Entitlement(IncludedEntitlements),
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedLicense {
    pub id: String,
    pub attributes: IncludedLicenseAttributes,
    pub relationships: IncludedLicenseRelationsips,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncludedLicenseAttributes {
    pub name: Option<String>,
    pub key: String,
    pub expiry: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedLicenseRelationsips {
    pub policy: IncludedLicensePolicy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedLicensePolicy {
    pub data: IncludedLicensePolicyData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedLicensePolicyData {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedEntitlements {
    pub attributes: IncludedEntitlementsAttributes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludedEntitlementsAttributes {
    pub code: String,
}
