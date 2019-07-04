use std::{mem, ptr};

#[derive(Deserialize)]
pub struct Conf {

}

#[thread_local]
pub static mut ACTIVE_USER_CONF: &'static Conf = unsafe { mem::transmute(420usize) };

pub fn active_conf() -> &'static Conf { unsafe { ACTIVE_USER_CONF } }

pub fn set_active_conf(new_conf: Conf) {
    // Free the old conf if it exists
    if !(active_conf() as *const Conf).is_null() {
        let old_conf = unsafe { Box::from_raw(active_conf() as *const Conf as *mut Conf) };
        drop(old_conf);
    }

    let new_conf = Box::new(new_conf);
    unsafe { ACTIVE_USER_CONF = &mut *Box::into_raw(new_conf) };
}
