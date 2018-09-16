const wasm = import('./engine');

wasm.then(engine => {
  engine.hello();
});
