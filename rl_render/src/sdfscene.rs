use nalgebra_glm::{Mat3x4, Mat4x4, inverse};

use rl_core::Array;
use rl_math::{AABB, VEC3_ZERO, VEC3_HALF};
use crate::SDFShape;

struct SDFSceneEntry {
    shape: SDFShape,

    xform: Mat3x4,
    inv_xform: Mat3x4,
    aabb: AABB,

    distance_scaling_factor: f32,

    group_id: usize,
}

impl SDFSceneEntry {
    fn new(shape: SDFShape, transform: &Mat4x4, group_id: usize) -> Self {
        let inv_transform = inverse(transform);
        let xform = Mat3x4::from_rows(
            &[transform.row(0)
            , transform.row(1)
            , transform.row(2)]);
        let inv_xform = Mat3x4::from_rows(
            &[inv_transform.row(0)
            , inv_transform.row(1)
            , inv_transform.row(2)]);

        let aabb = shape.get_local_aabb().transform(&xform);

        let inv_dist_scaling_factor = AABB::from_center_extents(&VEC3_ZERO, &VEC3_HALF)
            .transform(&xform)
            .size()
            .min();
        let distance_scaling_factor = 1.0 / inv_dist_scaling_factor;

        Self {
            shape,

            xform,
            inv_xform,
            aabb,

            distance_scaling_factor,

            group_id,
        }
    }
}

pub struct SDFScene {
    entries: Array<SDFSceneEntry>,
}

impl SDFScene {
    fn add(&mut self, shape: SDFShape, transform: &Mat4x4, group_id: usize) {
        self.entries.push_back(SDFSceneEntry::new(shape, transform, group_id));
    }
}
