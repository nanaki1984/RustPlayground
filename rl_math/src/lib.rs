use nalgebra_glm::Vec3;

pub const VEC3_ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const VEC3_ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
pub const VEC3_HALF: Vec3 = Vec3::new(0.5, 0.5, 0.5);

mod aabb;

pub use aabb::AABB;
