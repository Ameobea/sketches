const wasm = import('./engine');

export const canvas_render = (colors: Uint8Array) => {
  // TODO
  console.log(colors);
};

let tick: null | (() => void) = null;
export const register_tick_callback = (minutiaeTick: () => void) => {
  tick = minutiaeTick;
  console.log(tick);
};

wasm.then(engine => {
  engine.init();
  engine.hello();
});
