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
/*
impl Into<cs_globalsdf::ty::SDFPrimitive> for SDFPrimitive {
    fn into(self) -> cs_globalsdf::ty::SDFPrimitive {
        let half_size_radius = match self.get_shape() {
            SDFShape::Sphere { radius } => {
                [radius, radius, radius, radius]
            },
            SDFShape::Box { half_size } => {
                [half_size.x, half_size.y, half_size.z, 0.0]
            },
            SDFShape::RoundedBox { half_size, radius } => {
                [half_size.x, half_size.y, half_size.z, *radius]
            }
        };

        cs_globalsdf::ty::SDFPrimitive {
            half_size_radius,
            inv_xform: [self.get_inv_xform().row(0), self.get_inv_xform.row(1), self.get_inv_xform().row(2)]
        }        
    }
}
*/
//mod sdfscene;
