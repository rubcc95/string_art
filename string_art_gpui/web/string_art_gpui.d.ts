/* tslint:disable */
/* eslint-disable */

export function start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start: () => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___JsValue__core_ed718c3d60ebd546___result__Result_____wasm_bindgen_3c440442e3c4bb80___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___sys__JsNullable_wgpu_77c52e6d9ecf39a5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_3c440442e3c4bb80___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___sys__JsNullable_wgpu_77c52e6d9ecf39a5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_3c440442e3c4bb80___JsError___true__8: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___sys__JsNullable_wgpu_77c52e6d9ecf39a5___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_ed718c3d60ebd546___result__Result_____wasm_bindgen_3c440442e3c4bb80___JsError___true__9: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___JsValue______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___JsValue______true__3: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wasm_bindgen_3c440442e3c4bb80___JsValue______true__7: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wgpu_77c52e6d9ecf39a5___backend__webgpu__webgpu_sys__gen_GpuDeviceLostInfo__GpuDeviceLostInfo______true_: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke___wgpu_77c52e6d9ecf39a5___backend__webgpu__webgpu_sys__gen_GpuDeviceLostInfo__GpuDeviceLostInfo______true__6: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_3c440442e3c4bb80___convert__closures_____invoke_______true_: (a: number, b: number) => void;
    readonly __wbindgen_malloc_command_export: (a: number, b: number) => number;
    readonly __wbindgen_realloc_command_export: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free_command_export: (a: number, b: number, c: number) => void;
    readonly __externref_table_alloc_command_export: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store_command_export: (a: number) => void;
    readonly __wbindgen_destroy_closure_command_export: (a: number, b: number) => void;
    readonly __externref_table_dealloc_command_export: (a: number) => void;
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
