import React, { useEffect, useState, Fragment } from 'react';
import ControlPanel from 'react-control-panel';

import { pause, resume, setCanvasScaleFactor } from '../';

const baseSimulationControlSettings = [
  {
    label: 'simulation_tick_delay',
    type: 'range',
    min: 0.1,
    max: 1000,
    steps: 250,
    scale: 'log',
  },
  { label: 'scale_factor', type: 'range', min: 3.0, max: 20.0, step: 0.1 },
  {
    label: 'pheremone_decay_interval',
    type: 'range',
    min: 10,
    max: 2500,
    stepSize: 25,
  },
  { label: 'pheremone_decay_multiplier', type: 'range', min: 0.0, max: 1.0 },
];

const internalSettingHandlers = {
  simulation_tick_delay: delay => {
    pause();
    resume(parseFloat(delay));
  },
  scale_factor: scaleFactor => setCanvasScaleFactor(parseFloat(scaleFactor)),
};

const SimulationControls = ({ buttons, onChange, state }) => (
  <ControlPanel
    position="top-right"
    title="Simulation Controls"
    settings={[...buttons, ...baseSimulationControlSettings]}
    onChange={(key, val, state) => {
      const internalHandler = internalSettingHandlers[key];
      if (internalHandler) {
        internalHandler(val);
      }

      onChange(state);
    }}
    state={state}
    width={550}
  />
);

const worldGenSettings = [
  { label: 'food_patch_count', type: 'range', min: 0, max: 500, steps: 250 },
  { label: 'food_patch_size', type: 'range', min: 0, max: 100, steps: 100 },
  { label: 'food_patch_size_variance', type: 'range', min: 0, max: 100, steps: 100 },
  { label: 'food_patch_capacity', type: 'range', min: 1, max: 5000, scale: 'log' },
  { label: 'barrier_patch_count', type: 'range', min: 0, max: 100, steps: 100 },
  { label: 'barrier_patch_size', type: 'range', min: 0, max: 500, steps: 100 },
];

const WorldGenerationSettings = ({ onChange, state }) => (
  <ControlPanel
    position="top-right"
    style={{ top: 240 }}
    title="World Generation"
    settings={worldGenSettings}
    onChange={(_key, _val, state) => onChange(state)}
    state={state}
    width={550}
  />
);

const antBehaviorSettings = [
  {
    label: 'wander_transition_chance_percent',
    type: 'range',
    min: 0.25,
    max: 100.0,
    steps: 400,
    scale: 'log',
  },
];

const AntBehaviorSettings = ({ onChange, state }) => (
  <ControlPanel
    position="top-right"
    style={{ top: 423 }}
    title="Ant Behavior"
    settings={antBehaviorSettings}
    onChange={(_key, _val, state) => onChange(state)}
    state={state}
    width={550}
  />
);

export const getInitialConf = (loadDefaults: boolean = false) => {
  const storedConf = localStorage.getItem('conf');
  if (!loadDefaults && storedConf) {
    return JSON.parse(storedConf);
  } else {
    return {
      // simulation controls
      simulation_tick_delay: 10,
      scale_factor: 3.0,
      pheremone_decay_interval: 250,
      pheremone_decay_multiplier: 0.8,
      // worldgen
      food_patch_count: 200,
      food_patch_size: 28,
      food_patch_size_variance: 3,
      food_patch_capacity: 5,
      barrier_patch_count: 44,
      barrier_patch_size: 128,
      // ant behavior
      wander_transition_chance_percent: 4.25,
    };
  }
};

const UI = ({ buttons, applyConf }) => {
  const [mergedConf, setMergedConf] = useState(getInitialConf());

  const handleChange = state => {
    const newMergedConf = { ...mergedConf, ...state };
    setMergedConf(newMergedConf);
    applyConf(JSON.stringify(newMergedConf));
  };

  return (
    <Fragment>
      <SimulationControls
        buttons={[
          ...buttons,
          {
            type: 'button',
            label: 'default settings',
            action: () => {
              const defaultConf = getInitialConf(true);
              setMergedConf(defaultConf);
              applyConf(JSON.stringify(defaultConf));
            },
          },
        ]}
        onChange={handleChange}
        state={mergedConf}
      />
      <WorldGenerationSettings onChange={handleChange} state={mergedConf} />
      <AntBehaviorSettings onChange={handleChange} state={mergedConf} />
    </Fragment>
  );
};

export default UI;
