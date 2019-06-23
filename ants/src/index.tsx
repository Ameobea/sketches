import React from 'react';
import ReactDOM from 'react-dom';

import { createBackgroundTexture, initWebGL, render } from './webgl';
import { getCanvas } from './canvas';
import UI, { getInitialConf } from './ui';
import {
  setCountFood,
  getIntervalHandle,
  pause,
  resume,
  doTick,
  setGraphicsDisplay,
  getDisplayGraphics,
} from './loop';

const wasm = import('./engine');

wasm
  .then(engine => {
    initWebGL();
    engine.set_user_conf(JSON.stringify(getInitialConf(false)));
    engine.init();
    setCountFood(engine.count_collected_food);

    const applyConf = (confJson: string) => {
      engine.set_user_conf(confJson);

      // show universe gen in realtime if we're paused.
      if (!getIntervalHandle()) {
        engine.init_universe();
        pause();
        doTick();
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
        action: () => setGraphicsDisplay(!getDisplayGraphics()),
      },
    ];

    const root = document.getElementById('root');
    ReactDOM.render(<UI buttons={buttons} applyConf={applyConf} />, root);
  })
  .catch(console.error);
