const wasm = import('./engine');

const SVG: SVGAElement = (document.getElementById('svg') as any) as SVGAElement;

export const render_triangle = (
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  x3: number,
  y3: number,
  color: string
) => {
  const poly: SVGPolygonElement = (document.createElementNS(
    'http://www.w3.org/2000/svg',
    'polygon'
  ) as any) as SVGPolygonElement;
  poly.setAttribute('points', `${x1},${y1} ${x2},${y2} ${x3},${y3}`);
  poly.setAttribute('style', `fill:${color};stroke:purple;stroke-width:1`);
  SVG.appendChild(poly);
};

wasm.then(engine => {
  engine.init();
  engine.render();
});
