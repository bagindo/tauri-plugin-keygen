import { invoke } from "@tauri-apps/api";

export type KeygenLicense = {
  key: string;
  code: string;
  detail: string;
  expiry: string | null;
  valid: string;
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

export async function validateLicense({
  key,
  shouldActivate = true,
  cacheResponse = true,
}: {
  key: string;
  shouldActivate?: boolean;
  cacheResponse?: boolean;
}): Promise<KeygenLicense> {
  const license = (await invoke("plugin:keygen|validate", {
    key,
    cacheResponse,
  })) as KeygenLicense;

  if (shouldActivate && licenseNotActivated(license)) {
    return await activateMachine();
  }

  return license;
}

export async function activateMachine(): Promise<KeygenLicense> {
  return (await invoke("plugin:keygen|activate")) as KeygenLicense;
}

export function licenseNotActivated(license: KeygenLicense): boolean {
  return (
    license.code === "NO_MACHINE" ||
    license.code === "NO_MACHINES" ||
    license.code === "FINGERPRINT_SCOPE_MISMATCH"
  );
}

export async function checkoutMachine(): Promise<void> {
  await invoke("plugin:keygen|checkout_machine");
}
