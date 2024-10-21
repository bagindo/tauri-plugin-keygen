'use strict';

var core = require('@tauri-apps/api/core');

function isErrorWithMessage(error) {
    return (typeof error === "object" &&
        error !== null &&
        "message" in error &&
        typeof error.message === "string");
}
function toErrorWithMessage(maybeError) {
    if (isErrorWithMessage(maybeError))
        return maybeError;
    try {
        return new Error(JSON.stringify(maybeError));
    }
    catch {
        // fallback in case there's an error stringifying the maybeError
        // like with circular references for example.
        return new Error(String(maybeError));
    }
}
function getErrorMessage(error) {
    return toErrorWithMessage(error).message;
}
function isKeygenError(error) {
    return (typeof error === "object" &&
        error !== null &&
        "code" in error &&
        "detail" in error &&
        typeof error.code === "string" &&
        typeof error.detail === "string");
}
class KeygenError extends Error {
    constructor({ code, detail }) {
        super(`Keygen Error: ${code}: ${detail}`);
        // This line is needed to restore the correct prototype chain.
        Object.setPrototypeOf(this, new.target.prototype);
        this.name = "KeygenError";
        this.code = code;
        this.detail = detail;
    }
}

function throwError(e) {
    if (isKeygenError(e)) {
        throw new KeygenError({ code: e.code, detail: e.detail });
    }
    else {
        throw new KeygenError({ code: "unknown", detail: getErrorMessage(e) });
    }
}
async function getLicense() {
    try {
        return (await core.invoke("plugin:keygen|get_license"));
    }
    catch (e) {
        throwError(e);
    }
}
async function getLicenseKey() {
    try {
        return (await core.invoke("plugin:keygen|get_license_key"));
    }
    catch (e) {
        throwError(e);
    }
}
async function validateKey({ key, entitlements = [], cacheValidResponse = true, }) {
    try {
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
    catch (e) {
        throwError(e);
    }
}
async function validateCheckoutKey({ key, entitlements = [], ttlSeconds = 86400, ttlForever = false, }) {
    try {
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
    catch (e) {
        throwError(e);
    }
}
async function resetLicense() {
    try {
        return await core.invoke("plugin:keygen|reset_license");
    }
    catch (e) {
        if (isKeygenError(e)) {
            throw new KeygenError({ code: e.code, detail: e.detail });
        }
        else {
            throw new KeygenError({ code: "unknown", detail: getErrorMessage(e) });
        }
    }
}
async function resetLicenseKey() {
    try {
        return await core.invoke("plugin:keygen|reset_license_key");
    }
    catch (e) {
        throwError(e);
    }
}

exports.KeygenError = KeygenError;
exports.getLicense = getLicense;
exports.getLicenseKey = getLicenseKey;
exports.resetLicense = resetLicense;
exports.resetLicenseKey = resetLicenseKey;
exports.validateCheckoutKey = validateCheckoutKey;
exports.validateKey = validateKey;
