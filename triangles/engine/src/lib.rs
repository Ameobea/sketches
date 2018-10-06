#![feature(box_syntax, const_fn)]

extern crate common;
extern crate nalgebra;
extern crate ncollide2d;
extern crate rand_core;
extern crate rand_pcg;
extern crate serde;
extern crate serde_json;
extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

use std::f32;
use std::mem;
use std::ptr;

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, BoundingVolume};
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVT};
use rand::Rng;
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
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
    pub fn render_quad(x: f32, y: f32, width: f32, height: f32, color: &str, border_color: &str);
}

type TriangleBuf = [Point2<f32>; 3];

const PLACEMENT_ATTEMPTS: usize = 3;

#[derive(Deserialize)]
pub struct Conf<'a> {
    pub prng_seed: f64,
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub triangle_size: f32,
    pub triangle_count: usize,
    pub max_rotation_rads: f32,
    pub triangle_color: &'a str,
    pub triangle_border_color: &'a str,
    pub rotation_offset: f32,
    pub debug_bounding_boxes: bool,
}

#[inline(always)]
fn p2(x: f32, y: f32) -> Point2<f32> {
    Point2::new(x, y)
}

fn render_triangle_array(
    base_triangle: &TriangleBuf,
    result_buf: &mut TriangleBuf,
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
    *result_buf = new_triangle
}

type World = DBVT<f32, usize, AABB<f32>>;
static mut COLLISION_WORLD: *mut World = ptr::null_mut();
static mut TRIANGLES: *mut Vec<TriangleBuf> = ptr::null_mut();
static mut RNG: *mut Pcg32 = ptr::null_mut();

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook();

    let world: Box<World> = box DBVT::new();
    let p: *mut World = Box::into_raw(world);
    unsafe { COLLISION_WORLD = p };

    let triangles: Box<Vec<TriangleBuf>> = box Vec::with_capacity(200);
    let p: *mut Vec<TriangleBuf> = Box::into_raw(triangles);
    unsafe { TRIANGLES = p };

    let rng_seed: [u8; 16] = unsafe { mem::transmute(1u128) };
    let rng: Box<Pcg32> = box Pcg32::from_seed(rng_seed);
    let p: *mut Pcg32 = Box::into_raw(rng);
    unsafe { RNG = p };
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
fn bounds(v1: Point2<f32>, v2: Point2<f32>, v3: Point2<f32>) -> (Point2<f32>, Point2<f32>) {
    (
        p2(min3(v1.x, v2.x, v3.x), min3(v1.y, v2.y, v3.y)),
        p2(max3(v1.x, v2.x, v3.x), max3(v1.y, v2.y, v3.y)),
    )
}

#[inline]
fn ccw(p1: Point2<f32>, p2: Point2<f32>, p3: Point2<f32>) -> bool {
    (p3.y - p1.y) * (p2.x - p1.x) >= (p2.y - p1.y) * (p3.x - p1.x)
}

/// adapted from https://stackoverflow.com/a/9997374/3833068
#[inline]
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
    for (l1p1, l1p2) in &[(t1[0], t1[1]), (t1[1], t1[2]), (t1[2], t1[0])] {
        for (l2p1, l2p2) in &[(t2[0], t2[1]), (t2[1], t2[2]), (t2[2], t2[0])] {
            if check_line_seg_intersection(*l1p1, *l1p2, *l2p1, *l2p2) {
                return true;
            }
        }
    }

    false
}

struct TriangleCollisionVisitor<'a> {
    pub triangle: &'a TriangleBuf,
    pub triangle_bv: &'a AABB<f32>,
    pub triangles: &'a [TriangleBuf],
    pub does_collide: &'a mut bool,
    pub debug: bool,
}

impl<'a> BVTVisitor<usize, AABB<f32>> for TriangleCollisionVisitor<'a> {
    fn visit_internal(&mut self, bv: &AABB<f32>) -> bool {
        if *self.does_collide {
            return false;
        }

        self.triangle_bv.intersects(bv)
    }

    fn visit_leaf(&mut self, i: &usize, _bv: &AABB<f32>) {
        if *self.does_collide {
            return;
        }

        if self.debug {
            common::log(format!(
                "Checking collision: {:?} x {:?}",
                self.triangle, self.triangles[*i]
            ));
        }

        if check_triangle_collision(&self.triangle, &self.triangles[*i]) {
            *self.does_collide = true;
        }
    }
}

fn draw_bounding_box(bv: &AABB<f32>, color: &str, border_color: &str) {
    let (min, max) = (bv.mins(), bv.maxs());
    render_quad(
        min.x,
        min.y,
        max.x - min.x,
        max.y - min.y,
        color,
        border_color,
    );
}

struct BoundingBoxDebugVisitor;

