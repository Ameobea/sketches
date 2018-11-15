use minutiae::engine::iterator::SerialEntityIterator;
use minutiae::engine::serial::SerialEngine;
use minutiae::universe::Universe2D;

use super::*;

pub struct AntEngine;

impl
    SerialEngine<
        AntCellState,
        AntEntityState,
        AntMutEntityState,
        AntCellAction,
        AntEntityAction,
        SerialEntityIterator<AntCellState, AntEntityState>,
        usize,
        Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
    > for AntEngine
{
    fn iter_entities(
        &self,
        universe: &Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
    ) -> SerialEntityIterator<AntCellState, AntEntityState> {
        SerialEntityIterator::new(universe.entities.len())
    }

    fn exec_actions(
        &self,
        universe: &mut Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        cell_actions: &[AntOwnedAction],
        self_actions: &[AntOwnedAction],
        entity_actions: &[AntOwnedAction],
    ) {
        unimplemented!();
    }

    fn drive_entity(
        &mut self,
        entity_index: usize,
        entity: &Entity<AntCellState, AntEntityState, AntMutEntityState>,
        universe: &Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        cell_action_executor: &mut dyn std::ops::FnMut(AntCellAction, usize),
        self_action_executor: &mut dyn FnMut(
            SelfAction<
                AntCellState,
                AntEntityState,
                AntEntityAction,
            >,
        ),
        entity_action_executor: &mut dyn std::ops::FnMut(AntEntityAction, usize, uuid::Uuid),
    ) {
        unimplemented!();
    }
}
