export declare type KeygenLicense = {
    key: string;
    code: string;
    detail: string;
    expiry: string | null;
    valid: string;
    policyId: string;
};
export declare type KeygenError = {
    code: string;
    detail: string;
};
export declare function getLicense(): Promise<KeygenLicense | null>;
export declare function getLicenseKey(): Promise<string | null>;
export declare function validateLicense({ key, shouldActivate, cacheResponse, }: {
    key: string;
    shouldActivate?: boolean;
    cacheResponse?: boolean;
}): Promise<KeygenLicense>;
export declare function activateMachine(): Promise<KeygenLicense>;
export declare function licenseNotActivated(license: KeygenLicense): boolean;
export declare function checkoutMachine(): Promise<void>;
