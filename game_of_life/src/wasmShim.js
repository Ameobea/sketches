import * as rawWasm from './engine_bg';

export const getWasmBuf = () => rawWasm.memory.buffer;
