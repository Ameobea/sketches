#[derive(Deserialize)]
pub struct UserConf {
    // universe gen
    pub food_patch_count: usize,
    pub food_patch_size: usize,
    pub food_patch_size_variance: usize,
    pub barrier_patch_count: usize,
    pub barrier_patch_size: usize,
    // ant behavior
    pub wander_transition_chance_percent: f32,
    // environment
    pub pheremone_decay_interval: usize,
    pub pheremone_decay_amount: f32,
}

const fn default_user_conf() -> UserConf {
    UserConf {
        food_patch_count: 5,
        food_patch_size: 16,
        food_patch_size_variance: 3,
        barrier_patch_count: 6,
        barrier_patch_size: 28,
        wander_transition_chance_percent: 4.5,
        pheremone_decay_interval: 10000,
        pheremone_decay_amount: 0.9,
    }
}

#[thread_local]
pub static mut ACTIVE_USER_CONF: UserConf = default_user_conf();

pub fn active_conf() -> &'static UserConf { unsafe { &ACTIVE_USER_CONF } }
