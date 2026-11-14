/* tslint:disable */
/* eslint-disable */

export function start(): void;

/**
 * Entry point for web workers
 */
export function wasm_thread_entry_point(ptr: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly start: () => void;
    readonly wasm_thread_entry_point: (a: number) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___js_sys_8f0cf26749d52a38___Function_fn_wasm_bindgen_ca90dcd92e2ca8f0___JsValue_____wasm_bindgen_ca90dcd92e2ca8f0___sys__Undefined___js_sys_8f0cf26749d52a38___Function_fn_wasm_bindgen_ca90dcd92e2ca8f0___JsValue_____wasm_bindgen_ca90dcd92e2ca8f0___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___JsValue__core_e1bfa762590861af___result__Result_____wasm_bindgen_ca90dcd92e2ca8f0___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___sys__JsNullable_wgpu_f2cd7869e40ef8d5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_e1bfa762590861af___result__Result_____wasm_bindgen_ca90dcd92e2ca8f0___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___sys__JsNullable_wgpu_f2cd7869e40ef8d5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_e1bfa762590861af___result__Result_____wasm_bindgen_ca90dcd92e2ca8f0___JsError___true__11: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___sys__JsNullable_wgpu_f2cd7869e40ef8d5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_e1bfa762590861af___result__Result_____wasm_bindgen_ca90dcd92e2ca8f0___JsError___true__12: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures________invoke___web_sys_598419f25e3ffb89___features__gen_MessageEvent__MessageEvent______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___js_sys_8f0cf26749d52a38___Array______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___js_sys_8f0cf26749d52a38___Array______true__10: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___js_sys_8f0cf26749d52a38___futures__task__wait_async_polyfill__MessageEvent______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___JsValue______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___JsValue______true__8: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke___wasm_bindgen_ca90dcd92e2ca8f0___JsValue______true__9: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_ca90dcd92e2ca8f0___convert__closures_____invoke_______true_: (a: number, b: number) => void;
    readonly memory: WebAssembly.Memory;
    readonly __wbindgen_malloc_command_export: (a: number, b: number) => number;
    readonly __wbindgen_realloc_command_export: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free_command_export: (a: number, b: number, c: number) => void;
    readonly __externref_table_alloc_command_export: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store_command_export: (a: number) => void;
    readonly __wbindgen_destroy_closure_command_export: (a: number, b: number) => void;
    readonly __externref_table_dealloc_command_export: (a: number) => void;
    readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
    readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
