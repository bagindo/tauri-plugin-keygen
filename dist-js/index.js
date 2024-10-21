import { invoke } from '@tauri-apps/api/core';

async function getLicense() {
    return (await invoke("plugin:keygen|get_license"));
}
async function getLicenseKey() {
    return (await invoke("plugin:keygen|get_license_key"));
}
async function validateKey({ key, entitlements = [], cacheValidResponse = true, }) {
    let license = (await invoke("plugin:keygen|validate_key", {
        key,
        entitlements,
        cacheValidResponse,
    }));
    const noMachine = license.code === "NO_MACHINE" ||
        license.code === "NO_MACHINES" ||
        license.code === "FINGERPRINT_SCOPE_MISMATCH";
    if (noMachine) {
        await invoke("plugin:keygen|activate");
        // re-validate: update License object in Tauri App State
        // machine activation response is not "parsable" into KeygenLicense
        license = (await invoke("plugin:keygen|validate_key", {
            key,
            entitlements,
            cacheValidResponse,
        }));
    }
    return license;
}
async function validateCheckoutKey({ key, entitlements = [], ttlSeconds = 86400, ttlForever = false, }) {
    const license = (await validateKey({
        key,
        entitlements,
        cacheValidResponse: false,
    }));
    if (license.valid) {
        await invoke("plugin:keygen|checkout_machine", {
            ttlSeconds,
            ttlForever,
        });
    }
    return license;
}
async function resetLicense() {
    return await invoke("plugin:keygen|reset_license");
}
async function resetLicenseKey() {
    return await invoke("plugin:keygen|reset_license_key");
}

export { getLicense, getLicenseKey, resetLicense, resetLicenseKey, validateCheckoutKey, validateKey };
