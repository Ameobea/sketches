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
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVTLeafId, DBVT};
use rand::Rng;
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./index")]
extern "C" {
    #[allow(clippy::too_many_arguments)]
    pub fn render_triangle(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &str,
        border_color: &str,
    ) -> usize;
    pub fn render_quad(x: f32, y: f32, width: f32, height: f32, color: &str, border_color: &str);
    pub fn delete_elem(elem_id: usize);
}

type TriangleBuf = [Point2<f32>; 3];

const PLACEMENT_ATTEMPTS: usize = 3;
const PLACEMENT_BAILOUT_THRESHOLD: usize = 1000;

#[derive(Deserialize)]
pub struct Conf {
    pub prng_seed: f64,
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub triangle_size: f32,
    pub triangle_count: usize,
    pub max_rotation_rads: f32,
    pub triangle_color: String,
    pub triangle_border_color: String,
    pub rotation_offset: f32,
    pub debug_bounding_boxes: bool,
    pub generation_rate: f32,
}

impl Conf {
    /// Returns `(offset_x, offset_y)`
    fn get_base_triangle_offsets(&self) -> (f32, f32) {
        let triangle_offset_x = self.triangle_size / 2.0;
        (
            triangle_offset_x,
            ((self.triangle_size * self.triangle_size) - (triangle_offset_x * triangle_offset_x))
                .sqrt(),
        )
    }
}

struct Env {
    pub conf: Conf,
    pub triangle_offset_x: f32,
    pub triangle_offset_y: f32,
    pub base_triangle_coords: TriangleBuf,
    pub last_triangle: TriangleBuf,
    pub last_triangle_ix: usize,
    pub rotation: f32,
    pub oldest_triangle_ix: usize,
}

impl Env {
    pub fn parse_from_str(s: &str) -> Result<Self, String> {
        serde_json::from_str(s)
            .map_err(|err| format!("Error decoding provided conf object: {:?}", err))
            .map(Self::new)
    }

    pub fn new(conf: Conf) -> Self {
        let (triangle_offset_x, triangle_offset_y) = conf.get_base_triangle_offsets();
        // Clear out the collision world, empty the geometry buffer, + reseed PRNG
        let world = unsafe { &mut *COLLISION_WORLD };
        let triangles = unsafe { &mut *TRIANGLES };
        *world = DBVT::new();
        triangles.clear();
        *rng() = Pcg32::from_seed(unsafe { mem::transmute((conf.prng_seed, conf.prng_seed)) });

        let base_triangle_coords = [
            Point2::origin(),
            p2(-triangle_offset_x, triangle_offset_y),
            p2(triangle_offset_x, triangle_offset_y),
        ];
        let initial_offset = Vector2::new(
            conf.canvas_width as f32 / 2.0,
            conf.canvas_height as f32 / 2.0,
        );
        let last_triangle = [
            base_triangle_coords[0] + initial_offset,
            base_triangle_coords[1] + initial_offset,
            base_triangle_coords[2] + initial_offset,
        ];
        let rotation = 0.0;

        Env {
            conf,
            triangle_offset_x,
            triangle_offset_y,
            base_triangle_coords,
            last_triangle_ix: 0,
            last_triangle,
            rotation,
            oldest_triangle_ix: 0,
        }
    }

    pub fn set_new_last_triangle(&mut self) {
        let ix = rng().gen_range(0, triangles().len());
        self.last_triangle = triangles()[ix].geometry;
        self.last_triangle_ix = ix;
    }
}

#[derive(Debug)]
struct TriangleHandle {
    pub geometry: TriangleBuf,
    pub collider_handle: DBVTLeafId,
    pub dom_id: usize,
    pub prev_node: Option<usize>,
    pub next_node_1: Option<usize>,
    pub next_node_2: Option<usize>,
}

impl TriangleHandle {
    pub fn degree(&self) -> usize {
        let mut degree = 0;
        for link in &[self.prev_node, self.next_node_1, self.next_node_2] {
            if link.is_some() {
                degree += 1;
            }
        }
        degree
    }
}

#[inline(always)]
fn p2(x: f32, y: f32) -> Point2<f32> {
    Point2::new(x, y)
}

