use nalgebra_glm::{Vec3, Vec4, Mat3x4, min2, max2};//, all, less_than_equal, greater_than_equal};

use crate::VEC3_ONE;

pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new() -> AABB {
        AABB {
            min: VEC3_ONE * f32::MAX,
            max: VEC3_ONE * f32::MIN,
        }
    }

    #[inline]
    pub const fn from_min_max(min: Vec3, max: Vec3) -> AABB {
        Self {
            min,
            max,
        }
    }

    #[inline]
    pub fn from_center_extents(center: &Vec3, extents: &Vec3) -> AABB {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    #[inline]
    pub fn intersects(&self, other: &AABB) -> bool {
        //all(&less_than_equal(&self.min, &other.max)) && all(&greater_than_equal(&self.max, &other.min))
        self.min <= other.max && self.max >= other.min
    }

    #[inline]
    pub fn encapsulate_point(&mut self, point: &Vec3) {
        self.min = min2(&self.min, point);
        self.max = max2(&self.max, point);
    }

    #[inline]
    pub fn encapsulate_aabb(&mut self, aabb: &AABB) {
        self.min = min2(&self.min, &aabb.min);
        self.max = max2(&self.max, &aabb.max);
    }

    #[inline]
    pub fn expand(&mut self, amount: f32) {
        self.min -= VEC3_ONE * amount;
        self.max += VEC3_ONE * amount;
    }

    #[inline]
    pub fn transform(&self, xform: &Mat3x4) -> AABB {
        let mut new_aabb = Self::new();
        for i in 0..7 {
            new_aabb.encapsulate_point(&(xform * Vec4::new(
                if 1 == (i & 1) { self.min.x } else { self.max.x },
                if 2 == (i & 2) { self.min.y } else { self.max.y },
                if 4 == (i & 4) { self.min.z } else { self.max.z },
                1.0)));
        }
        new_aabb
    }
}
