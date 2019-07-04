import { createBackgroundTexture, render } from './webgl';

let canvasScaleFactor = 3;
export const setCanvasScaleFactor = (newScaleFactor: number) => {
  canvasScaleFactor = newScaleFactor;
};

let displayGraphics = true;
export const setGraphicsDisplay = (showGraphics: boolean) => {
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

let tick: null | (() => void) = null;
let intervalHandle: number | null;
let tickDelay = 10.0;
export const register_tick_callback = (minutiaeTick: () => void) => {
  tick = () => {
    minutiaeTick();
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
  if (!intervalHandle) {
    if (delay !== undefined) {
      tickDelay = delay;
    }

    intervalHandle = setInterval(tick!, tickDelay);
  }
};

export const getIntervalHandle = () => intervalHandle;

export const doTick = () => {
  if (!tick) {
    console.error('Tried to tick before the tick callback was registered');
    return;
  }

  tick();
};

export const getDisplayGraphics = () => displayGraphics;
