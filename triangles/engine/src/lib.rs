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

use nalgebra::{Isometry2, Point2, Translation2, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVT};
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

type World = DBVT<f32, usize, AABB<f32>>;
static mut COLLISION_WORLD: *mut World = ptr::null_mut();
static mut TRIANGLES: *mut Vec<TriangleBuf> = ptr::null_mut();

#[wasm_bindgen]
pub fn init() {
    let world: Box<World> = box DBVT::new();
    let p: *mut World = Box::into_raw(world);
    unsafe { COLLISION_WORLD = p };

    let triangles: Box<Vec<TriangleBuf>> = box Vec::with_capacity(200);
    let p: *mut Vec<TriangleBuf> = Box::into_raw(triangles);
    unsafe { TRIANGLES = p };
}

#[inline]
const fn deg_to_rad(degrees: f32) -> f32 {
    degrees * (f32::consts::PI / 180.0)
}

#[inline]
fn min3(a: f32, b: f32, c: f32) -> f32 {
    a.min(b).min(c)
}

#[inline]
fn max3(a: f32, b: f32, c: f32) -> f32 {
    a.max(b).max(c)
}

#[inline]
fn bounds(p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>) -> (Point2<f32>, Point2<f32>) {
    (
        Point2::new(min3(p1.x, p2.x, p3.x), min3(p1.y, p2.y, p3.y)),
        Point2::new(max3(p1.x, p2.x, p3.x), max3(p1.y, p2.y, p3.y)),
    )
}

#[inline]
fn ccw(p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>) -> bool {
    (p3.y - p1.y) * (p2.x - p1.x) > (p2.y - p1.y) * (p3.x - p1.x)
}

/// adapted from https://stackoverflow.com/a/9997374/3833068
fn check_line_seg_intersection(
    l1p1: Point2<f32>,
    l1p2: Point2<f32>,
    l2p1: Point2<f32>,
    l2p2: Point2<f32>,
) -> bool {
    ccw(l1p1, l2p1, l2p2) != ccw(l1p2, l2p1, l2p2) && ccw(l1p1, l1p2, l2p1) != ccw(l1p1, l1p2, l2p2)
}

/// If any side of the first triangle intersects any side of the second triangle, they intersect.
/// Additionally, if two sides of the first triangle don't intersect, the triangles don't
/// intersect.
fn check_triangle_collision(t1: &TriangleBuf, t2: &TriangleBuf) -> bool {
    false // TODO
}

struct TriangleCollisionVisitor<'a> {
    pub triangle: &'a TriangleBuf,
    pub triangle_bv: &'a AABB<f32>,
    pub triangles: &'a [TriangleBuf],
    pub does_collide: &'a mut bool,
}

impl<'a> BVTVisitor<usize, AABB<f32>> for TriangleCollisionVisitor<'a> {
    fn visit_internal(&mut self, bv: &AABB<f32>) -> bool {
        if (*self.does_collide) {
            return false;
        }

        false // TODO
    }

    fn visit_leaf(&mut self, i: &usize, bv: &AABB<f32>) {
        // TODO: Perform accurate collision detection on the triangles to see if they actually
        // intersect and set `does_collide` to true if it does
        if (*self.does_collide) {
            return;
        }

        if check_triangle_collision(&self.triangle, &self.triangles[*i]) {
            *self.does_collide = true;
        }
    }
}

#[wasm_bindgen]
pub fn render(conf_str: &str) {
    let world: &mut World = unsafe { &mut *COLLISION_WORLD };
    let triangles: &mut Vec<TriangleBuf> = unsafe { &mut *TRIANGLES };
    *world = DBVT::new();
    triangles.clear();

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
        ((triangle_size * triangle_size) - (triangle_offset_x * triangle_offset_x)).sqrt();
    let initial_offset = Vector2::new(canvas_width as f32 / 2.0, canvas_height as f32 / 2.0);
    let base_triangle_coords: TriangleBuf = [
        Point2::origin(),
        Point2::new(-triangle_offset_x, triangle_offset_y),
        Point2::new(triangle_offset_x, triangle_offset_y),
    ];

    let mut last_triangle = [
        base_triangle_coords[0] + initial_offset,
        base_triangle_coords[1] + initial_offset,
        base_triangle_coords[2] + initial_offset,
    ];
    let mut rotation = 0.0;
    for _ in 0..triangle_count {
        // pick one of the other two vertices to use as the new origin
        let (ix, rot_offset) = if common::math_random() > 0.5 {
            (1, deg_to_rad(60.0))
        } else {
            (2, deg_to_rad(-60.0))
        };

        rotation += rot_offset;
        rotation += (common::math_random() as f32 - 0.5) * 2. * max_rotation_rads;
        let origin = last_triangle[ix];

        render_triangle_array(
            &base_triangle_coords,
            &mut last_triangle,
            Isometry2::new(Vector2::new(origin.x, origin.y), rotation),
            triangle_color,
            triangle_border_color,
        );

        let (min, max) = bounds(last_triangle[0], last_triangle[1], last_triangle[2]);
        let bounding_box = AABB::new(min, max);
        triangles.push(last_triangle);
        let leaf_id = world.insert(DBVTLeaf::new(bounding_box, triangles.len()));
    }
}
