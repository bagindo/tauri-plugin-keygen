import { invoke } from "@tauri-apps/api";

export type KeygenLicense = {
  key: string;
  code: string;
  detail: string;
  expiry: string | null;
  valid: boolean;
  policyId: string;
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
    await activateMachine();

    // re-validate: update License object in Tauri App State
    // activateMachine() response is not "parsable" to KeygenLicense type
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
  ttlSeconds,
  entitlements = [],
}: {
  key: string;
  ttlSeconds: number;
  entitlements?: string[];
}) {
  const license = (await validateKey({
    key,
    entitlements,
    cacheValidResponse: false,
  })) as KeygenLicense;

  if (license.valid) {
    await checkoutMachine({ ttlSeconds });
  }

  return license;
}

export async function activateMachine(): Promise<KeygenLicense> {
  return (await invoke("plugin:keygen|activate")) as KeygenLicense;
}

export async function checkoutMachine({
  ttlSeconds,
}: {
  ttlSeconds: number;
}): Promise<void> {
  const min = 3600; // 1 hour
  const max = 31556952; // 1 year
  const ttl = ttlSeconds >= min && ttlSeconds <= max ? ttlSeconds : 3600;

  await invoke("plugin:keygen|checkout_machine", {
    ttl,
  });
}
