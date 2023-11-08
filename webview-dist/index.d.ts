export declare type KeygenLicense = {
    key: string;
    code: string;
    detail: string;
    expiry: string;
    valid: string;
    policyId: string;
};
export declare type KeygenError = {
    code: string;
    detail: string;
};
export declare function isLicensed(): Promise<boolean>;
export declare function getLicense(): Promise<KeygenLicense | null>;
export declare function getLicenseKey(): Promise<string | null>;
export declare function canUpdate(): Promise<boolean>;
export declare function validateLicense({ key, cacheResponse, }: {
    key: string;
    cacheResponse?: boolean;
}): Promise<KeygenLicense>;
export declare function activateMachine(): Promise<KeygenLicense>;
export declare function licenseNotActivated(license: KeygenLicense): boolean;
export declare function checkoutMachine(): Promise<void>;
