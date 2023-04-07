use std::sync::Arc;
use std::mem::MaybeUninit;
use nalgebra_glm::Vec3;
use rl_math::{AABB, VEC3_ONE};
use crate::SDFPrimitivesList;

use vulkano::{
    buffer::{Buffer, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::{StandardDescriptorSetAllocator, DescriptorSetAllocator}, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, MemoryAllocator, StandardMemoryAllocator},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

const GLOBALSDF_CASCADE_SIZE: usize = 128;
const GLOBALSDF_CHUNKS_PER_SIDE: usize = 4;

const GLOBALSDF_CHUNK_SIZE: usize = GLOBALSDF_CASCADE_SIZE / GLOBALSDF_CHUNKS_PER_SIDE;
const GLOBALSDF_CHUNKS_NUM: usize = GLOBALSDF_CHUNKS_PER_SIDE * GLOBALSDF_CHUNKS_PER_SIDE * GLOBALSDF_CHUNKS_PER_SIDE;

pub(crate) const GLOBALSDF_MAX_DIST_VOXELS: i32 = 4;

struct GlobalSDFChunk {
    aabb: AABB,
    extended_aabb: AABB,
    primitives: SDFPrimitivesList,
}

impl GlobalSDFChunk {
    pub fn new(cascade_primitives: &SDFPrimitivesList, cascade_voxel_size: f32, aabb: AABB) -> Self {
        let extended_aabb = aabb.expand(cascade_voxel_size * (GLOBALSDF_MAX_DIST_VOXELS as f32));

        let mut primitives = cascade_primitives.cull(&extended_aabb);
        primitives.sort_by_group_id();

        Self {
            aabb,            
            extended_aabb,
            primitives,
        }
    }
}

pub struct GlobalSDFCascade {
    voxel_size: f32,
    aabb: AABB,
    extended_aabb: AABB,
    chunks: [MaybeUninit<GlobalSDFChunk>; GLOBALSDF_CHUNKS_NUM],

    //primitives_buffer: CpuAccessibleBuffer<[crate::cs_globalsdf::SDFPrimitive]>,
}

impl GlobalSDFCascade {
    pub fn new(scene_primitives: &SDFPrimitivesList, aabb: AABB) -> Self {
        let voxel_size = aabb.size().x / (GLOBALSDF_CASCADE_SIZE as f32);
        let extended_aabb = aabb.expand(voxel_size * (GLOBALSDF_MAX_DIST_VOXELS as f32));

        let primitives = scene_primitives.cull(&extended_aabb);

        let chunk_size = voxel_size * (GLOBALSDF_CHUNK_SIZE as f32);
        let half_chunk_size = chunk_size * 0.5;
        let chunk_extends = VEC3_ONE * half_chunk_size;

        let first_chunk_center = aabb.min + VEC3_ONE * half_chunk_size;

        let mut chunks: [MaybeUninit<GlobalSDFChunk>; GLOBALSDF_CHUNKS_NUM] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        for i in 0..GLOBALSDF_CHUNKS_NUM {
            let chunk_x = i % GLOBALSDF_CHUNKS_PER_SIDE;
            let chunk_y = (i / GLOBALSDF_CHUNKS_PER_SIDE) % GLOBALSDF_CHUNKS_PER_SIDE;
            let chunk_z = i / (GLOBALSDF_CHUNKS_PER_SIDE * GLOBALSDF_CHUNKS_PER_SIDE);

            let chunk_center = first_chunk_center + Vec3::new
                ( chunk_size * (chunk_x as f32)
                , chunk_size * (chunk_y as f32)
                , chunk_size * (chunk_z as f32));

            chunks[i].write(GlobalSDFChunk::new(
                &primitives,
                voxel_size,
                AABB::from_center_extents(&chunk_center, &chunk_extends)));
        }

        Self {
            voxel_size,
            aabb,
            extended_aabb,
            chunks,
        }
    }

    pub fn compute_on_gpu<A>(
        &self,
        pipeline: Arc<ComputePipeline>,
        buffer_allocator: &(impl MemoryAllocator + ?Sized),
        set_allocator: &A)
    where
        A: DescriptorSetAllocator
    {
        for elem in &self.chunks[..] {
            let chunk = unsafe{ elem.assume_init_ref() };
            let prims = chunk.primitives.send_to_gpu();

            //CpuAccessibleBuffer::uninitialized_array(allocator, usage, host_cached)
            /*let data_buffer =
                CpuAccessibleBuffer::from_data(buffer_allocator, BufferUsage {
                    storage_buffer: true,
                    ..Default::default()
                }, false, prims.into_iter())
                .unwrap();

            let layout = pipeline.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                set_allocator,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, data_buffer.clone())],
            )
            .unwrap();*/
        }
    }
}
