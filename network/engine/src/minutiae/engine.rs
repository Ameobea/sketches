use minutiae::{
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    entity,
    prelude::*,
    universe::Universe2D,
};
use uuid::Uuid;

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
    let source_entity_index: usize = self_action.source_entity_index;
    let source_uuid: Uuid = self_action.source_uuid;
    // TODO: Dedup
    let self_action = match &self_action.action {
        Action::SelfAction(self_action) => self_action,
        _ => unreachable!(),
    };

    let (target_entity, _) = match universe
        .entities
        .get_verify_mut(source_entity_index, source_uuid)
    {
        Some(entity) => entity,
        None => return,
    };
    match self_action {
        SelfAction::Custom(self_action) => match self_action {
            NetworkEntityAction::IncCharge(inc) => match target_entity.state {
                NetworkEntityState::Neuron { ref mut charge, .. } => *charge += inc,
            },
            NetworkEntityAction::SetCharge(new_charge) => match target_entity.state {
                NetworkEntityState::Neuron { ref mut charge, .. } => *charge = *new_charge,
            },
        },
        _ => unreachable!(),
    }
}

fn exec_entity_action(entity_action: &NetworkOwnedAction, universe: &mut NetworkUniverse) {
    let (entity_action, target_entity_index, target_uuid) = match &entity_action.action {
        Action::EntityAction {
            action,
            target_entity_index,
            target_uuid,
        } => (action, target_entity_index, target_uuid),
        _ => unreachable!(),
    };

    let (target_entity, _) = match universe
        .entities
        .get_verify_mut(*target_entity_index, *target_uuid)
    {
        Some(entity) => entity,
        None => return,
    };
    match entity_action {
        NetworkEntityAction::IncCharge(inc) => match target_entity.state {
            NetworkEntityState::Neuron { ref mut charge, .. } => *charge += inc,
        },
        NetworkEntityAction::SetCharge(new_charge) => match target_entity.state {
            NetworkEntityState::Neuron { ref mut charge, .. } => *charge = *new_charge,
        },
    }
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
        _cell_action_executor: &mut dyn std::ops::FnMut(NetworkCellAction, usize),
        self_action_executor: &mut dyn FnMut(
            SelfAction<NetworkCellState, NetworkEntityState, NetworkEntityAction>,
        ),
        entity_action_executor: &mut dyn std::ops::FnMut(NetworkEntityAction, usize, uuid::Uuid),
    ) {
        // If our charge has exceeded our activation threshold, emit a pulse to all neighboring
        // entities after taking resistances into account
        match &entity.state {
            NetworkEntityState::Neuron {
                charge,
                activation_threshold,
                resistances,
            } => {
                if charge < activation_threshold {
                    return;
                }

                let (src_x, src_y) =
                    util::get_coords(source_universe_index, UNIVERSE_SIZE as usize);

                if src_y > 0 {
                    if let Some((entity_ix, entity_above)) = universe
                        .entities
                        .get_entities_at(util::get_index(src_x, src_y - 1, UNIVERSE_SIZE as usize))
                        .get(0)
                        .map(|entity_ix| (*entity_ix, unsafe { universe.entities.get(*entity_ix) }))
                    {
                        entity_action_executor(
                            NetworkEntityAction::IncCharge(charge * resistances.up),
                            entity_ix,
                            entity_above.uuid,
                        );
                    }
                }

                if src_y < (UNIVERSE_SIZE as usize - 1) {
                    if let Some((entity_ix, entity_below)) = universe
                        .entities
                        .get_entities_at(util::get_index(src_x, src_y + 1, UNIVERSE_SIZE as usize))
                        .get(0)
                        .map(|entity_ix| (*entity_ix, unsafe { universe.entities.get(*entity_ix) }))
                    {
                        entity_action_executor(
                            NetworkEntityAction::IncCharge(charge * resistances.down),
                            entity_ix,
                            entity_below.uuid,
                        );
                    }
                }

                if src_x > 0 {
                    if let Some((entity_ix, entity_left)) = universe
                        .entities
                        .get_entities_at(util::get_index(src_x - 1, src_y, UNIVERSE_SIZE as usize))
                        .get(0)
                        .map(|entity_ix| (*entity_ix, unsafe { universe.entities.get(*entity_ix) }))
                    {
                        entity_action_executor(
                            NetworkEntityAction::IncCharge(charge * resistances.left),
                            entity_ix,
                            entity_left.uuid,
                        );
                    }
                }

                if src_x < (UNIVERSE_SIZE as usize - 1) {
                    if let Some((entity_ix, entity_right)) = universe
                        .entities
                        .get_entities_at(util::get_index(src_x + 1, src_y, UNIVERSE_SIZE as usize))
                        .get(0)
                        .map(|entity_ix| (*entity_ix, unsafe { universe.entities.get(*entity_ix) }))
                    {
                        entity_action_executor(
                            NetworkEntityAction::IncCharge(charge * resistances.up),
                            entity_ix,
                            entity_right.uuid,
                        );
                    }
                }

                // Reset own charge to 0
                self_action_executor(SelfAction::Custom(NetworkEntityAction::SetCharge(0.)));
            }
        }
    }
}
