import React from 'react';
import ReactDOM from 'react-dom';
import Panel from 'react-control-panel';

const wasm = import('./engine');

const SVG: HTMLElement = document.getElementById('svg') as any;

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
  const poly = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
  poly.setAttribute('points', `${x1},${y1} ${x2},${y2} ${x3},${y3}`);
  poly.setAttribute('style', `fill:${color};stroke:${border_color};stroke-width:1`);
  SVG.appendChild(poly);
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

const deleteAllChildren = (node: HTMLElement) => {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
};

wasm.then(engine => {
  engine.init();

  const App = () => (
    <Panel
      position="top-right"
      title="Sketch Config"
      settings={settings}
      onChange={(_key, _val, state) => {
        SVG.setAttribute('height', state.canvas_height);
        SVG.setAttribute('width', state.canvas_width);
        deleteAllChildren(SVG);

        engine.render(
          JSON.stringify({ ...state, triangle_count: Math.round(state.triangle_count) })
        );
      }}
      width={500}
    />
  );

  const root = document.getElementById('root');
  ReactDOM.render(<App />, root);
});

const settings = [
  { type: 'range', label: 'prng_seed', min: 0, max: 1, steps: 1000, initial: 0.5 },
  { type: 'range', label: 'canvas_width', min: 100, max: 1000, initial: 800 },
  { type: 'range', label: 'canvas_height', min: 100, max: 1000, initial: 800 },
  { type: 'range', label: 'triangle_size', min: 1.0, max: 50.0, step: 0.5, initial: 10.0 },
  { type: 'color', label: 'triangle_border_color', initial: '#7c007c', format: 'rgb' },
  { type: 'color', label: 'triangle_color', initial: 'rgb(81, 12, 84)', format: 'rgb' },
  { type: 'range', label: 'rotation_offset', min: -180, max: 180, initial: 60, steps: 250 },
  {
    type: 'range',
    label: 'triangle_count',
    initial: 200,
    min: 1,
    max: 20000,
    steps: 250,
    scale: 'log',
  },
  { type: 'range', label: 'max_rotation_rads', initial: 0.5, min: 0.0, max: Math.PI },
  { type: 'checkbox', label: 'debug_bounding_boxes', initial: false },
];
