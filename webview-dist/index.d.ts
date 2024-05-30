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
export declare function validateKey({ key, cacheValidResponse, }: {
    key: string;
    cacheValidResponse?: boolean;
}): Promise<KeygenLicense>;
export declare function activateMachine(): Promise<KeygenLicense>;
export declare function checkoutMachine({ ttlSeconds, }: {
    ttlSeconds?: number;
}): Promise<void>;
