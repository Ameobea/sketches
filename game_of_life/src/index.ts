const wasm = import('./engine');
const asyncWasmShim = import('./wasmShim');

const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;

export let canvasRender = ptr => console.warn('`canvasRender` called before loaded');

wasm.then(async engine => {
  const wasmShim = await asyncWasmShim;

  canvasRender = (ptr: number) => {
    let buf;
    try {
      buf = new Uint8ClampedArray(wasmShim.getWasmBuf(), ptr, canvas.height * canvas.width * 4);
    } catch (e) {
      console.log(e);
      setTimeout(() => requestAnimationFrame(tick), 0);
      return;
    }

    const imageData = new ImageData(buf, canvas.height, canvas.width);
    ctx.putImageData(imageData, 0, 0);
    setTimeout(() => requestAnimationFrame(tick), 0);
  };

  console.log('Engine loaded');
  engine.init();

  const tick = () => {
    engine.tick();
  };

  tick();
});
