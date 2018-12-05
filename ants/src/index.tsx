import React from 'react';
import ReactDOM from 'react-dom';

import { createBackgroundTexture, initWebGL, render } from './webgl';
import { getCanvas } from './canvas';
import UI from './ui';

const wasm = import('./engine');

let canvasScaleFactor = 3;
export const setCanvasScaleFactor = (newScaleFactor: number) =>
  (canvasScaleFactor = newScaleFactor);

export const canvas_render = (colors: Uint8Array) => {
  const textureWidth = Math.sqrt(colors.length);
  createBackgroundTexture(colors);
  render(canvasScaleFactor, 0, 0);
};

let tick: null | (() => void) = null;
let intervalHandle: number | null;
export const register_tick_callback = (minutiaeTick: () => void) => {
  tick = minutiaeTick;
  intervalHandle = setInterval(tick, 10);
};

export const pause = () => {
  if (intervalHandle) {
    clearInterval(intervalHandle);
    intervalHandle = null;
  }
};

let tickDelay = 10.0;
export const resume = (delay?: number) => {
  if (!intervalHandle) {
    if (delay !== undefined) {
      tickDelay = delay;
    }

    intervalHandle = setInterval(tick!, tickDelay);
  }
};

wasm
  .then(engine => {
    initWebGL();
    engine.init();

    const buttons = [
      {
        type: 'button',
        label: 'reset',
        action: () => {
          pause();
          // Delete the old universe, freeing associated resources, and trigger a new tick callback
          // to be set.
          engine.init_universe();
        },
      },
      {
        type: 'button',
        label: 'pause',
        action: pause,
      },
      {
        type: 'button',
        label: 'resume',
        action: () => resume,
      },
    ];

    const root = document.getElementById('root');
    ReactDOM.render(<UI buttons={buttons} setConf={engine.set_user_conf} />, root);
  })
  .catch(console.error);
