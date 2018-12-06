use std::cmp::Ordering;

use minutiae::{
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    universe::Universe2D,
};
use rand::seq::SliceRandom;

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
                AntCellAction::LayPheremone(pheremone_type, intensity) => {
                    let pheremones = if let AntCellState::Empty(pheremones) = &mut cell.state {
                        pheremones
                    } else {
                        return;
                    };

                    match pheremone_type {
                        PheremoneType::Wandering =>
                            pheremones.wandering = (pheremones.wandering + intensity).min(18.0),
                        PheremoneType::Returning =>
                            pheremones.returning = (pheremones.returning + intensity).min(18.0),
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
                                entity.state =
                                    AntEntityState::ReturningToNestWithFood { last_diff: (0, 0) },
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
                    None => return, // entity has been deleted, so do nothing.
                };

            match self_action {
                SelfAction::Translate(x_offset, y_offset) => {
                    let (cur_x, cur_y) = get_coords(universe_index, UNIVERSE_SIZE as usize);
                    let (new_x, new_y) = (
                        (cur_x as isize + x_offset) as usize,
                        (cur_y as isize + y_offset) as usize,
                    );

                    // We assume that ants are well-behaved and don't request illegal moves.
                    let dst_universe_index =
                        get_index(new_x as usize, new_y as usize, UNIVERSE_SIZE as usize);

                    // Only allow moves onto empty squares
                    if !universe.cells[dst_universe_index].state.is_traversable() {
                        if let AntEntityState::Wandering(ref mut wander_state) = entity.state {
                            *wander_state = WanderingState::default();
                            return;
                        }
                    }

                    if let AntEntityState::ReturningToNestWithFood { ref mut last_diff } =
                        entity.state
                    {
                        *last_diff = (*x_offset, *y_offset);
                    }

                    universe
                        .entities
                        .move_entity(entity_index, dst_universe_index);
                },
                SelfAction::Custom(AntEntityAction::UpdateWanderState { reset }) =>
                    if let AntEntityState::Wandering(ref mut wander_state) = entity.state {
                        *wander_state = if *reset {
                            WanderingState::default()
                        } else {
                            wander_state.next(active_conf().wander_transition_chance_percent)
                        };
                    } else {
                        entity.state = AntEntityState::Wandering(WanderingState::default());
                    },
                SelfAction::Custom(AntEntityAction::DepositFood) => {
                    common::log("Food deposited!");
                    entity.state = AntEntityState::Wandering(WanderingState::default());
                },
                SelfAction::Custom(AntEntityAction::FollowFoodTrail(x_diff, y_diff)) => {
                    // 20% chance of ignoring
                    // if rng().gen_range(1, 10) <= 2 {
                    //     return;
                    // }

                    let last_diff = (*x_diff, *y_diff);
                    entity.state = AntEntityState::FollowingPheremonesToFood { last_diff };
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
    pred: impl Fn(((usize, usize), &AntCellState)) -> bool,
) -> Option<(usize, usize)> {
    let reduce_closest_food = |acc: Option<((usize, usize), usize)>, ((x, y), cell_state)| {
        if pred(((x, y), cell_state)) {
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

/// Verifies we're not trying to move diagonally through non-traversable blocks
fn validate_diagonal(
    cur_x: usize,
    cur_y: usize,
    new_x: usize,
    new_y: usize,
    cells: &[Cell<AntCellState>],
) -> bool {
    if cur_x != new_x && cur_y != new_y {
        let (c1, c2) = (
            &cells[get_index(cur_x, new_y, UNIVERSE_SIZE as usize)],
            &cells[get_index(new_x, cur_y, UNIVERSE_SIZE as usize)],
        );
        c1.state.is_traversable() && c2.state.is_traversable()
    } else {
        true
    }
}

fn validate_move(
    cur_x: usize,
    cur_y: usize,
    x_diff: isize,
    y_diff: isize,
    cells: &[Cell<AntCellState>],
) -> Option<(usize, usize)> {
    let new_x = cur_x as isize + x_diff;
    let new_y = cur_y as isize + y_diff;

    // verify that the supplied desination coordinates are in bounds
    let valid_move = new_x >= 0
        && new_x < UNIVERSE_SIZE as isize
        && new_y >= 0
        && new_y < UNIVERSE_SIZE as isize
        && cells[get_index(new_x as usize, new_y as usize, UNIVERSE_SIZE as usize)]
            .state
            .is_traversable()
        && validate_diagonal(cur_x, cur_y, new_x as usize, new_y as usize, cells);

    if valid_move {
        Some((new_x as usize, new_y as usize))
    } else {
        None
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
        _entity_action_executor: &mut dyn std::ops::FnMut(AntEntityAction, usize, uuid::Uuid),
    ) {
        let (cur_x, cur_y) = get_coords(source_universe_index, UNIVERSE_SIZE as usize);

        let mut lay_pheromone = |phero_type: PheremoneType, intensity: f32| {
            cell_action_executor(
                AntCellAction::LayPheremone(phero_type, intensity),
                source_universe_index,
            );
        };

        match &entity.state {
            AntEntityState::Wandering(WanderingState { x_dir, y_dir }) => {
                let mut translate = |x_diff, y_diff| -> Result<(usize, usize), ()> {
                    let new_coords =
                        match validate_move(cur_x, cur_y, x_diff, y_diff, &universe.cells) {
                            Some(new_coords) => new_coords,
                            None => Err(())?,
                        };
                    self_action_executor(SelfAction::Translate(x_diff, y_diff));
                    Ok(new_coords)
                };

                let mut move_towards = |cur_x: usize,
                                        cur_y: usize,
                                        dst_x: usize,
                                        dst_y: usize|
                 -> Result<(usize, usize), ()> {
                    let (xdiff, ydiff) = (
                        dst_x as isize - cur_x as isize,
                        dst_y as isize - cur_y as isize,
                    );
                    let (xdiff, ydiff) = (clamp(xdiff, -1, 1), clamp(ydiff, -1, 1));
                    translate(xdiff, ydiff)
                };

                // Check if we're currently standing on food
                if let AntCellState::Food(_) = universe.cells[source_universe_index].state {
                    // consume the food and path back towards the nest
                    cell_action_executor(AntCellAction::EatFood, source_universe_index);
                    return;
                }

                let closest_food =
                    find_closest(cur_x, cur_y, &universe.cells, |((x, y), cell_state)| {
                        if let AntCellState::Food(_) = cell_state {
                            // make sure it's not an invalid diagonal move
                            validate_diagonal(cur_x, cur_y, x, y, &universe.cells)
                        } else {
                            false
                        }
                    });

                if let Some((x, y)) = closest_food {
                    // We see food!  Path towards it.
                    move_towards(cur_x, cur_y, x, y)
                        .expect("Invalid movement attempt while moving towards closest food");

                    lay_pheromone(PheremoneType::Wandering, 0.5);
                    lay_pheromone(PheremoneType::Returning, 0.5);
                    return;
                }

                let mut surrounding_weights: [((usize, usize), f32); 9] = [((0, 0), 1.0); 9];
                let mut total_wander = 0.0;
                let mut total_return = 0.0;
                get_visible_cells_iterator(cur_x, cur_y, &universe.cells)
                    .enumerate()
                    .for_each(|(i, ((x, y), cell_state))| {
                        if let AntCellState::Empty(pheromones) = cell_state {
                            total_wander += pheromones.wandering;
                            total_return += pheromones.returning;
                            surrounding_weights[i] = ((x, y), (pheromones.returning).max(0.0));
                        }
                    });

                if surrounding_weights.iter().all(|(_, weight)| *weight < 1.0) {
                    // No food within sight, no food trails to follow, so continue wandering.
                    let movement_successful =
                        translate(x_dir.get_coord_offset(), y_dir.get_coord_offset()).is_ok();
                    self_action_executor(SelfAction::Custom(AntEntityAction::UpdateWanderState {
                        reset: !movement_successful,
                    }));
                    if movement_successful {
                        lay_pheromone(PheremoneType::Wandering, 1.0);
                    }
                    return;
                }

                let ((x, y), _) = surrounding_weights
                    .choose_weighted(rng(), |(_, weight)| *weight)
                    .expect("Probability choice failed");
                if move_towards(cur_x, cur_y, *x, *y).is_ok() {
                    lay_pheromone(PheremoneType::Wandering, 1.5);
                    self_action_executor(SelfAction::Custom(AntEntityAction::FollowFoodTrail(
                        *x as isize - cur_x as isize,
                        *y as isize - cur_y as isize,
                    )));
                    return;
                };
            },
            AntEntityState::ReturningToNestWithFood { last_diff } => {
                let mut translate = |x_diff, y_diff| -> Result<(usize, usize), ()> {
                    let new_coords =
                        match validate_move(cur_x, cur_y, x_diff, y_diff, &universe.cells) {
                            Some(new_coords) => new_coords,
                            None => Err(())?,
                        };
                    self_action_executor(SelfAction::Translate(x_diff, y_diff));
                    Ok(new_coords)
                };

                let mut move_towards = |cur_x: usize,
                                        cur_y: usize,
                                        dst_x: usize,
                                        dst_y: usize|
                 -> Result<(usize, usize), ()> {
                    let (xdiff, ydiff) = (
                        dst_x as isize - cur_x as isize,
                        dst_y as isize - cur_y as isize,
                    );
                    let (xdiff, ydiff) = (clamp(xdiff, -1, 1), clamp(ydiff, -1, 1));
                    translate(xdiff, ydiff)
                };

                // If we've reached the anthill (hooray), deposit food and switch back to wandering
                if let AntCellState::Anthill = universe.cells[source_universe_index].state {
                    self_action_executor(SelfAction::Custom(AntEntityAction::DepositFood));
                    return;
                }

                let (mut x_bias, mut y_bias) = (last_diff.0 * 3, last_diff.0 * 3);

                let mut inc_biases = |bias: isize, x: usize, y: usize| {
                    let (x_diff, y_diff) =
                        (x as isize - cur_x as isize, y as isize - cur_y as isize);
                    x_bias += x_diff * bias;
                    y_bias += y_diff * bias;
                };
                let mut max_wander_phero = (0.0, (0, 0));
                let AntMutEntityState { nest_x, nest_y } = entity.mut_state;
                let diff_mag =
                    ((cur_x as isize - nest_x as isize) + (cur_y as isize - nest_y as isize)).abs();
                inc_biases((diff_mag / 2).min(3), nest_x, nest_y);

                for ((x, y), cell_state) in
                    get_visible_cells_iterator(cur_x, cur_y, &universe.cells)
                {
                    match cell_state {
                        AntCellState::Anthill =>
                            if move_towards(cur_x, cur_y, x, y).is_ok() {
                                lay_pheromone(PheremoneType::Returning, 2.0);
                                return;
                            },
                        AntCellState::Barrier | AntCellState::Food(_) => inc_biases(-2, x, y),
                        AntCellState::Empty(pheromones) => {
                            if pheromones.wandering > pheromones.returning
                                && pheromones.returning > 0.0
                            {
                                inc_biases(2, x, y);
                            } else if pheromones.returning > pheromones.wandering {
                                inc_biases(-1, x, y);
                            }

                            if pheromones.wandering > max_wander_phero.0 {
                                max_wander_phero = (pheromones.wandering, (x, y));
                            }
                        },
                    }
                }
                if max_wander_phero.0 > 0.0 {
                    inc_biases(1, (max_wander_phero.1).0, (max_wander_phero.1).1);
                }

                match translate(clamp(x_bias, -1, 1), clamp(y_bias, -1, 1)) {
                    Ok(_) => lay_pheromone(PheremoneType::Returning, 4.0),
                    Err(()) => {
                        for _ in 0..6 {
                            if rng().gen::<bool>() {
                                x_bias = 0
                            } else {
                                y_bias = 0;
                            }
                            if x_bias == 0 && y_bias == 0 {
                                break;
                            }

                            if translate(clamp(x_bias, -1, 1), clamp(y_bias, -1, 1)).is_ok() {
                                lay_pheromone(PheremoneType::Returning, 1.0);
                                return;
                            }
                        }

                        // Try to move *anywhere*
                        for y in -1..=1 {
                            for x in -1..=1 {
                                if translate(x, y).is_ok() {
                                    lay_pheromone(PheremoneType::Returning, 1.0);
                                    return;
                                }
                            }
                        }

                        common::error("We're PROPER stuck.");
                    },
                }
            },
            AntEntityState::FollowingPheremonesToFood {
                last_diff: (last_diff_x, last_diff_y),
            } => {
                // Check if we're currently standing on food
                if let AntCellState::Food(_) = universe.cells[source_universe_index].state {
                    // consume the food and path back towards the nest
                    cell_action_executor(AntCellAction::EatFood, source_universe_index);
                    return;
                }

                let closest_food =
                    find_closest(cur_x, cur_y, &universe.cells, |((x, y), cell_state)| {
                        if let AntCellState::Food(_) = cell_state {
                            // make sure it's not an invalid diagonal move
                            validate_diagonal(cur_x, cur_y, x, y, &universe.cells)
                        } else {
                            false
                        }
                    });

                if let Some((x, y)) = closest_food {
                    let mut translate = |x_diff, y_diff| -> Result<(usize, usize), ()> {
                        let new_coords =
                            match validate_move(cur_x, cur_y, x_diff, y_diff, &universe.cells) {
                                Some(new_coords) => new_coords,
                                None => Err(())?,
                            };
                        self_action_executor(SelfAction::Translate(x_diff, y_diff));
                        Ok(new_coords)
                    };

                    let mut move_towards = |cur_x: usize,
                                            cur_y: usize,
                                            dst_x: usize,
                                            dst_y: usize|
                     -> Result<(usize, usize), ()> {
                        let (xdiff, ydiff) = (
                            dst_x as isize - cur_x as isize,
                            dst_y as isize - cur_y as isize,
                        );
                        let (xdiff, ydiff) = (clamp(xdiff, -1, 1), clamp(ydiff, -1, 1));
                        translate(xdiff, ydiff)
                    };
                    // We see food!  Path towards it.
                    move_towards(cur_x, cur_y, x, y)
                        .expect("Invalid movement attempt while moving towards closest food");

                    lay_pheromone(PheremoneType::Wandering, 0.5);
                    lay_pheromone(PheremoneType::Returning, 0.5);
                    return;
                }

                let mut surrounding_weights: [((usize, usize), f32); 9] = [((0, 0), 1.0); 9];
                let mut total_wander = 0.0;
                let mut total_return = 0.0;
                get_visible_cells_iterator(cur_x, cur_y, &universe.cells)
                    .enumerate()
                    .for_each(|(i, ((x, y), cell_state))| {
                        if let AntCellState::Empty(pheromones) = cell_state {
                            total_wander += pheromones.wandering;
                            total_return += pheromones.returning;
                            surrounding_weights[i] = ((x, y), pheromones.returning);
                        }
                    });

                // If there's more than 2x as much wandering pheros as there are returning
                // surrounding us, 20% chance of going into wander mode.
                // if total_wander > 20.0
                //     && total_wander > (total_return * 2.0)
                //     && rng().gen_range(0, 10) >= 2
                // {
                //     self_action_executor(SelfAction::Custom(AntEntityAction::UpdateWanderState {
                //         reset: true,
                //     }));
                //     return;
                // }

                // apply bias of last diff
                for ((x, y), weight) in &mut surrounding_weights {
                    if (*x, *y) == (cur_x, cur_y) {
                        *weight = 0.0;
                    }

                    // if (*x as isize == cur_x as isize + last_diff_x)
                    //     && (*y as isize == cur_y as isize + last_diff_y)
                    // {
                    //     *weight *= 0.2;
                    // }

                    if ((*last_diff_x < 0) == (*x < cur_x)) || ((*last_diff_x > 0) == (*x > cur_x))
                    {
                        *weight *= 2.0;
                    } else if ((*last_diff_x < 0) == (*x > cur_x))
                        || ((*last_diff_x > 0) == (*x < cur_x))
                    {
                        *weight *= 0.5;
                    }

                    if ((*last_diff_y < 0) == (*y < cur_y)) || ((*last_diff_y > 0) == (*y > cur_y))
                    {
                        *weight *= 2.0;
                    } else if ((*last_diff_y < 0) == (*y > cur_y))
                        || ((*last_diff_y > 0) == (*y < cur_y))
                    {
                        *weight *= 0.5;
                    }
                }

                if surrounding_weights.iter().all(|(_, weight)| *weight < 8.0) {
                    // We've lost the trail; go back to wandering.
                    self_action_executor(SelfAction::Custom(AntEntityAction::UpdateWanderState {
                        reset: true,
                    }));
                    return;
                }

                let mut translate = |x_diff, y_diff| -> Result<(usize, usize), ()> {
                    let new_coords =
                        match validate_move(cur_x, cur_y, x_diff, y_diff, &universe.cells) {
                            Some(new_coords) => new_coords,
                            None => Err(())?,
                        };
                    self_action_executor(SelfAction::Translate(x_diff, y_diff));
                    Ok(new_coords)
                };

                let mut move_towards = |cur_x: usize,
                                        cur_y: usize,
                                        dst_x: usize,
                                        dst_y: usize|
                 -> Result<(usize, usize), ()> {
                    let (xdiff, ydiff) = (
                        dst_x as isize - cur_x as isize,
                        dst_y as isize - cur_y as isize,
                    );
                    let (xdiff, ydiff) = (clamp(xdiff, -1, 1), clamp(ydiff, -1, 1));
                    translate(xdiff, ydiff)
                };

                let ((x, y), _) = surrounding_weights
                    .choose_weighted(rng(), |(_, weight)| *weight)
                    .expect("Probability choice failed");
                if move_towards(cur_x, cur_y, *x, *y).is_ok() {
                    lay_pheromone(PheremoneType::Wandering, 1.5);
                };
            },
        }
    }
}
