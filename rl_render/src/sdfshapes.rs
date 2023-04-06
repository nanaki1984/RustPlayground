use nalgebra_glm::{Vec3, Mat3x4, Mat4x4, inverse};
use rl_core::Array;
use rl_math::{AABB, VEC3_ZERO, VEC3_ONE, VEC3_HALF};

#[derive(Clone)]
pub enum SDFShape {
    Sphere{ radius: f32 },
    Box{ half_size: Vec3 },
    RoundedBox{ half_size: Vec3, radius: f32 },
}

impl SDFShape {
    #[inline]
    pub fn get_local_aabb(&self) -> AABB {
        match &self {
            SDFShape::Sphere{ radius } => {
                AABB::from_center_extents(&VEC3_ZERO, &(VEC3_ONE * (*radius)))
            }
            SDFShape::Box{ half_size } => {
                AABB::from_center_extents(&VEC3_ZERO, half_size)
            }
            SDFShape::RoundedBox{ half_size, radius } => {
                AABB::from_center_extents(&VEC3_ZERO, &(half_size + VEC3_ONE * (*radius)))
            }
        }
    }
}

#[derive(Clone)]
pub struct SDFPrimitive {
    shape: SDFShape,

    xform: Mat3x4, // TODO: this is redundant, I could remove it
    inv_xform: Mat3x4,
    aabb: AABB,

    distance_scaling_factor: f32,

    group_id: usize,
}

impl SDFPrimitive {
    pub fn new(shape: SDFShape, transform: &Mat4x4, group_id: usize) -> Self {
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

#[derive(Default)]
pub struct SDFPrimitivesList {
    primitives: Array<SDFPrimitive>,
}

impl SDFPrimitivesList {
    pub fn add(&mut self, shape: SDFShape, transform: &Mat4x4, group_id: usize) {
        self.primitives.push_back(SDFPrimitive::new(shape, transform, group_id));
    }

    pub fn cull(&self, aabb: &AABB) -> SDFPrimitivesList {
        // TODO: parallelize this and make it possible to reuse already allocated arrays?
        let culled_primitives = self.primitives
            .iter()
            .filter(|&primitive| { primitive.aabb.intersects(&aabb) })
            .collect();

        Self {
            primitives: culled_primitives,
        }
    }

    pub fn sort_by_group_id(&mut self) {
        self.primitives.sort_by(|prim0, prim1| { prim0.group_id.cmp(&prim1.group_id) });
    }

    pub fn clear(&mut self) {
        self.primitives.clear()
    }
}
