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

export async function validateKey({
  key,
  shouldActivate = true,
  cacheValidResponse = true,
}: {
  key: string;
  shouldActivate?: boolean;
  cacheValidResponse?: boolean;
}): Promise<KeygenLicense> {
  let license = (await invoke("plugin:keygen|validate_key", {
    key,
    cacheValidResponse,
  })) as KeygenLicense;

  const noMachine =
    license.code === "NO_MACHINE" ||
    license.code === "NO_MACHINES" ||
    license.code === "FINGERPRINT_SCOPE_MISMATCH";

  if (noMachine && shouldActivate) {
    await activateMachine();

    // re-validate
    license = (await invoke("plugin:keygen|validate_key", {
      key,
      cacheValidResponse,
    })) as KeygenLicense;
  }

  return license;
}

export async function activateMachine(): Promise<KeygenLicense> {
  return (await invoke("plugin:keygen|activate")) as KeygenLicense;
}

export async function checkoutMachine(): Promise<void> {
  await invoke("plugin:keygen|checkout_machine");
}
