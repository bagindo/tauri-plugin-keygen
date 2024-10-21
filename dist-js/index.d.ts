export declare type KeygenLicense = {
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
export { KeygenError } from "./error";
export declare function getLicense(): Promise<KeygenLicense | null>;
export declare function getLicenseKey(): Promise<string | null>;
export declare function validateKey({ key, entitlements, cacheValidResponse, }: {
    key: string;
    entitlements?: string[];
    cacheValidResponse?: boolean;
}): Promise<KeygenLicense>;
export declare function validateCheckoutKey({ key, entitlements, ttlSeconds, ttlForever, }: {
    key: string;
    entitlements?: string[];
    ttlSeconds?: number;
    ttlForever?: boolean;
}): Promise<KeygenLicense>;
export declare function resetLicense(): Promise<void>;
export declare function resetLicenseKey(): Promise<void>;
