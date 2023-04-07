use nalgebra_glm::{Vec3, Vec4, Mat4x3};

pub const VEC3_ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const VEC3_ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
pub const VEC3_HALF: Vec3 = Vec3::new(0.5, 0.5, 0.5);

mod aabb;

pub use aabb::AABB;

// TODO: make a Transform struct with Vec3 Location Quaternion Rotation Vec3 Scale because using glm directly is unbearable

pub fn transform_vec4(m: &Mat4x3, v: &Vec4) -> Vec3 {
    Vec3::new(m.column(0).dot(v), m.column(1).dot(v), m.column(2).dot(v))
}

pub fn transform_vector(xform: &Mat4x3, vector: &Vec3) -> Vec3 {
    transform_vec4(xform, &Vec4::new(vector.x, vector.y, vector.z, 0.0))
}

pub fn transform_point(xform: &Mat4x3, point: &Vec3) -> Vec3 {
    transform_vec4(xform, &Vec4::new(point.x, point.y, point.z, 1.0))
}
