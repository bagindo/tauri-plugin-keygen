export declare function getErrorMessage(error: unknown): string;
declare type ErrorWithCodeDetail = {
    code: string;
    detail: string;
};
export declare function isKeygenError(error: unknown): error is ErrorWithCodeDetail;
export declare class KeygenError extends Error {
    code: string;
    detail: string;
    constructor({ code, detail }: {
        code: string;
        detail: string;
    });
}
export {};
