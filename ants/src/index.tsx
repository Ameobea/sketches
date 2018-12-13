import React from 'react';
import ReactDOM from 'react-dom';

import { createBackgroundTexture, initWebGL, render } from './webgl';
import { getCanvas } from './canvas';
import UI, { getInitialConf } from './ui';

const wasm = import('./engine');

let canvasScaleFactor = 3;
export const setCanvasScaleFactor = (newScaleFactor: number) =>
  (canvasScaleFactor = newScaleFactor);

let displayGraphics = true;
export const setGraphicsDisplay = (showGraphics: boolean) => {
  console.log(showGraphics);
  displayGraphics = showGraphics;
};
export const canvas_render = (colors: Uint8Array) => {
  if (!displayGraphics) {
    return;
  }

  const textureWidth = Math.sqrt(colors.length);
  createBackgroundTexture(colors);
  render(canvasScaleFactor, 0, 0);
};

let ticks = 0;
let countFood: () => number = () => {
  console.error('`countTicks` called before it was initialized!');
  return 0;
};
let tickTotals: number[] = [];
let tick: null | (() => void) = null;
let intervalHandle: number | null;
let tickDelay = 10.0;
export const register_tick_callback = (minutiaeTick: () => void) => {
  tick = () => {
    minutiaeTick();
    if (ticks % 100 == 0 && ticks != 0) {
      tickTotals.push(countFood());
      if (tickTotals.length == 500) {
        console.log(JSON.stringify(tickTotals));
      } else if (tickTotals.length % 50 == 0 && tickTotals.length > 0) {
        console.log(tickTotals.length);
      }
    }

    ticks += 1;
  };

  intervalHandle = setInterval(tick, tickDelay);
};

export const pause = () => {
  if (intervalHandle) {
    clearInterval(intervalHandle);
    intervalHandle = null;
  }
};

export const resume = (delay?: number) => {
  console.log(intervalHandle);
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
    engine.set_user_conf(JSON.stringify(getInitialConf(false)));
    engine.init();
    countFood = engine.count_collected_food;

    const applyConf = (confJson: string) => {
      engine.set_user_conf(confJson);

      // show universe gen in realtime if we're paused.
      if (!intervalHandle) {
        engine.init_universe();
        pause();
        tick!();
      }

      // persist settings to localstorage
      localStorage.setItem('conf', confJson);
    };
    applyConf(JSON.stringify(getInitialConf()));

    const buttons = [
      {
        type: 'button',
        label: 'reset',
        action: () => {
          tickTotals = [];
          ticks = 0;
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
        action: resume,
      },
      {
        type: 'button',
        label: 'toggle graphics',
        action: () => setGraphicsDisplay(!displayGraphics),
      },
    ];

    const root = document.getElementById('root');
    ReactDOM.render(<UI buttons={buttons} applyConf={applyConf} />, root);
  })
  .catch(console.error);
