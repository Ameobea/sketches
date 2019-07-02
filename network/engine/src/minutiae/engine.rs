use minutiae::{
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    prelude::*,
    universe::Universe2D,
};

use super::*;

pub type NetworkUniverse = Universe2D<NetworkCellState, NetworkEntityState, NetworkMutEntityState>;

pub type OurEngineType = dyn SerialEngine<
    NetworkCellState,
    NetworkEntityState,
    NetworkMutEntityState,
    NetworkCellAction,
    NetworkEntityAction,
    SerialEntityIterator<NetworkCellState, NetworkEntityState>,
    NetworkUniverse,
>;

pub struct NetworkEngine;

fn exec_cell_action(cell_action: &NetworkOwnedAction, universe: &mut NetworkUniverse) {
    unimplemented!();
}

fn exec_self_action(self_action: &NetworkOwnedAction, universe: &mut NetworkUniverse) {
    unimplemented!();
}

fn exec_entity_action(entity_action: &NetworkOwnedAction, universe: &mut NetworkUniverse) {
    unimplemented!();
}

impl
    SerialEngine<
        NetworkCellState,
        NetworkEntityState,
        NetworkMutEntityState,
        NetworkCellAction,
        NetworkEntityAction,
        SerialEntityIterator<NetworkCellState, NetworkEntityState>,
        NetworkUniverse,
    > for NetworkEngine
{
    fn iter_entities(
        &self,
        universe: &Universe2D<NetworkCellState, NetworkEntityState, NetworkMutEntityState>,
    ) -> SerialEntityIterator<NetworkCellState, NetworkEntityState> {
        SerialEntityIterator::new(universe.entities.len())
    }

    fn exec_actions(
        &self,
        universe: &mut NetworkUniverse,
        cell_actions: &[NetworkOwnedAction],
        self_actions: &[NetworkOwnedAction],
        entity_actions: &[NetworkOwnedAction],
    ) {
        for cell_action in cell_actions {
            exec_cell_action(cell_action, universe);
        }

        for self_action in self_actions {
            exec_self_action(self_action, universe);
        }

        for entity_action in entity_actions {
            exec_entity_action(entity_action, universe);
        }
    }

    fn drive_entity(
        &mut self,
        source_universe_index: usize,
        entity: &Entity<NetworkCellState, NetworkEntityState, NetworkMutEntityState>,
        universe: &NetworkUniverse,
        cell_action_executor: &mut dyn std::ops::FnMut(NetworkCellAction, usize),
        self_action_executor: &mut dyn FnMut(
            SelfAction<NetworkCellState, NetworkEntityState, NetworkEntityAction>,
        ),
        _entity_action_executor: &mut dyn std::ops::FnMut(NetworkEntityAction, usize, uuid::Uuid),
    ) {
        unimplemented!();
    }
}
