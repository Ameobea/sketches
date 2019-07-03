use minutiae::{prelude::*, universe::Universe2DConf, util};
use noise::{Billow, NoiseFn, Point2, Seedable};
use rand::Rng;
use rand_distr::{Distribution, Gamma};
use sketches_util::rng;

use super::*;

pub struct NetworkUniverseGenerator;

impl Generator<NetworkCellState, NetworkEntityState, NetworkMutEntityState>
    for NetworkUniverseGenerator
{
    fn gen(
        &mut self,
        _conf: &Universe2DConf,
    ) -> (
        Vec<Cell<NetworkCellState>>,
        Vec<Vec<Entity<NetworkCellState, NetworkEntityState, NetworkMutEntityState>>>,
    ) {
        let cells = vec![
            Cell {
                state: NetworkCellState::Empty
            };
            UNIVERSE_SIZE as usize
        ];

        let mut entities = Vec::with_capacity(UNIVERSE_SIZE as usize);
        for _ in 0..UNIVERSE_SIZE {
            entities.push(Vec::new());
        }

        for _ in 0..10 {
            // Generate some noise.  For each cell that exceeds some bound, we create an entity there if
            // one doesn't exist.  If it does, we increase its values.
            let mut noise_fn = Billow::new();
            noise_fn.octaves = 5;
            noise_fn.frequency = 3.0;
            noise_fn.lacunarity = 1.5;
            noise_fn.persistence = 0.36;
            noise_fn = noise_fn.set_seed(rng().gen());

            let cutoff = 0.8;
            let distr = Gamma::new(2.0, 5.0).unwrap();
            for i in 0..(UNIVERSE_SIZE as usize) {
                let (x, y) = util::get_coords(i, UNIVERSE_SIZE as usize);
                let val: f64 = NoiseFn::get(&noise_fn, [x as f64, y as f64]);

                if val >= cutoff {
                    if entities[i].is_empty() {
                        entities[i].push(Entity::new(
                            NetworkEntityState::Neuron {
                                activation_threshold: rng()
                                    .gen_range(0., distr.sample(rng()) * 200_000.0),
                                charge: 0.0,
                                resistances: Directions {
                                    up: rng().gen_range(0., 1.) * val as f32,
                                    down: rng().gen_range(0., 1.) * val as f32,
                                    left: rng().gen_range(0., 1.) * val as f32,
                                    right: rng().gen_range(0., 1.) * val as f32,
                                },
                            },
                            NetworkMutEntityState {},
                        ));
                    } else {
                        match &mut entities[i][0].state {
                            NetworkEntityState::Neuron {
                                ref mut activation_threshold,
                                ..
                            } => {
                                *activation_threshold +=
                                    rng().gen_range(0., distr.sample(rng()) * 50_000.0)
                            }
                        }
                    }
                }
            }
        }

        (cells, entities)
    }
}
