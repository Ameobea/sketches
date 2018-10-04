#![feature(box_syntax, const_fn)]

extern crate common;
extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate ncollide2d;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

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
    pub fn render_triangle(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &str,
        border_color: &str,
    );
}

type TriangleBuf = [Point2<f32>; 3];

const RADS_PER_CIRCLE: f32 = f32::consts::PI * 2.0;

#[derive(Deserialize)]
pub struct Conf<'a> {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub triangle_size: f32,
    pub triangle_count: usize,
    pub max_rotation_rads: f32,
    pub triangle_color: &'a str,
    pub triangle_border_color: &'a str,
}

fn render_triangle_array(
    base_triangle: &TriangleBuf,
    transformed: &mut TriangleBuf,
    offset: Isometry2<f32>,
    color: &str,
    border_color: &str,
) {
    let new_triangle = [
        offset * base_triangle[0],
        offset * base_triangle[1],
        offset * base_triangle[2],
    ];
    let [p1, p2, p3] = new_triangle;
    render_triangle(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, color, border_color);
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

const fn deg_to_rad(degrees: f32) -> f32 {
    degrees * (f32::consts::PI / 180.0)
}

#[wasm_bindgen]
pub fn render(conf_str: &str) {
    let world: &mut World = unsafe { &mut *COLLISION_WORLD };
    *world = DBVT::new();
    let Conf {
        canvas_width,
        canvas_height,
        triangle_size,
        triangle_count,
        max_rotation_rads,
        triangle_color,
        triangle_border_color,
    } = match serde_json::from_str(conf_str) {
        Ok(conf) => conf,
        Err(err) => {
            common::error(format!("Error decoding provided conf object: {:?}", err));
            return;
        }
    };
    let triangle_offset_x = triangle_size / 2.0;
    let triangle_offset_y =
        ((triangle_size * triangle_size) - (triangle_offset_x * triangle_offset_x)).sqrt();;
    let base_triangle_coords: TriangleBuf = [
        Point2::new(0.0, 0.0),
        Point2::new(-triangle_offset_x, triangle_offset_y),
        Point2::new(triangle_offset_x, triangle_offset_y),
    ];

    let mut last_triangle = base_triangle_coords;
    let mut rotation = 0.0;
    render_triangle_array(
        &base_triangle_coords,
        &mut last_triangle,
        Isometry2::new(
            Vector2::new(canvas_width as f32 / 2.0, canvas_height as f32 / 2.0),
            rotation,
        ),
        triangle_color,
        triangle_border_color,
    );
    let shape = Triangle::new(last_triangle[0], last_triangle[1], last_triangle[2]);
    // world.insert(DBVTLeaf::new(
    //     shape.to_polyline(()).aabb(&Isometry2::identity()),
    //     (),
    // ));

    for _ in 0..triangle_count {
        // pick one of the other two vertices to use as the new origin
        let (ix, rot_offset) = if common::math_random() > 0.5 {
            (1, deg_to_rad(60.0))
        } else {
            (2, deg_to_rad(-60.0))
        };
        let origin = last_triangle[ix];
        // set a random rotation for now
        rotation += rot_offset;
        rotation += (common::math_random() as f32 - 0.5) * 2. * max_rotation_rads;
        let base = (rotation / RADS_PER_CIRCLE).trunc();
        common::log(format!("{:?}", rotation - base * RADS_PER_CIRCLE));
        render_triangle_array(
            &base_triangle_coords,
            &mut last_triangle,
            Isometry2::new(
                Vector2::new(origin[0], origin[1]),
                rotation - base * RADS_PER_CIRCLE,
            ),
            triangle_color,
            triangle_border_color,
        )
    }
}
