use nalgebra_glm::{Mat3x4};

use rl_core::Array;
use rl_math::AABB;
use crate::SDFShape;

struct SDFSceneEntry {
    shape: SDFShape,

    xform: Mat3x4,
    inv_xform: Mat3x4,
    aabb: AABB,

    group_id: u32,

    distance_scaling_factor: f32,
}

pub struct SDFScene {
    entries: Array<SDFSceneEntry>,
}
/*
impl SDFScene {
    fn create
}
*/