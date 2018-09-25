#![feature(box_syntax, nll)]

extern crate common;
extern crate wasm_bindgen;

use std::mem;
use std::ptr;

use common::error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn canvasRender(ptr: *const u8);
}

const BOARD_HEIGHT: usize = 150;
const BOARD_WIDTH: usize = 150;
const CELL_COUNT: usize = BOARD_HEIGHT * BOARD_WIDTH;
const CANVAS_SCALE_FACTOR: usize = 6;
const CANVAS_SIZE: usize = CELL_COUNT * 4 * CANVAS_SCALE_FACTOR * CANVAS_SCALE_FACTOR;

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Dead,
    Alive,
}

impl Cell {
    fn is_alive(&self) -> bool {
        *self == Cell::Alive
    }
}

struct Board(pub Box<[Cell; CELL_COUNT]>);

impl Board {
    pub fn new() -> Self {
        let mut cells = [Cell::Dead; CELL_COUNT];
        // Initialize cells randomly
        for i in 0..CELL_COUNT {
            if common::math_random() > 0.5 {
                cells[i] = Cell::Alive;
            }
        }
        Board(box cells)
    }

    pub fn get(&self, x: isize, y: isize) -> Option<Cell> {
        if x < 0 || y < 0 || x >= BOARD_WIDTH as isize || y >= BOARD_HEIGHT as isize {
            return None;
        }

        let index = (y * BOARD_WIDTH as isize) + x;
        Some(self.0[index as usize])
    }
}

struct State {
    pub cur_buf_1: bool,
    pub buf1: Board,
    pub buf2: Board,
    pub canvas_buf: Box<[u8; CANVAS_SIZE]>,
}

#[inline]
fn get_coord(index: usize) -> (isize, isize) {
    let x = index % BOARD_WIDTH;
    let y = (index - x) / BOARD_WIDTH;
    return (x as isize, y as isize);
}

impl State {
    pub fn new() -> Self {
        let mut canvas_buf = box [0u8; CANVAS_SIZE];

        // Set transparency to 1 for all pixels
        for i in 0..CANVAS_SIZE {
            if i % 4 == 3 {
                canvas_buf[i] = 255;
            }
        }

        // Draw initial canvas buf
        let buf1 = Board::new();
        for i in 0..CELL_COUNT {
            if buf1.0[i] == Cell::Alive {
                State::draw_canvas_cell(&mut canvas_buf, i, Cell::Alive);
            }
        }

        State {
            cur_buf_1: true,
            buf1,
            buf2: Board::new(),
            canvas_buf: canvas_buf,
        }
    }

    pub fn draw_canvas_cell(canvas_buf: &mut [u8; CANVAS_SIZE], i: usize, state: Cell) {
        let (x, y) = get_coord(i);
        let write_val: u8 = if state == Cell::Alive { 255 } else { 0 };

        let px_per_row = BOARD_WIDTH * CANVAS_SCALE_FACTOR * 4;
        let px_per_cell_row = px_per_row * CANVAS_SCALE_FACTOR;

        let start_ix = (px_per_cell_row * y as usize) + (4 * x as usize * CANVAS_SCALE_FACTOR);
        for row in 0..CANVAS_SCALE_FACTOR {
            let cell_row_start_index = start_ix + (row * px_per_row);
            for col in 0..CANVAS_SCALE_FACTOR {
                let cell_col_start_index = cell_row_start_index + (col * 4);
                let array_ptr = unsafe { canvas_buf.as_ptr().offset(cell_col_start_index as isize) }
                    as *mut u32;
                unsafe { *array_ptr = mem::transmute((write_val, write_val, write_val, 255u8)) };
            }
        }
    }

    pub fn get_cur_buf(&mut self) -> &mut Board {
        if self.cur_buf_1 {
            &mut self.buf1
        } else {
            &mut self.buf2
        }
    }
}

static mut STATE: *mut State = ptr::null_mut();

#[inline]
fn state() -> &'static mut State {
    unsafe { mem::transmute(STATE) }
}

