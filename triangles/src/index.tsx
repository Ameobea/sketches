import React from 'react';
import ReactDOM from 'react-dom';
import Panel from 'react-control-panel';

const wasm = import('./engine');

const SVG: HTMLElement = document.getElementById('svg') as any;

let renderIx: number = 0;

export const render_triangle = (
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  x3: number,
  y3: number,
  color: string,
  border_color: string
) => {
  renderIx += 1;
  const poly = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
  poly.setAttribute('points', `${x1},${y1} ${x2},${y2} ${x3},${y3}`);
  poly.setAttribute('style', `fill:${color};stroke:${border_color};stroke-width:1`);
  poly.setAttribute('id', `poly-${renderIx}`);
  SVG.appendChild(poly);
  return renderIx;
};

export const render_quad = (
  x: number,
  y: number,
  width: number,
  height: number,
  color: string,
  border_color: string
) => {
  const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
  rect.setAttribute('x', x.toString());
  rect.setAttribute('y', y.toString());
  rect.setAttribute('width', width.toString());
  rect.setAttribute('height', height.toString());
  rect.setAttribute('style', `fill:${color};stroke:${border_color};stroke-width:1`);
  SVG.appendChild(rect);
};

export const delete_elem = (id: number) => document.getElementById(`poly-${id}`)!.remove();

const deleteAllChildren = (node: HTMLElement) => {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
};

wasm.then(engine => {
  engine.init();

  let frame = 0;
  let genDelayMs: number = 1000.0 / 20.0;
  let genIntervalHandle: number | undefined = undefined;

  const genAllChains = () => {
    for (let i = 0; i < 3; i++) {
      engine.generate(i);
    }
  };

  const settings = [
    { type: 'range', label: 'prng_seed', min: 0, max: 1, steps: 1000, initial: 0.5 },
    { type: 'range', label: 'canvas_width', min: 100, max: 2000, initial: 1400 },
    { type: 'range', label: 'canvas_height', min: 100, max: 1600, initial: 800 },
    { type: 'range', label: 'triangle_size', min: 1.0, max: 50.0, step: 0.5, initial: 10.0 },
    // TODO: handle these client side
    {
      type: 'color',
      label: 'chain_1_triangle_border_color',
      initial: '#E20CA3',
      format: 'rgb',
    },
    { type: 'color', label: 'chain_1_triangle_color', initial: 'rgb(81, 12, 84)', format: 'rgb' },
    {
      type: 'color',
      label: 'chain_2_triangle_border_color',
      initial: 'rgb(15, 190, 230)',
      format: 'rgb',
    },
    { type: 'color', label: 'chain_2_triangle_color', initial: 'rgb(9, 89, 135)', format: 'rgb' },
    {
      type: 'color',
      label: 'chain_3_triangle_border_color',
      initial: 'rgb(36, 189, 6)',
      format: 'rgb',
    },
    { type: 'color', label: 'chain_3_triangle_color', initial: 'rgb(9, 112, 5)', format: 'rgb' },
    { type: 'color', label: 'background_color', initial: '#080808', format: 'hex' },
    { type: 'range', label: 'rotation_offset', min: -180, max: 180, initial: 60, steps: 250 },
    {
      type: 'range',
      label: 'triangle_count',
      initial: 50,
      min: 3,
      max: 10000,
      steps: 250,
      scale: 'log',
    },
    { type: 'range', label: 'max_rotation_rads', initial: 0.5, min: 0.0, max: Math.PI },
    { type: 'checkbox', label: 'debug_bounding_boxes', initial: false },
    { type: 'range', label: 'generation_rate', min: 0, max: 180, steps: 60, initial: 20 },
    {
      type: 'button',
      label: 'start_generating',
      action: () => {
        genIntervalHandle = setInterval(genAllChains, genDelayMs);
      },
    },
    {
      type: 'button',
      label: 'stop_generating',
      action: () => {
        clearInterval(genIntervalHandle);
        genIntervalHandle = undefined;
      },
    },
  ];

  const App = () => (
    <Panel
      position="top-right"
      title="Sketch Config"
      settings={settings}
      onChange={(_key, _val, state) => {
        SVG.setAttribute('height', state.canvas_height);
        SVG.setAttribute('width', state.canvas_width);
        SVG.setAttribute('style', `background-color: ${state.background_color};`);
        const newGenDelayMs = 1000.0 / state.generation_rate;
        if (genDelayMs != newGenDelayMs) {
          genDelayMs = newGenDelayMs;
          clearInterval(genIntervalHandle);
          setInterval(genAllChains, genDelayMs);
          return;
        }
        deleteAllChildren(SVG);

        for (let i = 0; i < 3; i++) {
          engine.render(
            JSON.stringify({
              ...state,
              triangle_count: Math.round(state.triangle_count),
              triangle_color: state[`chain_${i + 1}_triangle_color`],
              triangle_border_color: state[`chain_${i + 1}_triangle_border_color`],
            }),
            i
          );
        }
      }}
      width={500}
    />
  );

  const root = document.getElementById('root');
  ReactDOM.render(<App />, root);
});
