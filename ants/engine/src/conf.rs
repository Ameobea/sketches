#[derive(Deserialize)]
pub struct UserConf {
    // universe gen
    pub food_patch_count: usize,
    pub food_patch_size: usize,
    pub food_patch_size_variance: usize,
    pub food_patch_capacity: usize,
    pub barrier_patch_count: usize,
    pub barrier_patch_size: usize,
    // ant behavior
    pub wander_transition_chance_percent: f32,
    // environment
    pub pheremone_decay_interval: f32,
    pub pheremone_decay_multiplier: f32,
}

const fn default_user_conf() -> UserConf {
    UserConf {
        food_patch_count: 200,
        food_patch_size: 20,
        food_patch_size_variance: 3,
        food_patch_capacity: 5,
        barrier_patch_count: 44,
        barrier_patch_size: 128,
        wander_transition_chance_percent: 4.25,
        pheremone_decay_interval: 250.0,
        pheremone_decay_multiplier: 0.8,
    }
}

#[thread_local]
pub static mut ACTIVE_USER_CONF: UserConf = default_user_conf();

pub fn active_conf() -> &'static UserConf { unsafe { &ACTIVE_USER_CONF } }
