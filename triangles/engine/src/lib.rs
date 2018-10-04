#![feature(box_syntax)]

extern crate common;
extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate ncollide2d;

use std::f32;
use std::ptr;

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::partitioning::{DBVTLeaf, DBVT};
use ncollide2d::shape::{Shape, Triangle};
use ncollide2d::transformation::ToPolyline;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn render_triangle(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: &str);
}

type TriangleBuf = [Point2<f32>; 3];

const CANVAS_HEIGHT: usize = 800;
const CANVAS_WIDTH: usize = 800;
const TRIANGLE_SIZE: f32 = 7.0;
const TRIANGLE_OFFSET_X: f32 = TRIANGLE_SIZE / 2.0;
const TRIANGLE_COUNT: usize = 200;
const MAX_ROTATION_RADS: f32 = 0.;

lazy_static! {
    static ref TRIANGLE_OFFSET_Y: f32 =
        ((TRIANGLE_SIZE * TRIANGLE_SIZE) - (TRIANGLE_OFFSET_X * TRIANGLE_OFFSET_X)).sqrt();
    static ref TRIANGLE_COORDS: TriangleBuf = [
        Point2::new(0.0, 0.0),
        Point2::new(-TRIANGLE_OFFSET_X, *TRIANGLE_OFFSET_Y),
        Point2::new(TRIANGLE_OFFSET_X, *TRIANGLE_OFFSET_Y),
    ];
}

fn render_triangle_array(transformed: &mut TriangleBuf, offset: Isometry2<f32>, color: &str) {
    let new_triangle = [
        offset * (*TRIANGLE_COORDS)[0],
        offset * (*TRIANGLE_COORDS)[1],
        offset * (*TRIANGLE_COORDS)[2],
    ];
    let [p1, p2, p3] = new_triangle;
    render_triangle(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, color);
    *transformed = new_triangle
}

type World = DBVT<f32, (), AABB<f32>>;
static mut COLLISION_WORLD: *mut World = ptr::null_mut();

#[wasm_bindgen]
pub fn init() {
    let world: Box<World> = box DBVT::new();
    let p: *mut World = Box::into_raw(world);
    unsafe {
        COLLISION_WORLD = p;
    }
}

#[wasm_bindgen]
pub fn render() {
    let world: &mut World = unsafe { &mut *COLLISION_WORLD };

    let color = "#ab13dc";
    let mut last_triangle = *TRIANGLE_COORDS;
    let mut rotation = 0.0;
    render_triangle_array(
        &mut last_triangle,
        Isometry2::new(Vector2::new(400.0, 400.0), rotation),
        color,
    );
    let shape = Triangle::new(last_triangle[0], last_triangle[1], last_triangle[2]);
    world.insert(DBVTLeaf::new(
        shape.to_polyline(0.0).aabb(&Isometry2::identity()),
        (),
    ));

    for _ in 0..TRIANGLE_COUNT {
        // pick one of the other two vertices to use as the new origin
        let (ix, rot_offset) = if common::math_random() > 0.5 {
            (1, 1. * (2.0 * f32::consts::PI) / 3.)
        } else {
            (2, 2. * (2.0 * f32::consts::PI) / 3.)
        };
        let origin = last_triangle[ix];
        // set a random rotation for now
        rotation += rot_offset + (common::math_random() as f32 - 0.5) * 2. * MAX_ROTATION_RADS;
        render_triangle_array(
            &mut last_triangle,
            Isometry2::new(Vector2::new(origin[0], origin[1]), rotation),
            color,
        )
    }
}
