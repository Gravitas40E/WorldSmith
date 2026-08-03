/* tslint:disable */
/* eslint-disable */

export class Explorer {
    free(): void;
    [Symbol.dispose](): void;
    export_json(): string;
    generate_planet(seed: bigint, radius_m: number, mass_kg: number, stellar_class?: string | null, initial_water_fraction?: number | null): any;
    import_json(json: string): any;
    constructor(seed: bigint);
    planet_state(): any;
    snapshot(): any;
    tick(ticks: number): any;
}

export function run(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_explorer_free: (a: number, b: number) => void;
    readonly explorer_export_json: (a: number) => [number, number, number, number];
    readonly explorer_generate_planet: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
    readonly explorer_import_json: (a: number, b: number, c: number) => [number, number, number];
    readonly explorer_new: (a: bigint) => [number, number, number];
    readonly explorer_planet_state: (a: number) => [number, number, number];
    readonly explorer_snapshot: (a: number) => [number, number, number];
    readonly explorer_tick: (a: number, b: number) => [number, number, number];
    readonly run: () => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