#[inline]
fn render_triangle_array(triangle: &TriangleBuf, color: &str, border_color: &str) -> usize {
    let [p1, p2, p3] = *triangle;
    render_triangle(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, color, border_color)
}

type World = DBVT<f32, usize, AABB<f32>>;
static mut COLLISION_WORLD: *mut World = ptr::null_mut();
static mut TRIANGLES: *mut Vec<TriangleHandle> = ptr::null_mut();
static mut RNG: *mut Pcg32 = ptr::null_mut();
static mut ENV: *mut Env = ptr::null_mut();

#[inline(always)]
fn rng() -> &'static mut Pcg32 {
    unsafe { &mut *RNG }
}

#[inline(always)]
fn triangles() -> &'static mut Vec<TriangleHandle> {
    unsafe { &mut *TRIANGLES }
}

#[inline(always)]
fn world() -> &'static mut World {
    unsafe { &mut *COLLISION_WORLD }
}

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook();

    let world: Box<World> = box DBVT::new();
    let p: *mut World = Box::into_raw(world);
    unsafe { COLLISION_WORLD = p };

    let triangles: Box<Vec<TriangleHandle>> = box Vec::with_capacity(200);
    let p: *mut Vec<TriangleHandle> = Box::into_raw(triangles);
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
    pub triangles: &'a [TriangleHandle],
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

        if check_triangle_collision(&self.triangle, &self.triangles[*i].geometry) {
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

/// Attempts to find a valid rotation for the next triangle, returning the proposed triangle if it
/// is found.
fn find_triangle_placement(
    env: &Env,
    origin: Point2<f32>,
    rotation: f32,
    i: usize,
) -> Option<(AABB<f32>, TriangleBuf)> {
    let Env {
        conf:
            Conf {
                max_rotation_rads,
                debug_bounding_boxes,
                triangle_count,
                ..
            },
        base_triangle_coords,
        ..
    } = env;

    let proposed_rotation =
        rotation + rng().gen_range(-*max_rotation_rads, *max_rotation_rads + 0.00001);
    // determine if this proposed triangle would intersect any other triangle
    let proposed_isometry = Isometry2::new(Vector2::new(origin.x, origin.y), proposed_rotation);
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
    let bounding_box = AABB::new(min, max);

    let mut does_collide = false;
    let mut visitor = TriangleCollisionVisitor {
        triangle: &proposed_triangle,
        triangle_bv: &bounding_box,
        triangles: triangles(),
        does_collide: &mut does_collide,
        debug: *debug_bounding_boxes && (i + 1 == *triangle_count),
    };
    world().visit(&mut visitor);

    if !does_collide {
        // we've found a valid triangle placement
        Some((bounding_box, proposed_triangle))
    } else {
        None
    }
}

fn generate_triangle(env: &mut Env, i: usize) -> Option<(AABB<f32>, TriangleBuf)> {
    // pick one of the other two vertices to use as the new origin
    let (ix, rot_offset) = if rng().gen_range(0, 2) == 0 {
        (1, deg_to_rad(env.conf.rotation_offset))
    } else {
        (2, deg_to_rad(-env.conf.rotation_offset))
    };

    let origin = env.last_triangle[ix];
    for _ in 0..PLACEMENT_ATTEMPTS {
        if let Some((bv, triangle)) =
            find_triangle_placement(env, origin, env.rotation + rot_offset, i)
        {
            env.rotation += rot_offset;
            return Some((bv, triangle));
        }
    }

    None // failed to place a triangle at this origin in `PLACEMENT_ATTTEMPTS` attempts
}

fn place_triangle(env: &mut Env, i: usize, triangle_arr_ix: Option<usize>) -> Result<(), ()> {
    for _ in 0..PLACEMENT_BAILOUT_THRESHOLD {
        if let Some((bv, triangle)) = generate_triangle(env, i) {
            let dom_id = render_triangle_array(
                &triangle,
                &env.conf.triangle_color,
                &env.conf.triangle_border_color,
            );
            let insertion_ix = triangle_arr_ix.unwrap_or_else(|| triangles().len());
            let leaf_id = world().insert(DBVTLeaf::new(bv, insertion_ix));

            let handle = TriangleHandle {
                dom_id,
                collider_handle: leaf_id,
                geometry: triangle,
                prev_node: Some(env.last_triangle_ix),
                next_node_1: None,
                next_node_2: None,
            };
            if let Some(i) = triangle_arr_ix {
                triangles()[i] = handle;
            } else {
                triangles().push(handle);
            }

            env.last_triangle = triangle;
            match (
                triangles()[env.last_triangle_ix].next_node_1,
                triangles()[env.last_triangle_ix].next_node_2,
            ) {
                (Some(_), None) => {
                    triangles()[env.last_triangle_ix].next_node_2 = Some(insertion_ix)
                }
                (None, Some(_)) | (None, None) => {
                    triangles()[env.last_triangle_ix].next_node_1 = Some(insertion_ix)
                }
                (Some(_), Some(_)) => {
                    panic!("Tried to add new triangle to triangle with two children")
                }
            }
            env.last_triangle_ix = insertion_ix;
            return Ok(());
        }

        // we failed to place a triangle at this origin; we have to pick a new origin point.
        env.set_new_last_triangle();
    }

    common::error(format!(
        "Failed to place a triangle in {} iterations; bailing out.",
        PLACEMENT_BAILOUT_THRESHOLD,
    ));
    Err(())
}

#[wasm_bindgen]
pub fn render(conf_str: &str) {
    let mut env = match Env::parse_from_str(conf_str) {
        Ok(env) => env,
        Err(err) => {
            common::error(err);
            return;
        }
    };

    // place `triangle_count` triangles
    'triangle_generator: for i in 0..env.conf.triangle_count {
        // give up placing any more triangles after `PLACEMENT_BAILOUT_THRESHOLD` placement attempts
        match place_triangle(&mut env, i, None) {
            Ok(()) => {
                continue 'triangle_generator;
            }
            Err(()) => break,
        }
    }

    if env.conf.debug_bounding_boxes {
        world().visit(&mut BoundingBoxDebugVisitor);
    }

    if unsafe { !ENV.is_null() } {
        // Drop the old conf to avoid leaking memory from the allocated strings
        let old_env = unsafe { Box::from_raw(ENV) };
        drop(old_env);
    }
    let new_env = box env;
    unsafe { ENV = Box::into_raw(new_env) };
}

