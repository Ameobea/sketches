import React, { useEffect, useState, Fragment } from 'react';
import ControlPanel from 'react-control-panel';

import { pause, resume, setCanvasScaleFactor, setGraphicsDisplay } from '../';

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
  { label: 'pheromone_max', type: 'range', min: 0.0, max: 100.0 },
];

const internalSettingHandlers = {
  simulation_tick_delay: delay => {
    pause();
    resume(parseFloat(delay));
  },
  scale_factor: scaleFactor => setCanvasScaleFactor(parseFloat(scaleFactor)),
  show_graphics: setGraphicsDisplay,
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
  { label: 'ant_count', type: 'range', min: 1, max: 10000, steps: 500, scale: 'log' },
  { label: 'food_patch_count', type: 'range', min: 0, max: 1000, steps: 250 },
  { label: 'food_patch_size', type: 'range', min: 0, max: 750, steps: 250 },
  { label: 'food_patch_size_variance', type: 'range', min: 0, max: 100, steps: 100 },
  { label: 'food_patch_capacity', type: 'range', min: 1, max: 5000, steps: 250 },
  { label: 'barrier_patch_count', type: 'range', min: 0, max: 1000, steps: 100 },
  { label: 'barrier_patch_size', type: 'range', min: 0, max: 500, steps: 100 },
];

const WorldGenerationSettings = ({ onChange, state }) => (
  <ControlPanel
    position="top-right"
    style={{ top: 310 }}
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
  {
    label: 'anthill_attraction_pos_bias',
    type: 'range',
    min: 1.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'anthill_attraction_neg_bias',
    type: 'range',
    min: 0.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'anthill_attraction_distrance_multiplier',
    type: 'range',
    min: 0.0,
    max: 1.0,
    steps: 100,
  },
  {
    label: 'returning_maintain_pos_bias',
    type: 'range',
    min: 0.0,
    max: 50.0,
    steps: 100,
  },
  {
    label: 'returning_maintain_neg_bias',
    type: 'range',
    min: 0.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'returning_wander_threshold',
    type: 'range',
    min: 0.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'following_pos_bias',
    type: 'range',
    min: 0.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'following_neg_bias',
    type: 'range',
    min: 0.0,
    max: 10.0,
    steps: 100,
  },
  {
    label: 'base_wandering_weight',
    type: 'range',
    min: 0.0,
    max: 3.0,
    steps: 100,
  },
  {
    label: 'base_returning_weight',
    type: 'range',
    min: 0.0,
    max: 3.0,
    steps: 100,
  },
  {
    label: 'base_following_weight',
    type: 'range',
    min: 0.0,
    max: 3.0,
    steps: 100,
  },
  { label: 'scout_pursuit_cutoff', type: 'range', min: 0.0, max: 15.0, steps: 120 },
];

const AntBehaviorSettings = ({ onChange, state }) => (
  <ControlPanel
    position="top-right"
    style={{ top: 535 }}
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
      simulation_tick_delay: 0.1,
      scale_factor: 3,
      pheremone_decay_interval: 250,
      pheremone_decay_multiplier: 0.94,
      pheromone_max: 11,
      show_graphics: true,
      ant_count: 308,
      food_patch_count: 248,
      food_patch_size: 27,
      food_patch_size_variance: 3,
      food_patch_capacity: 5,
      barrier_patch_count: 44,
      barrier_patch_size: 128,
      wander_transition_chance_percent: 4.25,
      anthill_attraction_pos_bias: 3.07,
      anthill_attraction_neg_bias: 1,
      anthill_attraction_distrance_multiplier: 0.1,
      returning_maintain_pos_bias: 50,
      returning_maintain_neg_bias: 0.3,
      following_pos_bias: 1.1,
      following_neg_bias: 0.5,
      scout_pursuit_cutoff: 13.25,
      base_wandering_weight: 0.12,
      base_returning_weight: 0.03,
      base_following_weight: 0.12,
      returning_wander_threshold: 1.1,
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
