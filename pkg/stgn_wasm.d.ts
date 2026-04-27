/* tslint:disable */
/* eslint-disable */

export function decode_payload(image_bytes: Uint8Array, encryption: string, key: Uint8Array): string;

export function decode_string(image_bytes: Uint8Array): string;

export function decode_string_secure(image_bytes: Uint8Array, encryption: string, key: Uint8Array): string;

export function embed_image_in_pdf(image_bytes: Uint8Array): Uint8Array;

export function encode_max_capacity(image_bytes: Uint8Array): number;

export function encode_payload(image_bytes: Uint8Array, entries_json: string, encryption: string, key: Uint8Array, compress: boolean): Uint8Array;

export function encode_payload_size(entries_json: string, encryption: string, _key: Uint8Array, compress: boolean): number;

export function encode_string(image_bytes: Uint8Array, message: string): Uint8Array;

export function encode_string_secure(image_bytes: Uint8Array, message: string, encryption: string, key: Uint8Array, compress: boolean): Uint8Array;

export function extract_image_from_pdf(pdf_bytes: Uint8Array): Uint8Array;

/**
 * Create a ZIP archive containing the encoded image. Returns the ZIP as Vec<u8>.
 */
export function zip_encoded_image(image_bytes: Uint8Array, filename: string): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly decode_payload: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly decode_string: (a: number, b: number) => [number, number, number, number];
    readonly decode_string_secure: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly embed_image_in_pdf: (a: number, b: number) => [number, number, number, number];
    readonly encode_max_capacity: (a: number, b: number) => [number, number, number];
    readonly encode_payload: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly encode_payload_size: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly encode_string: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly encode_string_secure: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number, number];
    readonly extract_image_from_pdf: (a: number, b: number) => [number, number, number, number];
    readonly zip_encoded_image: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
