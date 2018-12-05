use minutiae::{prelude::*, universe::Universe2DConf};
use rand::Rng;

use super::*;

pub struct AntUniverseGenerator(pub &'static UserConf);

const INITIAL_ENTITY_COUNT: usize = 20;

fn gen_barriers(cells: &mut [Cell<AntCellState>], count: usize, size: usize) {
    // unimplemented!(); // TODO
}

fn gen_food(cells: &mut [Cell<AntCellState>], count: usize, size: usize, variance: usize) {
    // unimplemented!(); // TODO
}

impl Generator<AntCellState, AntEntityState, AntMutEntityState> for AntUniverseGenerator {
    fn gen(
        &mut self,
        _conf: &Universe2DConf,
    ) -> (
        Vec<Cell<AntCellState>>,
        Vec<Vec<Entity<AntCellState, AntEntityState, AntMutEntityState>>>,
    ) {
        let mut cells = vec![
            Cell {
                state: AntCellState::Empty(Pheremones::default())
            };
            (UNIVERSE_SIZE * UNIVERSE_SIZE) as usize
        ];
        gen_barriers(
            &mut cells,
            self.0.barrier_patch_count,
            self.0.barrier_patch_size,
        );
        gen_food(
            &mut cells,
            self.0.food_patch_count,
            self.0.food_patch_size,
            self.0.food_patch_size_variance,
        );

        let mut entities = vec![Vec::new(); (UNIVERSE_SIZE * UNIVERSE_SIZE) as usize];
        for _ in 0..INITIAL_ENTITY_COUNT {
            let x: usize = rng().gen_range(0, UNIVERSE_SIZE as usize);
            let y: usize = rng().gen_range(0, UNIVERSE_SIZE as usize);
            let entity = Entity::new(
                AntEntityState::Wandering(WanderingState::default()),
                AntMutEntityState {},
            );
            entities[(y * UNIVERSE_SIZE as usize) + x].push(entity);
        }

        (cells, entities)
    }
}
