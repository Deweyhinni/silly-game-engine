use std::{
    f64::consts::PI,
    sync::{Arc, Mutex},
};

/// degrees to radians
pub const fn deg_to_rad(deg: f64) -> f64 {
    deg * (PI / 180_f64)
}

/// radians to degrees
pub const fn rad_to_deg(rad: f64) -> f64 {
    rad * (180_f64 / PI)
}

pub type SharedBox<T> = Arc<Mutex<Box<T>>>;

pub fn new_shared_box<T>(i: T) -> SharedBox<T> {
    Arc::new(Mutex::new(Box::new(i)))
}
