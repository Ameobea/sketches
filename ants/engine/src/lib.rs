#![feature(
    box_syntax,
    const_fn_union,
    const_transmute,
    thread_local,
    bind_by_move_pattern_guards,
    nll
)]

extern crate common;
extern crate minutiae;
extern crate rand_core;
extern crate rand_pcg;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

use std::mem;

use minutiae::{
    emscripten::CanvasRenderer,
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    prelude::*,
    universe::Universe2D,
};
use rand::Rng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::*;

pub mod engine;
use self::engine::AntEngine;
pub mod driver;
use self::driver::JSDriver;
pub mod universe_generator;
use self::universe_generator::AntUniverseGenerator;
pub mod conf;
use self::conf::*;

const UNIVERSE_SIZE: u32 = 300;
const VIEW_DISTANCE: usize = 1;

#[thread_local]
static mut RNG: Pcg32 = unsafe { mem::transmute(0u128) };

fn reinit_rng() {
    *rng() = unsafe { mem::transmute((-42234i32, 1991u32, -234i32, 44444u32)) };
    rng().gen::<u32>();
    rng().gen::<u32>();
}

pub fn rng() -> &'static mut Pcg32 { unsafe { &mut RNG } }

#[derive(Clone, Default, Copy, Debug)]
pub struct Pheremones {
    wandering: f32,
    returning: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum AntCellState {
    Empty(Pheremones),
    Barrier,
    Food(usize),
    Anthill,
}

impl AntCellState {
    pub fn is_traversable(&self) -> bool {
        match self {
            AntCellState::Barrier => false,
            _ => true,
        }
    }
}

impl CellState for AntCellState {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WanderDirection {
    Up,
    Stay,
    Down,
}

impl WanderDirection {
    pub fn next(self, transition_chance_percentage: f32) -> Self {
        let should_change = rng().gen_range(0.0, 100.0) <= transition_chance_percentage;
        if !should_change {
            return self;
        }

        match self {
            WanderDirection::Up | WanderDirection::Down => WanderDirection::Stay,
            WanderDirection::Stay =>
                if rng().gen::<bool>() {
                    WanderDirection::Up
                } else {
                    WanderDirection::Down
                },
        }
    }

