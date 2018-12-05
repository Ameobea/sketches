import React from 'react';
import ControlPanel from 'react-control-panel';

import { pause, resume, setCanvasScaleFactor } from '../';

const baseSettings = [
  { label: 'direction_change_chance', type: 'range', min: 1, max: 100, step: 0.25, initial: 4.5 },
  {
    label: 'simulation_tick_delay',
    type: 'range',
    min: 0.1,
    max: 1000,
    initial: 10,
    steps: 250,
    scale: 'log',
  },
  { label: 'scale_factor', type: 'range', min: 3.0, max: 20.0, step: 0.1, initial: 3.0 },
];

const internalSettingHandlers = {
  simulation_tick_delay: delay => {
    pause();
    resume(parseFloat(delay));
  },
  scale_factor: scaleFactor => setCanvasScaleFactor(parseFloat(scaleFactor)),
};

const UI = ({ buttons, setConf }) => (
  <ControlPanel
    position="top-right"
    title="Simulation Controls"
    settings={[...buttons, ...baseSettings]}
    onChange={(key, val, state) => {
      const internalHandler = internalSettingHandlers[key];
      if (internalHandler) {
        internalHandler(val);
      } else {
        setConf(JSON.stringify(state));
      }
    }}
    width={600}
  />
);

export default UI;
