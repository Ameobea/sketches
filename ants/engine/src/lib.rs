#![feature(box_syntax, const_transmute, thread_local)]

extern crate common;
extern crate minutiae;
extern crate rand_core;
extern crate rand_pcg;
extern crate uuid;
extern crate wasm_bindgen;

use std::{mem, ptr};

use minutiae::{
    emscripten::CanvasRenderer,
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    prelude::*,
    universe::Universe2D,
};
use rand::Rng;
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::*;

pub mod engine;
use self::engine::AntEngine;
pub mod driver;
use self::driver::JSDriver;
pub mod universe_generator;
use self::universe_generator::AntUniverseGenerator;

const UNIVERSE_SIZE: u32 = 800;

#[thread_local]
static mut RNG: Pcg32 = unsafe { mem::transmute(0u128) };

pub fn rng() -> &'static mut Pcg32 { unsafe { &mut RNG } }

pub struct UserConf {
    // universe gen
    pub food_patch_count: usize,
    pub food_patch_size: usize,
    pub food_patch_size_variance: usize,
    pub barrier_patch_count: usize,
    pub barrier_patch_size: usize,
    // ant behavior
    pub wander_transition_chance_percent: f32,
}

const fn default_user_conf() -> UserConf {
    UserConf {
        food_patch_count: 5,
        food_patch_size: 16,
        food_patch_size_variance: 3,
        barrier_patch_count: 6,
        barrier_patch_size: 28,
        wander_transition_chance_percent: 4.5,
    }
}

#[thread_local]
pub static mut ACTIVE_USER_CONF: UserConf = default_user_conf();

pub fn active_conf() -> &'static UserConf { unsafe { &ACTIVE_USER_CONF } }

pub fn active_conf_mut() -> &'static mut UserConf { unsafe { &mut ACTIVE_USER_CONF } }

#[derive(Clone, Default)]
pub struct Pheremones {
    wandering: f32,
    returning: f32,
}

#[derive(Clone)]
pub enum AntCellState {
    Empty(Pheremones),
    Barrier,
    Food(usize),
    Anthill,
}

impl CellState for AntCellState {}

#[derive(Clone, Copy, PartialEq)]
pub enum WanderDirection {
    Up,
    Stay,
    Down,
}

impl WanderDirection {
    pub fn next(self) -> Self {
        let should_change =
            rng().gen_range(0.0, 100.0) <= active_conf().wander_transition_chance_percent;
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

#[derive(Clone, Copy, Default, PartialEq)]
pub struct WanderingState {
    x_dir: WanderDirection,
    y_dir: WanderDirection,
}

impl WanderingState {
    /// Function as a markov chain.  Each tick, there's a chance for each of the two directions to
    /// transition to a different state.
    fn next(self) -> Self {
        let next_state = WanderingState {
            x_dir: self.x_dir.next(),
            y_dir: self.y_dir.next(),
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

#[derive(Clone)]
pub enum AntEntityState {
    Wandering(WanderingState),
    FollowingPheremonesToFood,
    ReturningToNestWithFood,
}

impl EntityState<AntCellState> for AntEntityState {}

#[derive(Clone, Copy, Default)]
pub struct AntMutEntityState {}

impl MutEntityState for AntMutEntityState {}

pub enum PheremoneType {
    Wandering,
    Returning,
}

pub enum AntCellAction {
    LayPheremone(PheremoneType),
}

impl CellAction<AntCellState> for AntCellAction {}

pub enum AntEntityAction {
    UpdateWanderState,
}

impl EntityAction<AntCellState, AntEntityState> for AntEntityAction {}

pub type AntOwnedAction = OwnedAction<AntCellState, AntEntityState, AntCellAction, AntEntityAction>;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn canvas_render(colors: &[u8]);
}

#[inline(always)]
fn average_colors(color1: [u8; 4], color2: [u8; 4]) -> [u8; 4] {
    [
        (color1[0] / 2 + color2[0] / 2),
        (color1[1] / 2 + color2[1] / 2),
        (color2[2] / 2 + color2[2] / 2),
        255,
    ]
}

fn calc_color(
    cell: &Cell<AntCellState>,
    entity_indexes: &[usize],
    _entity_container: &EntityContainer<AntCellState, AntEntityState, AntMutEntityState>,
) -> [u8; 4] {
    if !entity_indexes.is_empty() {
        return [255, 25, 42, 255];
    }

    match cell.state {
        AntCellState::Barrier => [255, 255, 255, 255],
        AntCellState::Empty(Pheremones {
            wandering,
            returning,
        }) => {
            let wandering_intensity = (wandering * 25.0).min(255.0) as u8;
            let returning_intensity = (returning * 25.0).min(255.0) as u8;
            [returning_intensity, 12, wandering_intensity, 255]
        },
        AntCellState::Food(_quantity) => [12, 223, 30, 255],
        AntCellState::Anthill => [252, 223, 32, 255],
    }
}

pub type OurUniverseType = Universe2D<AntCellState, AntEntityState, AntMutEntityState>;

pub type OurEngineType = SerialEngine<
    AntCellState,
    AntEntityState,
    AntMutEntityState,
    AntCellAction,
    AntEntityAction,
    SerialEntityIterator<AntCellState, AntEntityState>,
    OurUniverseType,
>;

struct PheremoneEvaporator;

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
        for cell in &mut universe.cells {
            if let AntCellState::Empty(ref mut pheremones) = cell.state {
                pheremones.returning *= 0.9;
                pheremones.wandering *= 0.9;
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_user_conf() -> UserConf {
    unimplemented!(); // TODO
}

#[wasm_bindgen]
pub fn init_universe() {
    let mut conf = UniverseConf::default();
    conf.size = UNIVERSE_SIZE;
    let engine: Box<OurEngineType> = box AntEngine;

    let universe: OurUniverseType = Universe2D::new(conf, &mut AntUniverseGenerator(active_conf()));
    JSDriver.init(universe, engine, vec![
        box CanvasRenderer::new(UNIVERSE_SIZE as usize, calc_color, canvas_render),
        box PheremoneEvaporator,
    ])
}

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook(); // this will trigger `console.error` to be called during panics
    unsafe {
        ptr::write(
            &mut RNG as *mut _,
            Pcg32::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
        )
    };

    init_universe();
}