/// Called by the JS to initialize the game state before starting the simulation
#[wasm_bindgen]
pub fn init() {
    let initial_state = box State::new();
    let initial_state = Box::into_raw(initial_state);
    unsafe { STATE = initial_state as *mut State };
    let state = state();
    canvasRender(state.canvas_buf.as_ptr() as *const u8)
}

#[wasm_bindgen]
pub fn set_pixel(x: usize, y: usize) {
    if x >= BOARD_WIDTH || y >= BOARD_HEIGHT {
        error(format!("({}, {}) is outside of the board", x, y));
        return;
    }

    let state = state();
    let cur_buf = state.get_cur_buf();

    let i = y * BOARD_WIDTH + x;
    let new_val = if cur_buf.0[i] == Cell::Alive {
        Cell::Dead
    } else {
        Cell::Alive
    };
    state.buf1.0[i] = new_val;
    state.buf2.0[i] = new_val;
    State::draw_canvas_cell(&mut state.canvas_buf, i, new_val);
    canvasRender(state.canvas_buf.as_ptr() as *const u8);
}

#[inline]
fn get_next_cell_state(last_buf: &Board, index: usize) -> Cell {
    let cur_state: Cell = last_buf.0[index];
    let (x, y) = get_coord(index);

    let neighbor_offets: [(isize, isize); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let live_neighbor_count = neighbor_offets
        .iter()
        .map(|(x_offset, y_offset)| last_buf.get(x + *x_offset, y + *y_offset))
        .filter(Option::is_some)
        .map(Option::unwrap)
        .filter(Cell::is_alive)
        .count();

    if cur_state == Cell::Alive {
        // Die if underpopulated
        if live_neighbor_count < 2 {
            return Cell::Dead;
        }
        // Live if 2 or 3 neighbors
        else if live_neighbor_count == 2 || live_neighbor_count == 3 {
            return Cell::Alive;
        }
        // Die if more than 3 neighbors
        else {
            return Cell::Dead;
        }
    }

    // Spawn new cell if exactly three neighbors
    if live_neighbor_count == 3 {
        return Cell::Alive;
    }

    // Stay dead as the base case
    return Cell::Dead;
}

#[wasm_bindgen]
pub fn set_state(canvas_pattern: &[u8]) {
    let state = state();

    for (i, cell) in canvas_pattern.iter().enumerate() {
        let cur_buf = state.get_cur_buf();
        let cell_state = if *cell == 0 { Cell::Dead } else { Cell::Alive };
        if cur_buf.0[i] != cell_state {
            cur_buf.0[i] = cell_state;
            State::draw_canvas_cell(&mut state.canvas_buf, i, cell_state);
        }
    }

    canvasRender(state.canvas_buf.as_ptr() as *const u8);
}

#[wasm_bindgen]
pub fn set_random_state() {
    let state = state();

    for i in 0..CELL_COUNT {
        let cur_buf = state.get_cur_buf();
        let new_state = if common::math_random() > 0.5 {
            Cell::Alive
        } else {
            Cell::Dead
        };
        if cur_buf.0[i] != new_state {
            cur_buf.0[i] = new_state;
            State::draw_canvas_cell(&mut state.canvas_buf, i, new_state);
        }
    }

    canvasRender(state.canvas_buf.as_ptr() as *const u8);
}

#[wasm_bindgen]
pub fn tick() {
    let state = state();
    let (last_board, target_board): (&Board, &mut Board) = if state.cur_buf_1 {
        (&state.buf1, &mut state.buf2)
    } else {
        (&state.buf2, &mut state.buf1)
    };

    for i in 0..CELL_COUNT {
        let new_val_for_cell = get_next_cell_state(last_board, i);
        target_board.0[i] = new_val_for_cell;

        if last_board.0[i] != new_val_for_cell {
            State::draw_canvas_cell(&mut state.canvas_buf, i, new_val_for_cell);
        }
    }
    state.cur_buf_1 = !state.cur_buf_1;

    canvasRender(state.canvas_buf.as_ptr() as *const u8);
}
