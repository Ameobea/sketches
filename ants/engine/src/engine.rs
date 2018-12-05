use minutiae::{
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    universe::Universe2D,
};

use super::*;

pub struct AntEngine;

fn exec_cell_action(
    action: &AntOwnedAction,
    universe: &mut Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
) {
    match &action.action {
        Action::CellAction {
            universe_index,
            action: self_action,
        } => match self_action {
            AntCellAction::LayPheremone(pheremone_type) => {
                let pheremones = if let AntCellState::Empty(pheremones) =
                    &mut universe.cells[*universe_index].state
                {
                    pheremones
                } else {
                    return;
                };

                match pheremone_type {
                    PheremoneType::Wandering => pheremones.wandering += 1.,
                    PheremoneType::Returning => pheremones.returning += 1.,
                }
            },
        },
        _ => unreachable!(),
    }
}

fn exec_self_action(
    action: &AntOwnedAction,
    universe: &mut Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
) {
    match action.action {
        Action::SelfAction(ref self_action) => {
            let (entity_index, entity_uuid) = (action.source_entity_index, action.source_uuid);
            // this function will return early if the entity has been deleted
            let (entity, universe_index) =
                match universe.entities.get_verify_mut(entity_index, entity_uuid) {
                    Some((entity, universe_index)) => (entity, universe_index),
                    None => {
                        return;
                    }, // entity has been deleted, so do nothing.
                };

            match self_action {
                SelfAction::Translate(x_offset, y_offset) => {
                    let (cur_x, cur_y) = get_coords(universe_index, UNIVERSE_SIZE as usize);
                    let new_x = cur_x as isize + x_offset;
                    let new_y = cur_y as isize + y_offset;

                    // verify that the supplied desination coordinates are in bounds
                    if new_x >= 0
                        && new_x < UNIVERSE_SIZE as isize
                        && new_y >= 0
                        && new_y < UNIVERSE_SIZE as isize
                    {
                        let dst_universe_index =
                            get_index(new_x as usize, new_y as usize, UNIVERSE_SIZE as usize);

                        // Only allow moves onto empty squares
                        match universe.cells[dst_universe_index].state {
                            AntCellState::Empty(_) => (),
                            _ => return,
                        };
                        universe
                            .entities
                            .move_entity(entity_index, dst_universe_index);
                    } else if let AntEntityState::Wandering(ref mut wander_state) = entity.state {
                        // reset the wander state since we've hit a side
                        *wander_state = WanderingState::default();
                    }
                },
                SelfAction::Custom(AntEntityAction::UpdateWanderState) =>
                    if let AntEntityState::Wandering(ref mut wander_state) = entity.state {
                        *wander_state = wander_state.next()
                    },
                _ => unimplemented!(),
            }
        },
        _ => unreachable!(),
    }
}

fn exec_entity_action(
    action: &AntOwnedAction,
    _universe: &mut Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
) {
    match action.action {
        Action::EntityAction {
            action: ref entity_action,
            target_entity_index: _,
            target_uuid: _,
        } => match entity_action {
            _ => unimplemented!(),
        },
        _ => unreachable!(),
    }
}

impl
    SerialEngine<
        AntCellState,
        AntEntityState,
        AntMutEntityState,
        AntCellAction,
        AntEntityAction,
        SerialEntityIterator<AntCellState, AntEntityState>,
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
        entity: &Entity<AntCellState, AntEntityState, AntMutEntityState>,
        universe: &Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        cell_action_executor: &mut dyn std::ops::FnMut(AntCellAction, usize),
        self_action_executor: &mut dyn FnMut(
            SelfAction<AntCellState, AntEntityState, AntEntityAction>,
        ),
        entity_action_executor: &mut dyn std::ops::FnMut(AntEntityAction, usize, uuid::Uuid),
    ) {
        match &entity.state {
            AntEntityState::Wandering(WanderingState { x_dir, y_dir }) => {
                self_action_executor(SelfAction::Translate(
                    x_dir.get_coord_offset(),
                    y_dir.get_coord_offset(),
                ));
                self_action_executor(SelfAction::Custom(AntEntityAction::UpdateWanderState));
                cell_action_executor(
                    AntCellAction::LayPheremone(PheremoneType::Wandering),
                    source_universe_index,
                );
            },
            AntEntityState::ReturningToNestWithFood => {
                cell_action_executor(
                    AntCellAction::LayPheremone(PheremoneType::Returning),
                    source_universe_index,
                );
            },
            _ => unimplemented!(),
        }
    }
}
