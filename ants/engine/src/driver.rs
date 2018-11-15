use minutiae::prelude::*;
use minutiae::universe::Universe2D;
use wasm_bindgen::prelude::*;

use super::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn register_tick_callback(cb: &Closure<FnMut() -> ()>);
}

pub struct JSDriver;

impl
    Driver<
        AntCellState,
        AntEntityState,
        AntMutEntityState,
        AntCellAction,
        AntEntityAction,
        Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        Box<
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
        >,
    > for JSDriver
{
    fn init(
        self,
        mut universe: Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
        mut engine: Box<
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
        >,
        mut middleware: Vec<
            Box<
                Middleware<
                    AntCellState,
                    AntEntityState,
                    AntMutEntityState,
                    AntCellAction,
                    AntEntityAction,
                    Universe2D<AntCellState, AntEntityState, AntMutEntityState>,
                    Box<
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
                    >,
                >,
            >,
        >,
    ) {
        let cb = move || {
            for m in middleware.iter_mut() {
                m.before_render(&mut universe);
            }

            engine.step(&mut universe);

            for m in middleware.iter_mut() {
                m.after_render(&mut universe);
            }
        };

        register_tick_callback(&Closure::wrap(box cb));
    }
}