impl BVTVisitor<usize, AABB<f32>> for BoundingBoxDebugVisitor {
    fn visit_internal(&mut self, bv: &AABB<f32>) -> bool {
        draw_bounding_box(bv, "rgba(13, 24, 230, 0.035)", "#2212BB");
        true
    }

    fn visit_leaf(&mut self, _i: &usize, bv: &AABB<f32>) {
        draw_bounding_box(bv, "rgba(230, 24, 80, 0.2)", "#BC1231");
    }
}

#[wasm_bindgen]
pub fn render(conf_str: &str) {
    // Clear the collision world, empty the geometry buf
    let world: &mut World = unsafe { &mut *COLLISION_WORLD };
    let triangles: &mut Vec<TriangleBuf> = unsafe { &mut *TRIANGLES };
    *world = DBVT::new();
    triangles.clear();
    let rng = unsafe { &mut *RNG };

    let Conf {
        prng_seed,
        canvas_width,
        canvas_height,
        triangle_size,
        triangle_count,
        max_rotation_rads,
        triangle_color,
        triangle_border_color,
        rotation_offset,
        debug_bounding_boxes,
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
        p2(-triangle_offset_x, triangle_offset_y),
        p2(triangle_offset_x, triangle_offset_y),
    ];

    *rng = Pcg32::from_seed(unsafe { mem::transmute((prng_seed, prng_seed)) });

    let mut last_triangle = [
        base_triangle_coords[0] + initial_offset,
        base_triangle_coords[1] + initial_offset,
        base_triangle_coords[2] + initial_offset,
    ];
    let mut rotation = 0.0;
    let mut drawn_triangles = 0;
    let mut placement_failures = 0;
    'outer: loop {
        // pick one of the other two vertices to use as the new origin
        let (ix, rot_offset) = if rng.gen_range(0, 2) == 0 {
            (1, deg_to_rad(rotation_offset))
        } else {
            (2, deg_to_rad(-rotation_offset))
        };

        let origin = last_triangle[ix];
        rotation += rot_offset;
        let mut i = 0;
        let mut bounding_box: AABB<f32>;
        let mut proposed_rotation;
        loop {
            proposed_rotation =
                rotation + rng.gen_range(-max_rotation_rads, max_rotation_rads + 0.00001);
            // determine if this proposed triangle would intersect any other triangle
            let proposed_isometry =
                Isometry2::new(Vector2::new(origin.x, origin.y), proposed_rotation);
            let proposed_triangle = [
                proposed_isometry * base_triangle_coords[0],
                proposed_isometry * base_triangle_coords[1],
                proposed_isometry * base_triangle_coords[2],
            ];
            let (min, max) = bounds(
                proposed_triangle[0],
                proposed_triangle[1],
                proposed_triangle[2],
            );
            bounding_box = AABB::new(min, max);

            let mut does_collide = false;
            let mut visitor = TriangleCollisionVisitor {
                triangle: &proposed_triangle,
                triangle_bv: &bounding_box,
                triangles: triangles,
                does_collide: &mut does_collide,
                debug: debug_bounding_boxes && (drawn_triangles + 1 == triangle_count),
            };
            world.visit(&mut visitor);
            if !does_collide {
                // we've found a valid triangle placement
                break;
            }

            i += 1;

            if i > PLACEMENT_ATTEMPTS {
                placement_failures += 1;
                if placement_failures > 1000 {
                    common::error("Failed to place a triangle in 1000 iterations; bailing out.");
                    return;
                }

                // we failed to place a triangle at this origin; we have to pick a new origin point.
                let ix = rng.gen_range(0, triangles.len());
                last_triangle = triangles[ix];
                continue 'outer;
            }
        }
        rotation = proposed_rotation;

        render_triangle_array(
            &base_triangle_coords,
            &mut last_triangle,
            Isometry2::new(Vector2::new(origin.x, origin.y), rotation),
            triangle_color,
            triangle_border_color,
        );

        triangles.push(last_triangle);
        let _leaf_id = world.insert(DBVTLeaf::new(bounding_box, triangles.len() - 1));

        drawn_triangles += 1;
        if drawn_triangles == triangle_count {
            break;
        }
        placement_failures = 0;
    }

    if debug_bounding_boxes {
        world.visit(&mut BoundingBoxDebugVisitor);
    }
}

#[test]
fn triangle_intersection() {
    let triangle1 = [
        p2(305.66763, 439.45938),
        p2(278.40073, 428.20035),
        p2(282.28357, 457.4437),
    ];
    let triangle2 = [
        p2(290.44968, 472.76297),
        p2(310.24722, 450.89273),
        p2(281.4083, 444.68268),
    ];

    assert!(check_triangle_collision(&triangle1, &triangle2));
}
