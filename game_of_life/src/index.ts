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
      buf = new Uint8ClampedArray(wasmShim.getWasmBuf(), ptr, canvas.width * canvas.width * 4);
    } catch (e) {
      console.log('Buffer detached.');
      return;
    }

    console.log('canvas rendering');
    // console.log(buf);
    const imageData = new ImageData(buf, canvas.width, canvas.width);
    ctx.putImageData(imageData, 0, 0);
    requestAnimationFrame(tick);
  };

  console.log('Engine loaded');
  engine.init();

  const tick = () => {
    console.log('tick');
    engine.tick();
  };

  tick();
});
