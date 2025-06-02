use std::f64::consts::PI;

/// degrees to radians
pub const fn deg_to_rad(deg: f64) -> f64 {
    deg * (PI / 180_f64)
}

/// radians to degrees
pub const fn rad_to_deg(rad: f64) -> f64 {
    rad * (180_f64 / PI)
}
