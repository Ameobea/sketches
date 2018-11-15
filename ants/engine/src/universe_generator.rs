use minutiae::prelude::*;
use minutiae::universe::Universe2DConf;
use rand::Rng;

use super::*;

pub struct AntUniverseGenerator;

const INITIAL_ENTITY_COUNT: usize = 20;

impl Generator<AntCellState, AntEntityState, AntMutEntityState> for AntUniverseGenerator {
    fn gen(
        &mut self,
        conf: &Universe2DConf,
    ) -> (
        Vec<Cell<AntCellState>>,
        Vec<Vec<Entity<AntCellState, AntEntityState, AntMutEntityState>>>,
    ) {
        let cells = vec![
            Cell {
                state: AntCellState::Empty(Pheremones::default())
            };
            (UNIVERSE_SIZE * UNIVERSE_SIZE) as usize
        ];
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
