import React from 'react';
import ControlPanel from 'react-control-panel';

const settings = [{ label: 'idk', type: 'range', min: 0, max: 100, steps: 100 }];

const UI: React.FC<{ buttons: any[] }> = ({ buttons }) => {
  return (
    <ControlPanel
      position="top-right"
      title="Controls"
      settings={[...buttons, ...settings]}
      onChange={(key, val) => {}}
      width={550}
    />
  );
};

export default UI;
