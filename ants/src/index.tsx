import React from 'react';
import ReactDOM from 'react-dom';
import ControlPanel from 'react-control-panel';

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

const settings = [
  { type: 'range', label: 'direction_change_chance', min: 1, max: 100, step: 1, initial: 8 },
  { type: 'button', label: 'reset', action: () => console.log('TODO') },
  { type: 'button', label: 'pause', action: () => console.log('TODO') },
  { type: 'button', label: 'resume', action: () => console.log('TODO') },
];

wasm.then(engine => {
  engine.init();

  const App = () => (
    <ControlPanel
      position="top-right"
      title="Simulation Controls"
      settings={settings}
      onChange={(key, val, _state) => {
        switch (key) {
          case 'direction_change_chance': {
            engine.set_wander_transition_chance_percent(parseInt(val));
            break;
          }
          default: {
            console.error(`Unhandled setting key: ${key}`);
          }
        }
      }}
      width={600}
    />
  );

  const root = document.getElementById('root');
  ReactDOM.render(<App />, root);
});
