export const getPolicyName = (policyId: string) => {
  if (policyId === "YOUR-TRIAL-POLICY-ID") return "trial";
  return "";
};

export const getLicenseErrMessage = ({
  code,
  detail,
  policyId = "",
}: {
  code: string;
  detail: string;
  policyId?: string;
}) => {
  if (code === "NOT_FOUND") {
    return "This license doesn't exist";
  }

  if (
    code === "SUSPENDED" ||
    code === "EXPIRED" ||
    code === "OVERDUE" ||
    code === "TOO_MANY_MACHINES" ||
    code === "TOO_MANY_CORES" ||
    code === "TOO_MANY_PROCESSES" ||
    code === "BANNED"
  ) {
    return `Your ${getPolicyName(policyId).toUpperCase()} license ${detail}`;
  }

  if (
    code === "NO_MACHINE" ||
    code === "NO_MACHINES" ||
    code === "FINGERPRINT_SCOPE_MISMATCH"
  ) {
    return `Your ${getPolicyName(policyId).toUpperCase()} license hasn't been activated`;
  }

  if (code === "unknown") {
    return `Unknown error: ${detail}`;
  }

  return `Invalid license: ${detail}`;
};
