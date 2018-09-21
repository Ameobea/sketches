import * as React from 'react';
import * as R from 'ramda';

import { pause, resume } from '..';
import { getIndex, CELL_COUNT } from '../util';

const canvasState = (() => {
  const state = new Uint8Array(CELL_COUNT);
  const toSet = [[100, 100], [101, 100], [101, 100], [101, 101]];
  toSet.map(([x, y]) => getIndex(x, y)).forEach(i => (state[i] = 1));
  return state;
})();

export default ({ engine }: { engine: typeof import('../engine') }) => (
  <div style={{ display: 'flex', flexDirection: 'row' }}>
    <button onClick={pause}>Pause</button>
    <button onClick={resume}>Resume</button>
    <button onClick={engine.tick}>Step</button>
    <button onClick={R.partial(engine.set_state, [new Uint8Array(CELL_COUNT)])}>Clear</button>
    <button onClick={() => engine.set_state(canvasState)}>Set Pattern</button>
  </div>
);
