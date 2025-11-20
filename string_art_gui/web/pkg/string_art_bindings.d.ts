/* tslint:disable */
/* eslint-disable */
export class Computation {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  next(): Step | undefined;
}
export class MonocolorPipeline {
  free(): void;
  [Symbol.dispose](): void;
  constructor(image_buffer: Uint8Array);
  build(settings: MonocolorSettings): Computation;
}
export class MonocolorSettings {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  decay: number;
  minNailDistance: number;
  nailCount: number;
  circularNailRadius: number;
}
export class Point {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly x: number;
  readonly y: number;
}
export class Segment {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly start: Point;
  readonly end: Point;
}
export class Step {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  color: number;
  segment: Segment;
  nail: number;
  link: number;
}
export class WasmError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  message: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_step_free: (a: number, b: number) => void;
  readonly __wbg_get_step_color: (a: number) => number;
  readonly __wbg_set_step_color: (a: number, b: number) => void;
  readonly __wbg_get_step_segment: (a: number) => number;
  readonly __wbg_set_step_segment: (a: number, b: number) => void;
  readonly __wbg_get_step_nail: (a: number) => number;
  readonly __wbg_set_step_nail: (a: number, b: number) => void;
  readonly __wbg_get_step_link: (a: number) => number;
  readonly __wbg_set_step_link: (a: number, b: number) => void;
  readonly __wbg_segment_free: (a: number, b: number) => void;
  readonly segment_start: (a: number) => number;
  readonly segment_end: (a: number) => number;
  readonly __wbg_point_free: (a: number, b: number) => void;
  readonly point_x: (a: number) => number;
  readonly point_y: (a: number) => number;
  readonly __wbg_computation_free: (a: number, b: number) => void;
  readonly computation_next: (a: number) => number;
  readonly __wbg_wasmerror_free: (a: number, b: number) => void;
  readonly __wbg_get_wasmerror_message: (a: number) => [number, number];
  readonly __wbg_set_wasmerror_message: (a: number, b: number, c: number) => void;
  readonly __wbg_monocolorpipeline_free: (a: number, b: number) => void;
  readonly monocolorpipeline_new: (a: number, b: number) => [number, number, number];
  readonly monocolorpipeline_build: (a: number, b: number) => [number, number, number];
  readonly __wbg_monocolorsettings_free: (a: number, b: number) => void;
  readonly __wbg_get_monocolorsettings_decay: (a: number) => number;
  readonly __wbg_set_monocolorsettings_decay: (a: number, b: number) => void;
  readonly monocolorsettings_new: () => number;
  readonly monocolorsettings_minNailDistance: (a: number) => number;
  readonly monocolorsettings_set_minNailDistance: (a: number, b: number) => void;
  readonly monocolorsettings_nailCount: (a: number) => number;
  readonly monocolorsettings_set_nailCount: (a: number, b: number) => void;
  readonly monocolorsettings_circularNailRadius: (a: number) => number;
  readonly monocolorsettings_set_circularNailRadius: (a: number, b: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
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
