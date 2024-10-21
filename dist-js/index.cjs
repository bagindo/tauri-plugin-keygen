'use strict';

var core = require('@tauri-apps/api/core');

async function getLicense() {
    return (await core.invoke("plugin:keygen|get_license"));
}
async function getLicenseKey() {
    return (await core.invoke("plugin:keygen|get_license_key"));
}
async function validateKey({ key, entitlements = [], cacheValidResponse = true, }) {
    let license = (await core.invoke("plugin:keygen|validate_key", {
        key,
        entitlements,
        cacheValidResponse,
    }));
    const noMachine = license.code === "NO_MACHINE" ||
        license.code === "NO_MACHINES" ||
        license.code === "FINGERPRINT_SCOPE_MISMATCH";
    if (noMachine) {
        await core.invoke("plugin:keygen|activate");
        // re-validate: update License object in Tauri App State
        // machine activation response is not "parsable" into KeygenLicense
        license = (await core.invoke("plugin:keygen|validate_key", {
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
        await core.invoke("plugin:keygen|checkout_machine", {
            ttlSeconds,
            ttlForever,
        });
    }
    return license;
}
async function resetLicense() {
    return await core.invoke("plugin:keygen|reset_license");
}
async function resetLicenseKey() {
    return await core.invoke("plugin:keygen|reset_license_key");
}

exports.getLicense = getLicense;
exports.getLicenseKey = getLicenseKey;
exports.resetLicense = resetLicense;
exports.resetLicenseKey = resetLicenseKey;
exports.validateCheckoutKey = validateCheckoutKey;
exports.validateKey = validateKey;
