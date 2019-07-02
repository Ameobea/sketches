use wasm_bindgen::prelude::*;

use minutiae::{
    emscripten::{driver::JSDriver, CanvasRenderer},
    prelude::*,
    universe::Universe2D,
};

pub mod engine;
use self::engine::*;
pub mod world_generator;
use self::world_generator::*;

#[wasm_bindgen(raw_module = "./loop")]
extern "C" {
    pub fn register_tick_callback(cb: &Closure<dyn FnMut()>);
    pub fn canvas_render(colors: &[u8]);
}

const UNIVERSE_SIZE: u32 = 800;

#[derive(Clone, Debug, PartialEq)]
pub struct Directions<T: Clone + std::fmt::Debug + PartialEq> {
    pub up: T,
    pub down: T,
    pub right: T,
    pub left: T,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkCellState {
    Empty,
}

impl CellState for NetworkCellState {}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEntityState {
    Neuron {
        /// Total amount of charge currently stored by this neuron
        charge: f32,
        /// The amount of charge that must be reached before this neuron emits its own pulse
        activation_threshold: f32,
        /// Percentage of charge that will be transmitted in each direction when pulses are received
        resistances: Directions<f32>,
    },
}

impl EntityState<NetworkCellState> for NetworkEntityState {}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkCellAction {}

impl CellAction<NetworkCellState> for NetworkCellAction {}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEntityAction {}

impl EntityAction<NetworkCellState, NetworkEntityState> for NetworkEntityAction {}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetworkMutEntityState {}

impl MutEntityState for NetworkMutEntityState {}

pub type NetworkOwnedAction =
    OwnedAction<NetworkCellState, NetworkEntityState, NetworkCellAction, NetworkEntityAction>;

fn calc_color(
    cell: &Cell<NetworkCellState>,
    entity_indexes: &[usize],
    entity_container: &EntityContainer<NetworkCellState, NetworkEntityState, NetworkMutEntityState>,
) -> [u8; 4] {
    unimplemented!();
}

pub fn init_minutiae() {
    let mut conf = UniverseConf::default();
    conf.size = UNIVERSE_SIZE;

    let engine: Box<OurEngineType> = Box::new(engine::NetworkEngine);

    let universe: NetworkUniverse = Universe2D::new(conf, &mut NetworkUniverseGenerator);
    let driver = JSDriver {
        register_tick_callback,
    };
    driver.init(
        universe,
        engine,
        vec![Box::new(CanvasRenderer::new(
            UNIVERSE_SIZE as usize,
            calc_color,
            canvas_render,
        ))],
    )
}
