use nalgebra_glm::Vec3;
use rl_math::{AABB, VEC3_ZERO, VEC3_ONE};

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
