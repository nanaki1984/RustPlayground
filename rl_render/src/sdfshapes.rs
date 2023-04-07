use nalgebra_glm::{Vec3, Mat4x3, Mat4x4, inverse};
use rl_core::Array;
use rl_math::{AABB, VEC3_ZERO, VEC3_ONE, VEC3_HALF};
use crate::cs_globalsdf::SDFPrimitive as SDFPrimitiveGPU;

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

    inv_xform: Mat4x3,
    aabb: AABB,

    distance_scaling_factor: f32,

    group_id: u32,
}

impl SDFPrimitive {
    pub fn new(shape: SDFShape, transform: &Mat4x4, group_id: u32) -> Self {
        let inv_transform = inverse(transform);
        let xform = Mat4x3::from_columns(
            &[transform.column(0)
            , transform.column(1)
            , transform.column(2)]);
        let inv_xform = Mat4x3::from_columns(
            &[inv_transform.column(0)
            , inv_transform.column(1)
            , inv_transform.column(2)]);

        let aabb = shape.get_local_aabb().transform(&xform);

        let inv_dist_scaling_factor = AABB::from_center_extents(&VEC3_ZERO, &VEC3_HALF)
            .transform(&xform)
            .size()
            .min();
        let distance_scaling_factor = 1.0 / inv_dist_scaling_factor;

        Self {
            shape,

            inv_xform,
            aabb,

            distance_scaling_factor,

            group_id,
        }
    }

    pub fn get_shape(&self) -> &SDFShape {
        &self.shape
    }

    pub fn get_inv_xform(&self) -> &Mat4x3 {
        &self.inv_xform
    }

    pub fn get_dist_scaling_factor(&self) -> f32 {
        self.distance_scaling_factor
    }

    pub fn get_group_id(&self) -> u32 {
        self.group_id
    }
}

#[derive(Default)]
pub struct SDFPrimitivesList {
    primitives: Array<SDFPrimitive>,
}

impl SDFPrimitivesList {
    pub fn add(&mut self, shape: SDFShape, transform: &Mat4x4, group_id: u32) {
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

    pub fn clear(&mut self) {
        self.primitives.clear()
    }

    pub fn sort_by_group_id(&mut self) {
        self.primitives.sort_by(|prim0, prim1| { prim0.group_id.cmp(&prim1.group_id) });
    }

    pub fn send_to_gpu(&self) -> Array<SDFPrimitiveGPU> { // Do not return an array, return an IntoIter that converts every SDFPrimitive in SDFPrimitiveGPU
        self.primitives
            .iter()
            .map(|prim| -> SDFPrimitiveGPU { prim.into() })
            .collect()
    }
}
