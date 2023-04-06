mod sdfshapes;

pub use sdfshapes::SDFShape;
pub use sdfshapes::SDFPrimitive;
pub use sdfshapes::SDFPrimitivesList;

mod globalsdf;

pub use globalsdf::GlobalSDFCascade;

mod globalsdf_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "./src/shaders/globalsdf_write_chunk.glsl",
        define: [
            ("GROUP_SIZE", "4")
        ],
        //dump: true,
    }
}

//mod sdfscene;