    pub fn get_coord_offset(self) -> isize {
        match self {
            WanderDirection::Up => -1,
            WanderDirection::Stay => 0,
            WanderDirection::Down => 1,
        }
    }
}

impl Default for WanderDirection {
    fn default() -> Self {
        match rng().gen_range(0, 3) {
            0 => WanderDirection::Up,
            1 => WanderDirection::Stay,
            2 => WanderDirection::Down,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WanderingState {
    x_dir: WanderDirection,
    y_dir: WanderDirection,
}

impl WanderingState {
    /// Function as a markov chain.  Each tick, there's a chance for each of the two directions to
    /// transition to a different state.
    fn next(self, transition_chance: f32) -> Self {
        let next_state = WanderingState {
            x_dir: self.x_dir.next(transition_chance),
            y_dir: self.y_dir.next(transition_chance),
        };

        if next_state
            == (WanderingState {
                x_dir: WanderDirection::Stay,
                y_dir: WanderDirection::Stay,
            })
        {
            Self::default()
        } else {
            next_state
        }
    }
}

#[derive(Clone, Debug)]
pub enum AntEntityState {
    Wandering(WanderingState),
    FollowingPheremonesToFood { last_diff: (isize, isize) },
    ReturningToNestWithFood { last_diff: (isize, isize) },
}

impl EntityState<AntCellState> for AntEntityState {}

#[derive(Clone, Copy, Default)]
pub struct AntMutEntityState {
    pub nest_x: usize,
    pub nest_y: usize,
}

impl MutEntityState for AntMutEntityState {}

pub enum PheremoneType {
    Wandering,
    Returning,
}

pub enum AntCellAction {
    LayPheremone(PheremoneType, f32),
    EatFood,
}

impl CellAction<AntCellState> for AntCellAction {}

pub enum AntEntityAction {
    UpdateWanderState { reset: bool },
    DepositFood,
    FollowFoodTrail(isize, isize),
}

impl EntityAction<AntCellState, AntEntityState> for AntEntityAction {}

pub type AntOwnedAction = OwnedAction<AntCellState, AntEntityState, AntCellAction, AntEntityAction>;

#[wasm_bindgen(raw_module = "./loop")]
extern "C" {
    pub fn canvas_render(colors: &[u8]);
}

// #[inline(always)]
// fn average_colors(color1: [u8; 4], color2: [u8; 4]) -> [u8; 4] {
//     [
//         (color1[0] / 2 + color2[0] / 2),
//         (color1[1] / 2 + color2[1] / 2),
//         (color2[2] / 2 + color2[2] / 2),
//         255,
//     ]
// }

fn calc_color(
    cell: &Cell<AntCellState>,
    entity_indexes: &[usize],
    entity_container: &EntityContainer<AntCellState, AntEntityState, AntMutEntityState>,
) -> [u8; 4] {
    if let Some(entity_ix) = entity_indexes.first() {
        return match unsafe { &entity_container.get(*entity_ix).state } {
            AntEntityState::Wandering(_) => [67, 239, 55, 255],
            AntEntityState::ReturningToNestWithFood { .. } => [255, 251, 63, 255],
            AntEntityState::FollowingPheremonesToFood { .. } => [234, 30, 156, 255],
        };
    }

    match cell.state {
        AntCellState::Barrier => [255, 255, 255, 120],
        AntCellState::Empty(Pheremones {
            wandering,
            returning,
        }) => {
            let wandering_intensity = (wandering * 25.0).min(255.0) as u8;
            let returning_intensity = (returning * 25.0).min(255.0) as u8;
            [returning_intensity, 12, wandering_intensity, 120]
        },
        AntCellState::Food(_quantity) => [12, 223, 30, 120],
        AntCellState::Anthill => [252, 223, 32, 120],
    }
}

pub type OurUniverseType = Universe2D<AntCellState, AntEntityState, AntMutEntityState>;

pub type OurEngineType = dyn SerialEngine<
    AntCellState,
    AntEntityState,
    AntMutEntityState,
    AntCellAction,
    AntEntityAction,
    SerialEntityIterator<AntCellState, AntEntityState>,
    OurUniverseType,
>;

struct PheremoneEvaporator(usize);

impl
    Middleware<
        AntCellState,
        AntEntityState,
        AntMutEntityState,
        AntCellAction,
        AntEntityAction,
        OurUniverseType,
        Box<OurEngineType>,
    > for PheremoneEvaporator
{
    fn before_render(&mut self, universe: &mut OurUniverseType) {
        self.0 += 1;
        let UserConf {
            pheremone_decay_interval,
            pheremone_decay_multiplier,
            ..
        } = active_conf();
        if self.0 % (*pheremone_decay_interval as usize) != 0 {
            return;
        }

        for cell in &mut universe.cells {
            if let AntCellState::Empty(ref mut pheremones) = cell.state {
                pheremones.returning *= pheremone_decay_multiplier;
                pheremones.wandering *= pheremone_decay_multiplier;

                if pheremones.returning < 0.5 {
                    pheremones.returning = 0.0;
                }

                if pheremones.wandering < 0.5 {
                    pheremones.wandering = 0.0;
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_user_conf(conf_json: String) {
    let conf: UserConf = match serde_json::from_str(&conf_json) {
        Ok(conf) => conf,
        Err(err) => {
            common::log(format!("Error parsing provided user conf JSON: {:?}", err));
            return;
        },
    };

    unsafe { ACTIVE_USER_CONF = conf };
}

#[wasm_bindgen]
pub fn init_universe() {
    let mut conf = UniverseConf::default();
    conf.size = UNIVERSE_SIZE;
    let engine: Box<OurEngineType> = box AntEngine;

    let universe: OurUniverseType = Universe2D::new(conf, &mut AntUniverseGenerator(active_conf()));
    JSDriver.init(universe, engine, vec![
        box CanvasRenderer::new(UNIVERSE_SIZE as usize, calc_color, canvas_render),
        box PheremoneEvaporator(0),
    ])
}

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook(); // this will trigger `console.error` to be called during panics
    reinit_rng();

    init_universe();
}
