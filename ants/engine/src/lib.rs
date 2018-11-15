#![feature(box_syntax)]

extern crate common;
extern crate minutiae;
extern crate wasm_bindgen;

use minutiae::emscripten::CanvasRenderer;
use minutiae::engine::iterator::SerialEntityIterator;
use minutiae::engine::serial::SerialEngine;
use minutiae::prelude::*;
use minutiae::universe::Universe2D;
use wasm_bindgen::prelude::*;
extern crate uuid;

pub mod engine;
use self::engine::AntEngine;
pub mod driver;
use self::driver::JSDriver;

const UNIVERSE_SIZE: u32 = 800;

#[derive(Clone)]
pub enum AntCellState {}

impl CellState for AntCellState {}

#[derive(Clone)]
pub enum AntEntityState {}

impl EntityState<AntCellState> for AntEntityState {}

#[derive(Clone, Copy, Default)]
pub struct AntMutEntityState {}

impl MutEntityState for AntMutEntityState {}

pub enum AntCellAction {}

impl CellAction<AntCellState> for AntCellAction {}

pub enum AntEntityAction {}

impl EntityAction<AntCellState, AntEntityState> for AntEntityAction {}

pub type AntOwnedAction =
    OwnedAction<AntCellState, AntEntityState, AntCellAction, AntEntityAction, usize>;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn canvas_render(colors: &[u8]);
}

fn calc_color(
    cell: &Cell<AntCellState>,
    entity_indexes: &[usize],
    entity_container: &EntityContainer<AntCellState, AntEntityState, AntMutEntityState, usize>,
) -> [u8; 4] {
    unimplemented!();
}

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook(); // this will trigger `console.error` to be called during panics

    let mut conf = UniverseConf::default();
    conf.size = UNIVERSE_SIZE;
    let engine: Box<
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
        >,
    > = box AntEngine;

    let universe: Universe2D<AntCellState, AntEntityState, AntMutEntityState> =
        Universe2D::default();
    JSDriver.init(
        universe,
        engine,
        vec![box CanvasRenderer::new(
            UNIVERSE_SIZE as usize,
            calc_color,
            canvas_render,
        )],
    )
}

#[wasm_bindgen]
pub fn hello() {
    common::log("Hello, world!")
}
