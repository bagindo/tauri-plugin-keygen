type ErrorWithMessage = {
  message: string;
};

function isErrorWithMessage(error: unknown): error is ErrorWithMessage {
  return (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof (error as Record<string, unknown>).message === "string"
  );
}

function toErrorWithMessage(maybeError: unknown): ErrorWithMessage {
  if (isErrorWithMessage(maybeError)) return maybeError;

  try {
    return new Error(JSON.stringify(maybeError));
  } catch {
    // fallback in case there's an error stringifying the maybeError
    // like with circular references for example.
    return new Error(String(maybeError));
  }
}

export function getErrorMessage(error: unknown) {
  return toErrorWithMessage(error).message;
}

type ErrorWithCodeDetail = {
  code: string;
  detail: string;
};

export function isKeygenError(error: unknown): error is ErrorWithCodeDetail {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "detail" in error &&
    typeof (error as Record<string, unknown>).code === "string" &&
    typeof (error as Record<string, unknown>).detail === "string"
  );
}

export class KeygenError extends Error {
  public code: string;
  public detail: string;

  constructor({ code, detail }: { code: string; detail: string }) {
    super(`Keygen Error: ${code}: ${detail}`);

    // This line is needed to restore the correct prototype chain.
    Object.setPrototypeOf(this, new.target.prototype);

    this.name = "KeygenError";
    this.code = code;
    this.detail = detail;
  }
}
