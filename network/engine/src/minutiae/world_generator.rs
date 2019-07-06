use minutiae::{prelude::*, universe::Universe2DConf, util};
use noise::{Billow, NoiseFn, Seedable};
use rand::Rng;
use rand_distr::{Distribution, Exp};
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
            UNIVERSE_SIZE as usize * UNIVERSE_SIZE as usize
        ];

        let mut entities = Vec::with_capacity(UNIVERSE_SIZE as usize * UNIVERSE_SIZE as usize);
        for _ in 0..(UNIVERSE_SIZE * UNIVERSE_SIZE) {
            entities.push(Vec::new());
        }

        let cutoff = 0.55;
        let scale_factor = 0.01;
        let multiplier = 100_000.0;
        let gen_iters = 18;
        let initial_charge = 100_000_000.;

        for iter in 0..gen_iters {
            info!("iter: {}", iter);
            // Generate some noise.  For each cell that exceeds some bound, we create an entity there if
            // one doesn't exist.  If it does, we increase its values.
            let mut noise_fn = Billow::new();
            noise_fn.octaves = 5;
            noise_fn.frequency = 1.0;
            noise_fn.lacunarity = 1.5;
            noise_fn.persistence = 0.36;
            noise_fn = noise_fn.set_seed(rng().gen_range(0, 4_000_000_000));

            let distr = Exp::new(1.298342).unwrap();
            for i in 0..(UNIVERSE_SIZE as usize * UNIVERSE_SIZE as usize) {
                let (x, y) = util::get_coords(i, UNIVERSE_SIZE as usize);
                let val: f64 = (NoiseFn::get(
                    &noise_fn,
                    [
                        x as f64 * scale_factor + (iter as f64 * 1000.),
                        y as f64 * scale_factor + (iter as f64 * 1000.),
                    ],
                ) + 1.)
                    / 2.;

                if val >= cutoff {
                    if entities[i].is_empty() {
                        let activation_threshold = distr.sample(rng()) * multiplier;
                        entities[i].push(Entity::new(
                            NetworkEntityState::Neuron {
                                activation_threshold,
                                charge: 0.0,
                                resistances: Directions {
                                    up: rng().gen_range(0., 0.7) * val as f32,
                                    down: rng().gen_range(0., 0.7) * val as f32,
                                    left: rng().gen_range(0., 0.7) * val as f32,
                                    right: rng().gen_range(0., 0.7) * val as f32,
                                },
                            },
                            NetworkMutEntityState {},
                        ));
                    } else {
                        match &mut entities[i][0].state {
                            NetworkEntityState::Neuron {
                                ref mut activation_threshold,
                                ..
                            } => *activation_threshold += distr.sample(rng()) * multiplier,
                        }
                    }
                }
            }
        }

        // Pick a random cell(s) to give some initial charge to
        for _ in 0..10 {
            loop {
                let universe_ix = rng().gen_range(0, UNIVERSE_SIZE * UNIVERSE_SIZE) as usize;
                let entities_for_coord = &mut entities[universe_ix];
                if entities_for_coord.is_empty() {
                    continue;
                }
                info!(
                    "Setting entity at {:?} as that with initial change",
                    get_coords(universe_ix, UNIVERSE_SIZE as usize)
                );

                match entities_for_coord[0].state {
                    NetworkEntityState::Neuron { ref mut charge, .. } => *charge = initial_charge,
                };
                break;
            }
        }

        (cells, entities)
    }
}
