use std::cmp::Ordering;

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
    let source_entity_id = action.source_uuid;
    let source_entity_index = action.source_entity_index;

    match &action.action {
        Action::CellAction {
            universe_index,
            action: self_action,
        } => {
            let mut cell = &mut universe.cells[*universe_index];
            match self_action {
                AntCellAction::LayPheremone(pheremone_type) => {
                    let pheremones = if let AntCellState::Empty(pheremones) = &mut cell.state {
                        pheremones
                    } else {
                        return;
                    };

                    match pheremone_type {
                        PheremoneType::Wandering => pheremones.wandering += 1.,
                        PheremoneType::Returning => pheremones.returning += 1.,
                    }
                },
                AntCellAction::EatFood => {
                    if let AntCellState::Food(ref mut quantity) = &mut cell.state {
                        *quantity -= 1;

                        // Convert the square to an empty cell if all food is consumed
                        if *quantity == 0 {
                            cell.state = AntCellState::Empty(Pheremones::default());
                        }

                        let entity_opt = universe
                            .entities
                            .get_verify_mut(source_entity_index, source_entity_id);
                        match entity_opt {
                            Some((mut entity, _)) =>
                                entity.state = AntEntityState::ReturningToNestWithFood,
                            None => common::warn(
                                "Attempted to mark entity as returning to nest, but it was \
                                 deleted?",
                            ),
                        }
                    }
                },
            }
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

                    let can_move = {
                        // verify that the supplied desination coordinates are in bounds
                        let is_in_bounds = new_x >= 0
                            && new_x < UNIVERSE_SIZE as isize
                            && new_y >= 0
                            && new_y < UNIVERSE_SIZE as isize;

                        // verify we're not trying to move diagonally through non-traversable blocks
                        is_in_bounds
                            && if *x_offset != 0 && *y_offset != 0 {
                                let (c1, c2) = (
                                    &universe.cells
                                        [get_index(cur_x, new_y as usize, UNIVERSE_SIZE as usize)],
                                    &universe.cells
                                        [get_index(new_x as usize, cur_y, UNIVERSE_SIZE as usize)],
                                );
                                c1.state.is_traversable() && c2.state.is_traversable()
                            } else {
                                true
                            }
                    };

                    if can_move {
                        let dst_universe_index =
                            get_index(new_x as usize, new_y as usize, UNIVERSE_SIZE as usize);

                        // Only allow moves onto empty squares
                        if !universe.cells[dst_universe_index].state.is_traversable() {
                            if let AntEntityState::Wandering(ref mut wander_state) = entity.state {
                                *wander_state = WanderingState::default();
                                return;
                            }
                        }

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
                        *wander_state =
                            wander_state.next(active_conf().wander_transition_chance_percent)
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

fn clamp(val: isize, low: isize, high: isize) -> isize { val.max(low).min(high) }

fn get_visible_cells_iterator(
    cur_x: usize,
    cur_y: usize,
    cells: &[Cell<AntCellState>],
) -> impl Iterator<Item = ((usize, usize), &AntCellState)> {
    iter_visible(cur_x, cur_y, VIEW_DISTANCE, UNIVERSE_SIZE as usize).map(move |(x, y)| {
        let cell_state = &cells[get_index(x, y, UNIVERSE_SIZE as usize)].state;
        ((x, y), cell_state)
    })
}

fn find_closest(
    cur_x: usize,
    cur_y: usize,
    cells: &[Cell<AntCellState>],
    pred: impl Fn(&AntCellState) -> bool,
) -> Option<(usize, usize)> {
    let reduce_closest_food = |acc: Option<((usize, usize), usize)>, ((x, y), cell_state)| {
        if pred(cell_state) {
            let cur_distance = manhattan_distance(cur_x, cur_y, x, y);
            match acc {
                // There's a better target already
                Some(((..), best_distance)) if best_distance < cur_distance => acc,
                // We beat the previous closest
                _ => Some(((x, y), cur_distance)),
            }
        } else {
            acc
        }
    };

    get_visible_cells_iterator(cur_x, cur_y, cells)
        .fold(None, reduce_closest_food)
        .map(|(pos, _)| pos)
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
        let (cur_x, cur_y) = get_coords(source_universe_index, UNIVERSE_SIZE as usize);

        let mut translate = |xdiff, ydiff| {
            self_action_executor(SelfAction::Translate(xdiff, ydiff));
        };

        let mut move_towards = |cur_x: usize, cur_y: usize, dst_x: usize, dst_y: usize| {
            let (xdiff, ydiff) = (
                dst_x as isize - cur_x as isize,
                dst_y as isize - cur_y as isize,
            );
            let (xdiff, ydiff) = (clamp(xdiff, -1, 1), clamp(ydiff, -1, 1));
            translate(xdiff, ydiff);
        };

        match &entity.state {
            AntEntityState::Wandering(WanderingState { x_dir, y_dir }) => {
                // Check if we're currently standing on food
                if let AntCellState::Food(_) = universe.cells[source_universe_index].state {
                    // consume the food and path back towards the nest
                    cell_action_executor(AntCellAction::EatFood, source_universe_index);
                    return;
                }

                let closest_food = find_closest(cur_x, cur_y, &universe.cells, |cell_state| {
                    if let AntCellState::Food(_) = cell_state {
                        true
                    } else {
                        false
                    }
                });

                if let Some((x, y)) = closest_food {
                    // We see food!  Path towards it.
                    move_towards(cur_x, cur_y, x, y);

                    // TODO: Lay pheromones
                    return;
                }

                // No food within sight, no food trails to follow, so continue wandering.
                translate(x_dir.get_coord_offset(), y_dir.get_coord_offset());
                self_action_executor(SelfAction::Custom(AntEntityAction::UpdateWanderState));
                cell_action_executor(
                    AntCellAction::LayPheremone(PheremoneType::Wandering),
                    source_universe_index,
                );
            },
            AntEntityState::ReturningToNestWithFood => {
                // Path back to the anthill by doing the following:
                //  1. Move towards the strongest "looking for food" trail
                //  2. ...wander
                let cmp_candidate_cells = |cs1: &AntCellState, cs2: &AntCellState| -> Ordering {
                    match (cs1, cs2) {
                        (AntCellState::Empty(pher1), AntCellState::Empty(pher2)) => pher1
                            .wandering
                            .partial_cmp(&pher2.wandering)
                            .unwrap_or(Ordering::Equal),
                        (AntCellState::Empty(_), _) => Ordering::Greater,
                        (_, AntCellState::Empty(_)) => Ordering::Less,
                        _ => Ordering::Equal,
                    }
                };

                let best_dst_cell = get_visible_cells_iterator(cur_x, cur_y, &universe.cells)
                    .max_by(|(_, cs1), (_, cs2)| cmp_candidate_cells(cs1, cs2));
                match best_dst_cell {
                    Some(((dst_x, dst_y), _)) => move_towards(cur_x, cur_y, dst_x, dst_y),
                    None => {
                        // No returning to nest pheromones in sight
                        // TODO
                    },
                }

                cell_action_executor(
                    AntCellAction::LayPheremone(PheremoneType::Returning),
                    source_universe_index,
                );
            },
            _ => unimplemented!(),
        }
    }
}
