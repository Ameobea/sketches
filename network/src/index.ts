const wasm = import('./engine');

wasm.then(engine => {
  engine.init();
  engine.hello();
});
