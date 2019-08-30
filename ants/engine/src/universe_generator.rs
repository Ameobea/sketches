use minutiae::{prelude::*, universe::Universe2DConf, util::get_index};
use rand::Rng;

use super::*;

pub struct AntUniverseGenerator(pub &'static UserConf);

fn get_start_coords() -> (usize, usize) {
    let x: usize = rng().gen_range(0, UNIVERSE_SIZE as usize);
    let y: usize = rng().gen_range(0, UNIVERSE_SIZE as usize);
    (x, y)
}

fn is_in_bounds(x: usize, y: usize, x_offset: isize, y_offset: isize) -> Option<(usize, usize)> {
    let (new_x, new_y) = (x as isize + x_offset, y as isize + y_offset);
    if new_x > 0 && new_y > 0 && new_x < UNIVERSE_SIZE as isize && new_y < UNIVERSE_SIZE as isize {
        Some((new_x as usize, new_y as usize))
    } else {
        None
    }
}

const MAX_PLACEMENT_ITERS: usize = 10_000;

/// `wander_change_chance` should be from 0 to 100 (percentage chance of changing) inclusive.
fn gen_terrain(
    cells: &mut [Cell<AntCellState>],
    count: usize,
    size: usize,
    size_variance: usize,
    wander_change_chance: f32,
    state: AntCellState,
) {
    debug_assert!(wander_change_chance >= 0.0 && wander_change_chance <= 100.0);
    let variance: isize = if size_variance == 0 {
        0isize
    } else {
        rng().gen_range(-(size_variance as isize), size_variance as isize)
    };
    let real_size = ((size as isize) + variance).max(0) as usize;

    for _ in 0..count {
        let mut wander_state = WanderingState::default();
        let (mut x, mut y) = get_start_coords();
        let mut placed_tiles: Vec<(usize, usize)> = Vec::with_capacity(size);

        let mut iters = 0;
        while placed_tiles.len() < real_size {
            // Avoid infinite loops of being unable to place
            iters += 1;
            if iters > MAX_PLACEMENT_ITERS {
                warn!("Bailed out generating cells with state: {:?}", state);
                break;
            }

            wander_state = wander_state.next(wander_change_chance);

            // wander on to the next square
            let (x_offset, y_offset) = (
                wander_state.x_dir.get_coord_offset(),
                wander_state.y_dir.get_coord_offset(),
            );

            match is_in_bounds(x, y, x_offset, y_offset) {
                // If we hit a side of the universe, reset to a random previously placed square
                None => {
                    let (new_x, new_y) = rng()
                        .choose(&placed_tiles)
                        .map(|tup| tup.clone())
                        .unwrap_or_else(get_start_coords);
                    x = new_x;
                    y = new_y;
                    continue;
                },
                // set the new coords
                Some((new_x, new_y)) => {
                    x = new_x;
                    y = new_y;
                },
            }

            // don't re-place on already placed tiles.
            if placed_tiles.iter().find(|tup| **tup == (x, y)).is_some() {
                continue;
            }

            // make a placement and record it
            let universe_ix = get_index(x, y, UNIVERSE_SIZE as usize);
            cells[universe_ix].state = state;
            placed_tiles.push((x, y));
        }
    }
}

impl Generator<AntCellState, AntEntityState, AntMutEntityState> for AntUniverseGenerator {
    fn gen(
        &mut self,
        _conf: &Universe2DConf,
    ) -> (
        Vec<Cell<AntCellState>>,
        Vec<Vec<Entity<AntCellState, AntEntityState, AntMutEntityState>>>,
    ) {
        let mut entities = vec![Vec::new(); (UNIVERSE_SIZE * UNIVERSE_SIZE) as usize];
        let (anthill_x, anthill_y) = get_start_coords();
        let anthill_universe_coord = get_index(anthill_x, anthill_y, UNIVERSE_SIZE as usize);
        for _ in 0..active_conf().ant_count as usize {
            let entity = Entity::new(
                AntEntityState::Wandering(WanderingState::default()),
                AntMutEntityState {
                    nest_x: anthill_x,
                    nest_y: anthill_y,
                },
            );
            entities[anthill_universe_coord].push(entity);
        }

        let mut cells = vec![
            Cell {
                state: AntCellState::Empty(Pheremones::default())
            };
            (UNIVERSE_SIZE * UNIVERSE_SIZE) as usize
        ];

        gen_terrain(
            &mut cells,
            self.0.barrier_patch_count,
            self.0.barrier_patch_size,
            0,
            26.3,
            AntCellState::Barrier,
        );
        gen_terrain(
            &mut cells,
            self.0.food_patch_count,
            self.0.food_patch_size,
            self.0.food_patch_size_variance,
            94.632,
            AntCellState::Food(self.0.food_patch_capacity),
        );

        cells[anthill_universe_coord].state = AntCellState::Anthill;
        // clear around the anthill
        iter_visible(anthill_x, anthill_y, 2, UNIVERSE_SIZE as usize).for_each(|(x, y)| {
            if (x, y) == (anthill_x, anthill_y) {
                return;
            }

            let universe_ix = get_index(x, y, UNIVERSE_SIZE as usize);
            cells[universe_ix].state = AntCellState::Empty(Pheremones::default());
        });

        (cells, entities)
    }
}
