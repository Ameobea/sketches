import * as React from 'react';
import * as R from 'ramda';
import { style } from 'typestyle';

import { pause, resume } from '..';
import { getIndex, CELL_COUNT } from '../util';

const blankWith = (toSet: number[][]) => {
  const state = new Uint8Array(CELL_COUNT);
  toSet.map(([x, y]) => getIndex(x, y)).forEach(i => (state[i] = 1));
  return state;
};

const offset = (xOffset: number, yOffset: number): ((coord: number[]) => number[]) => ([x, y]) => [
  x + xOffset,
  y + yOffset,
];

const geneticLawsInitalState = () => {
  const birth = [[3, 3], [3, 4], [4, 4]];
  const deathOverpop = [[1, 0], [0, 1], [1, 1], [2, 1], [1, 2]].map(offset(30, 30));
  const deathUnderpop = [[80, 10]];
  const survival = [[0, 0], [0, 1], [1, 0], [1, 1]].map(offset(80, 20));

  return blankWith([...birth, ...deathOverpop, ...deathUnderpop, ...survival]);
};

const getGrowWithoutLimitInitalState = () => {
  const toSet = [
    [1, 6],
    [3, 6],
    [3, 5],
    [5, 2],
    [5, 3],
    [5, 4],
    [7, 1],
    [7, 2],
    [7, 3],
    [8, 2],
  ].map(offset(60, 60));
  return blankWith(toSet);
};

const getEmergent3a = () => {
  const toSet = [[-1, 1], [1, 1], [2, 1], [2, 0], [3, 0], [4, 1], [5, 2]].map(offset(60, 60));
  return blankWith(toSet);
};

const getEmergent3b = () => {
  const toSet = [[0, 1], [1, 1], [2, 1], [2, 0], [3, 0], [4, 0]].map(offset(60, 60));
  return blankWith(toSet);
};

const getEmergent3c = () => {
  const toSet = [[0, 0], [0, 1], [1, 1], [2, 1], [2, 0], [3, 0], [4, 0], [5, 0]].map(
    offset(60, 60)
  );
  return blankWith(toSet);
};

const styles = {
  buttonRow: style({ display: 'flex', flexDirection: 'row', flex: 1 }),
};

const ButtonRow = ({ children }) => <div className={styles.buttonRow}>{children}</div>;

const Writeup = () => (
  <div>
    <h2>Genetic Law Verification</h2>
    <p>
      The four variations of genetic laws (birth, death by overpopulation, death by underpopulation,
      and survival) are all demonstrated in the first step after the initial state.
    </p>
    <h2>Emergent Properties I</h2>
    <p>
      A fully random board is initialized, demonstrating that everything eventually finds a state of
      stability given enough time.
    </p>
    <h2>Emergent Properties II</h2>
    <p>
      I used a 10-cell infinite growth setup found by Paul Callahan that deposits squares as it
      moves. If the canvas were infinite, it would grow without bound.
    </p>
    <h2>Emergent Properties III</h2>
    <p>
      Part A demonstrates a small example of a starting condition that completely disappears after a
      while. It's not a super long version (I'm sure ones that go for longer before completely
      annihilating exist).
    </p>
    <p>
      Part B is called <a href="http://conwaylife.com/wiki/Lumps_of_muck">Lumps of Muck</a> which
      shows a process that takes 65 steps until it reaches a stable end state with 4 squares.
    </p>
    <p>
      Part C is a modified version of Lumps of Muck that produces 5 blinkers and 6 squares,
      demonstrating a 2-step oscillation that repeats forever.
    </p>
    <p><i>Created by Casey Primozic.  Source code is on Github: <a href="https://github.com/Ameobea/sketches/tree/master/game_of_life">https://github.com/Ameobea/sketches/tree/master/game_of_life</a></i></p>
  </div>
);

export default ({ engine }: { engine: typeof import('../engine') }) => (
  <React.Fragment>
    <ButtonRow>
      <button onClick={pause}>Pause</button>
      <button onClick={resume}>Resume</button>
      <button onClick={engine.tick}>Step</button>
      <button onClick={R.partial(engine.set_state, [new Uint8Array(CELL_COUNT)])}>Clear</button>
    </ButtonRow>
    <ButtonRow>
      <button onClick={() => engine.set_state(geneticLawsInitalState())}>
        Verify Genetic Laws
      </button>
    </ButtonRow>
    <ButtonRow>
      <button onClick={engine.set_random_state}>Demonstrate Emergent Properties I</button>
      <button onClick={() => engine.set_state(getGrowWithoutLimitInitalState())}>
        Demonstrate Emergent Properties II
      </button>
    </ButtonRow>
    <ButtonRow>
      <button onClick={() => engine.set_state(getEmergent3a())}>
        Demonstrate Emergent Properties III (a)
      </button>
      <button onClick={() => engine.set_state(getEmergent3b())}>
        Demonstrate Emergent Properties III (b)
      </button>
      <button onClick={() => engine.set_state(getEmergent3c())}>
        Demonstrate Emergent Properties III (c)
      </button>
    </ButtonRow>
    <Writeup />
  </React.Fragment>
);
