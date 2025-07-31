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

/// type alias for Arc<Mutex<Box<T>>> bc i really can't be bothered to write that every time
pub type SharedBox<T> = Arc<Mutex<Box<T>>>;

pub fn new_shared_box<T>(t: T) -> SharedBox<T> {
    Arc::new(Mutex::new(Box::new(t)))
}

/// type alias for Arc<Mutex<T>> bc i can't be bothered to write that every time
pub type Shared<T> = Arc<Mutex<T>>;

pub fn new_shared<T>(t: T) -> Shared<T> {
    Arc::new(Mutex::new(t))
}

pub type WeakShared<T> = std::sync::Weak<Mutex<T>>;

/// helper trait for turning glam types into cgmath types
pub trait IntoCgmath {
    type Output;
    fn into_cgmath(self) -> Self::Output;
}

impl IntoCgmath for glam::Vec3 {
    type Output = cgmath::Vector3<f32>;

    fn into_cgmath(self) -> Self::Output {
        cgmath::Vector3::new(self.x, self.y, self.z)
    }
}

impl IntoCgmath for glam::Vec2 {
    type Output = cgmath::Vector2<f32>;

    fn into_cgmath(self) -> Self::Output {
        cgmath::Vector2::new(self.x, self.y)
    }
}

impl IntoCgmath for glam::Mat4 {
    type Output = cgmath::Matrix4<f32>;
    fn into_cgmath(self) -> Self::Output {
        cgmath::Matrix4::new(
            self.x_axis.x,
            self.x_axis.y,
            self.x_axis.z,
            self.x_axis.w,
            self.y_axis.x,
            self.y_axis.y,
            self.y_axis.z,
            self.y_axis.w,
            self.z_axis.x,
            self.z_axis.y,
            self.z_axis.z,
            self.z_axis.w,
            self.w_axis.x,
            self.w_axis.y,
            self.w_axis.z,
            self.w_axis.w,
        )
    }
}

impl IntoCgmath for glam::Quat {
    type Output = cgmath::Quaternion<f32>;
    fn into_cgmath(self) -> Self::Output {
        cgmath::Quaternion::new(self.w, self.x, self.y, self.z)
    }
}
