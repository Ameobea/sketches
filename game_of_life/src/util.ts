export const WORLD_WIDTH = 150;
export const WORLD_HEIGHT = 150;
export const CELL_COUNT = WORLD_HEIGHT * WORLD_WIDTH;
export const CANVAS_SCALE_FACTOR = 6;

export const getIndex = (x: number, y: number): number => {
  return y * WORLD_WIDTH + x;
};
