import React from 'react';
import ReactDOM from 'react-dom';

import UI from './ui';
import { pause, resume, setGraphicsDisplay, getDisplayGraphics } from './loop';
import { initWebGL } from './webgl';

const wasm = import('./engine');

wasm.then(engine => {
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
      action: resume,
    },
    {
      type: 'button',
      label: 'toggle graphics',
      action: () => setGraphicsDisplay(!getDisplayGraphics()),
    },
  ];

  const root = document.getElementById('react-root')!;
  ReactDOM.render(<UI buttons={buttons} />, root);
});
