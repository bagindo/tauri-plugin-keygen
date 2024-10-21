import { invoke } from "@tauri-apps/api/core";
import { isKeygenError, getErrorMessage, KeygenError } from "./error";

export type KeygenLicense = {
  key: string;
  code: string;
  detail: string;
  /**
   * License expiry can be null at the beginning (before activation)
   * when its policy's expirationBasis is *not* set to "FROM_CREATION".
   */
  expiry: string | null;
  valid: boolean;
  policyId: string;
  entitlements: string[];
  metadata: Record<string, any>;
};

export { KeygenError } from "./error";

function throwError(e: unknown): never {
  if (isKeygenError(e)) {
    throw new KeygenError({ code: e.code, detail: e.detail });
  } else {
    throw new KeygenError({ code: "unknown", detail: getErrorMessage(e) });
  }
}

export async function getLicense(): Promise<KeygenLicense | null> {
  try {
    return (await invoke("plugin:keygen|get_license")) as KeygenLicense | null;
  } catch (e) {
    throwError(e);
  }
}

export async function getLicenseKey(): Promise<string | null> {
  try {
    return (await invoke("plugin:keygen|get_license_key")) as string | null;
  } catch (e) {
    throwError(e);
  }
}

export async function validateKey({
  key,
  entitlements = [],
  cacheValidResponse = true,
}: {
  key: string;
  entitlements?: string[];
  cacheValidResponse?: boolean;
}): Promise<KeygenLicense> {
  try {
    let license = (await invoke("plugin:keygen|validate_key", {
      key,
      entitlements,
      cacheValidResponse,
    })) as KeygenLicense;

    const noMachine =
      license.code === "NO_MACHINE" ||
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
      })) as KeygenLicense;
    }

    return license;
  } catch (e) {
    throwError(e);
  }
}

export async function validateCheckoutKey({
  key,
  entitlements = [],
  ttlSeconds = 86400,
  ttlForever = false,
}: {
  key: string;
  entitlements?: string[];
  ttlSeconds?: number;
  ttlForever?: boolean;
}): Promise<KeygenLicense> {
  try {
    const license = (await validateKey({
      key,
      entitlements,
      cacheValidResponse: false,
    })) as KeygenLicense;

    if (license.valid) {
      await invoke("plugin:keygen|checkout_machine", {
        ttlSeconds,
        ttlForever,
      });
    }

    return license;
  } catch (e) {
    throwError(e);
  }
}

export async function resetLicense(): Promise<void> {
  try {
    return await invoke("plugin:keygen|reset_license");
  } catch (e) {
    if (isKeygenError(e)) {
      throw new KeygenError({ code: e.code, detail: e.detail });
    } else {
      throw new KeygenError({ code: "unknown", detail: getErrorMessage(e) });
    }
  }
}

export async function resetLicenseKey(): Promise<void> {
  try {
    return await invoke("plugin:keygen|reset_license_key");
  } catch (e) {
    throwError(e);
  }
}
