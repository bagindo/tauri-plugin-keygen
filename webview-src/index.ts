import { invoke } from "@tauri-apps/api";

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

export type KeygenError = {
  code: string;
  detail: string;
};

export async function getLicense(): Promise<KeygenLicense | null> {
  return (await invoke("plugin:keygen|get_license")) as KeygenLicense | null;
}

export async function getLicenseKey(): Promise<string | null> {
  return (await invoke("plugin:keygen|get_license_key")) as string | null;
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
}

export async function resetLicense(): Promise<void> {
  return await invoke("plugin:keygen|reset_license");
}

export async function resetLicenseKey(): Promise<void> {
  return await invoke("plugin:keygen|reset_license_key");
}
