use wasm_bindgen::prelude::*;

use minutiae::{
    emscripten::{driver::JSDriver, CanvasRenderer},
    engine::{iterator::SerialEntityIterator, serial::SerialEngine},
    prelude::*,
    universe::Universe2D,
    util,
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
pub enum NetworkEntityAction {
    IncCharge(f32),
    SetCharge(f32),
}

impl EntityAction<NetworkCellState, NetworkEntityState> for NetworkEntityAction {}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetworkMutEntityState {}

impl MutEntityState for NetworkMutEntityState {}

pub type NetworkOwnedAction =
    OwnedAction<NetworkCellState, NetworkEntityState, NetworkCellAction, NetworkEntityAction>;

fn mix_colors(color1: [u8; 4], weight1: f32, color2: [u8; 4], weight2: f32) -> [u8; 4] {
    [
        (color1[0] as f32 * weight1 + color2[0] as f32 * weight2) as u8,
        (color1[1] as f32 * weight1 + color2[1] as f32 * weight2) as u8,
        (color1[2] as f32 * weight1 + color2[2] as f32 * weight2) as u8,
        (color1[2] as f32 * weight1 + color2[2] as f32 * weight2) as u8,
    ]
}

const ACTIVE_COLOR: [u8; 4] = [66, 135, 245, 255];
const CHARGE_CAP: f32 = 1_000.;

fn calc_color(
    _cell: &Cell<NetworkCellState>,
    entity_indexes: &[usize],
    entity_container: &EntityContainer<NetworkCellState, NetworkEntityState, NetworkMutEntityState>,
) -> [u8; 4] {
    if entity_indexes.is_empty() {
        return [0; 4];
    }

    match unsafe { entity_container.get(entity_indexes[0]) }.state {
        NetworkEntityState::Neuron {
            activation_threshold,
            charge,
            ..
        } => {
            let charge_ratio = (charge / CHARGE_CAP).min(1.0);
            let val = activation_threshold / 1_000_000.0;
            let val = val * 255.0;
            let val = val as u8;
            let base_color = [val, val, val, 255];

            mix_colors(base_color, 1. - charge_ratio, ACTIVE_COLOR, charge_ratio)

            // if charge > 0. {
            //     let val = (charge / 1_000_000.0).min(1.0);
            //     let val = val * 255.0;
            //     let val = val as u8;
            //     // info!("{}", val);
            //     [val, val, val, 255]
            // } else {
            //     [254, 44, 44, 255]
            // }
        }
    }
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

struct DebugMiddleware;

impl
    Middleware<
        NetworkCellState,
        NetworkEntityState,
        NetworkMutEntityState,
        NetworkCellAction,
        NetworkEntityAction,
        NetworkUniverse,
        Box<
            dyn SerialEngine<
                NetworkCellState,
                NetworkEntityState,
                NetworkMutEntityState,
                NetworkCellAction,
                NetworkEntityAction,
                SerialEntityIterator<NetworkCellState, NetworkEntityState>,
                NetworkUniverse,
            >,
        >,
    > for DebugMiddleware
{
    fn after_render(&mut self, universe: &mut NetworkUniverse) {
        for y in 0..(UNIVERSE_SIZE as usize) {
            let mut out = String::new();
            for x in 0..(UNIVERSE_SIZE as usize) {
                let entity_indices = universe.entities.get_entities_at(util::get_index(
                    x,
                    y,
                    UNIVERSE_SIZE as usize,
                ));
                if entity_indices.is_empty() {
                    out.push_str("----, ");
                    continue;
                }
                let entity_index = entity_indices[0];
                let entity_state = unsafe { &universe.entities.get(entity_index).state };
                match entity_state {
                    NetworkEntityState::Neuron {
                        activation_threshold,
                        ..
                    } => out.push_str(&format!("{:.*}, ", 2, activation_threshold)),
                }
            }
            info!("{}", out);
        }
    }
}

#[wasm_bindgen]
pub fn init_universe() {
    let mut conf = UniverseConf::default();
    conf.size = UNIVERSE_SIZE;
    let engine: Box<OurEngineType> = Box::new(NetworkEngine);

    let universe: NetworkUniverse = Universe2D::new(conf, &mut NetworkUniverseGenerator);
    let driver = JSDriver {
        register_tick_callback,
    };
    driver.init(
        universe,
        engine,
        vec![
            Box::new(CanvasRenderer::new(
                UNIVERSE_SIZE as usize,
                calc_color,
                canvas_render,
            )),
            // Box::new(DebugMiddleware),
        ],
    );
}
