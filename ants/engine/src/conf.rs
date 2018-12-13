#[derive(Deserialize)]
pub struct UserConf {
    // universe gen
    pub ant_count: f32,
    pub food_patch_count: usize,
    pub food_patch_size: usize,
    pub food_patch_size_variance: usize,
    pub food_patch_capacity: usize,
    pub barrier_patch_count: usize,
    pub barrier_patch_size: usize,
    // ant behavior
    pub wander_transition_chance_percent: f32,
    pub anthill_attraction_pos_bias: f32,
    pub anthill_attraction_neg_bias: f32,
    pub anthill_attraction_distrance_multiplier: f32,
    pub returning_maintain_pos_bias: f32,
    pub returning_maintain_neg_bias: f32,
    pub returning_wander_threshold: f32,
    pub following_pos_bias: f32,
    pub following_neg_bias: f32,
    pub base_wandering_weight: f32,
    pub base_following_weight: f32,
    pub base_returning_weight: f32,
    pub scout_pursuit_cutoff: f32,
    // environment
    pub pheremone_decay_interval: f32,
    pub pheremone_decay_multiplier: f32,
    pub pheromone_max: f32,
}

const fn default_user_conf() -> UserConf {
    UserConf {
        ant_count: 80.0,
        food_patch_count: 248,
        food_patch_size: 27,
        food_patch_size_variance: 3,
        food_patch_capacity: 5,
        barrier_patch_count: 44,
        barrier_patch_size: 128,
        wander_transition_chance_percent: 4.25,
        anthill_attraction_pos_bias: 1.2,
        anthill_attraction_neg_bias: 0.9,
        anthill_attraction_distrance_multiplier: 0.2,
        returning_maintain_pos_bias: 2.5,
        returning_maintain_neg_bias: 0.3,
        returning_wander_threshold: 1.0,
        following_pos_bias: 2.0,
        following_neg_bias: 0.5,
        base_wandering_weight: 1.0,
        base_returning_weight: 1.0,
        scout_pursuit_cutoff: 3.0,
        base_following_weight: 1.0,
        pheremone_decay_interval: 250.0,
        pheremone_decay_multiplier: 0.8,
        pheromone_max: 15.0,
    }
}

#[thread_local]
pub static mut ACTIVE_USER_CONF: UserConf = default_user_conf();

pub fn active_conf() -> &'static UserConf { unsafe { &ACTIVE_USER_CONF } }
