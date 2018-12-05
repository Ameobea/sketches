use std::ptr;

use minutiae::prelude::*;
use wasm_bindgen::prelude::*;

use super::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    fn register_tick_callback(cb: &Closure<FnMut() -> ()>);
}

pub struct JSDriver;

#[thread_local]
static mut CLOSURE: *mut Closure<(dyn std::ops::FnMut() + 'static)> = ptr::null_mut();

type OurMiddlewareType = Middleware<
    AntCellState,
    AntEntityState,
    AntMutEntityState,
    AntCellAction,
    AntEntityAction,
    OurUniverseType,
    Box<OurEngineType>,
>;

impl
    Driver<
        AntCellState,
        AntEntityState,
        AntMutEntityState,
        AntCellAction,
        AntEntityAction,
        OurUniverseType,
        Box<OurEngineType>,
    > for JSDriver
{
    fn init(
        self,
        mut universe: OurUniverseType,
        mut engine: Box<OurEngineType>,
        mut middleware: Vec<Box<OurMiddlewareType>>,
    ) {
        // check to see if we have an existing closure (which in turn holds references to all of the
        // universe state) and drop it if we do.
        if unsafe { CLOSURE != ptr::null_mut() } {
            let closure = unsafe { Box::from_raw(CLOSURE) };
            drop(closure);
        }

        let cb = move || {
            for m in middleware.iter_mut() {
                m.before_render(&mut universe);
            }

            engine.step(&mut universe);

            for m in middleware.iter_mut() {
                m.after_render(&mut universe);
            }
        };

        let closure = box Closure::wrap((box cb) as Box<FnMut()>);
        register_tick_callback(&*closure);
        // hold onto the closure we created
        unsafe { CLOSURE = Box::into_raw(closure) };
    }
}
