import * as React from 'react';
import * as ReactDOM from 'react-dom';

const wasm = import('./engine');
const asyncWasmShim = import('./wasmShim');
import App from './components/App';
import { CANVAS_SCALE_FACTOR } from './util';

const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;

export let canvasRender = ptr => console.warn('`canvasRender` called before loaded');

let paused = false;
let innerTick;

export const pause = () => {
  paused = true;
};
export const resume = () => {
  paused = false;
  innerTick();
};

const getBoardCoords = (canvas: HTMLCanvasElement, e: MouseEvent): { x: number; y: number } => {
  const rect = canvas.getBoundingClientRect();
  return {
    x: Math.floor((e.clientX - rect.left) / CANVAS_SCALE_FACTOR),
    y: Math.floor((e.clientY - rect.top) / CANVAS_SCALE_FACTOR),
  };
};

wasm.then(async engine => {
  const wasmShim = await asyncWasmShim;

  canvasRender = (ptr: number) => {
    let buf;
    try {
      buf = new Uint8ClampedArray(wasmShim.getWasmBuf(), ptr, canvas.height * canvas.width * 4);
    } catch (e) {
      console.log(e);
      setTimeout(() => requestAnimationFrame(innerTick), 0);
      return;
    }

    const imageData = new ImageData(buf, canvas.height, canvas.width);
    ctx.putImageData(imageData, 0, 0);
    if (!paused) {
      setTimeout(() => requestAnimationFrame(innerTick), 0);
    }
  };

  let mouseDown = false;
  let alreadySetValues = new Set();
  canvas.onmousedown = e => {
    const { x, y } = getBoardCoords(canvas, e);
    engine.set_pixel(x, y);
    alreadySetValues.add(`${x},${y}`);
    mouseDown = true;
  };

  canvas.onmouseup = () => {
    mouseDown = false;
    alreadySetValues.clear();
  };

  canvas.onmousemove = e => {
    if (!mouseDown) {
      return;
    }

    // Prevent us toggling pixels back when a mouse moves within the same board square
    const { x, y } = getBoardCoords(canvas, e);
    const cacheEntry = `${x},${y}`;
    if (alreadySetValues.has(cacheEntry)) {
      return;
    }
    alreadySetValues.add(cacheEntry);

    engine.set_pixel(x, y);
  };

  console.log('Engine loaded');
  engine.init();

  innerTick = () => {
    engine.tick();
  };

  innerTick();

  ReactDOM.render(<App engine={engine} />, document.getElementById('root'));
});
