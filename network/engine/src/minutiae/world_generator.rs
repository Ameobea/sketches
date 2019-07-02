use minutiae::{prelude::*, universe::Universe2DConf, util::get_index};

use super::*;

pub struct NetworkUniverseGenerator;

impl Generator<NetworkCellState, NetworkEntityState, NetworkMutEntityState>
    for NetworkUniverseGenerator
{
    fn gen(
        &mut self,
        _conf: &Universe2DConf,
    ) -> (
        Vec<Cell<NetworkCellState>>,
        Vec<Vec<Entity<NetworkCellState, NetworkEntityState, NetworkMutEntityState>>>,
    ) {
        unimplemented!();
    }
}
