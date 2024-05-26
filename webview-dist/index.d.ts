export declare type KeygenLicense = {
    key: string;
    code: string;
    detail: string;
    expiry: string | null;
    valid: boolean;
    policyId: string;
};
export declare type KeygenError = {
    code: string;
    detail: string;
};
export declare function getLicense(): Promise<KeygenLicense | null>;
export declare function getLicenseKey(): Promise<string | null>;
export declare function validateKey({ key, shouldActivate, cacheValidResponse, }: {
    key: string;
    shouldActivate?: boolean;
    cacheValidResponse?: boolean;
}): Promise<KeygenLicense>;
export declare function activateMachine(): Promise<KeygenLicense>;
export declare function checkoutMachine(): Promise<void>;
