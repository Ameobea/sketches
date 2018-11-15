const wasm = import('./engine');

const canvas = document.getElementById('canvas')! as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;

export const canvas_render = (colors: Uint8Array) => {
  const imageData = new ImageData(new Uint8ClampedArray(colors), canvas.width, canvas.height);
  ctx.putImageData(imageData, 0, 0);
};

let tick: null | (() => void) = null;
export const register_tick_callback = (minutiaeTick: () => void) => {
  tick = minutiaeTick;
  setInterval(tick, 10);
};

wasm.then(engine => engine.init());
