mod sdfshapes;

pub use sdfshapes::SDFShape;
pub use sdfshapes::SDFPrimitive;
pub use sdfshapes::SDFPrimitivesList;

mod globalsdf;

pub use globalsdf::GlobalSDFCascade;

pub mod cs_globalsdf {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "./src/shaders/globalsdf_write_chunk.glsl",
        define: [
            ("GROUP_SIZE", "4")
        ],
        //dump: true,
    }
}

impl From<&SDFPrimitive> for cs_globalsdf::ty::SDFPrimitive {
    fn from(value: &SDFPrimitive) -> Self {
        let (half_size_radius, shape) = match value.get_shape() {
            SDFShape::Sphere { radius } => {
                ([*radius, *radius, *radius, *radius], 0u32)
            },
            SDFShape::Box { half_size } => {
                ([half_size.x, half_size.y, half_size.z, 0.0], 1u32)
            },
            SDFShape::RoundedBox { half_size, radius } => {
                ([half_size.x, half_size.y, half_size.z, *radius], 2u32)
            }
        };
        
        Self {
            half_size_radius,
            inv_xform: *value.get_inv_xform().as_ref(),
            distance_scaling_factor: value.get_dist_scaling_factor(),
            shape,
            group_id: value.get_group_id(),
            _dummy0: Default::default()
        }
    }
}

//mod sdfscene;