/// Delete the oldest generated triangle and generate a new triangle.
#[wasm_bindgen]
pub fn generate() {
    let env: &mut Env = unsafe { &mut *ENV };

    let mut oldest_triangle = &triangles()[env.oldest_triangle_ix];
    let mut reset_count = 0;
    while oldest_triangle.geometry[0] == env.last_triangle[0] {
        env.set_new_last_triangle();
        oldest_triangle = &triangles()[env.oldest_triangle_ix];
        reset_count += 1;
        if reset_count == triangles().len() * 16 {
            common::error("Unable to find a triangle to use as a base; bailing out.");
            return;
        }
    }
    let triangle_valid = oldest_triangle.degree() == 1;
    if triangle_valid {
        delete_elem(oldest_triangle.dom_id);
        world().remove(oldest_triangle.collider_handle);
        if let Some(prev_ix) = oldest_triangle.prev_node {
            if triangles()[prev_ix].next_node_1 == Some(env.oldest_triangle_ix) {
                triangles()[prev_ix].next_node_1 = None;
            } else if triangles()[prev_ix].next_node_2 == Some(env.oldest_triangle_ix) {
                triangles()[prev_ix].next_node_2 = None;
            } else {
                panic!("Tried to delete triangle but its parent doesn't list it as its child");
            }
        }
        if let Some(child_ix) = oldest_triangle.next_node_1 {
            debug_assert!(triangles()[child_ix].prev_node == Some(env.oldest_triangle_ix));
            triangles()[child_ix].prev_node = None;
        }
        if let Some(child_ix) = oldest_triangle.next_node_2 {
            debug_assert!(triangles()[child_ix].prev_node == Some(env.oldest_triangle_ix));
            triangles()[child_ix].prev_node = None;
        }

        match place_triangle(env, env.conf.triangle_count, Some(env.oldest_triangle_ix)) {
            Ok(()) => (),
            Err(()) => {
                return;
            }
        };
    }

    if env.oldest_triangle_ix != env.conf.triangle_count - 1 {
        env.oldest_triangle_ix += 1;
    } else {
        env.oldest_triangle_ix = 0;
    }

    if !triangle_valid {
        generate();
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
