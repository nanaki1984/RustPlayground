use std::sync::Arc;
use std::mem::MaybeUninit;
use nalgebra_glm::Vec3;
use rl_math::{AABB, VEC3_ONE};
use crate::{SDFPrimitivesList, cs_globalsdf};

use vulkano::{
    buffer::{Buffer, BufferUsage, BufferCreateInfo, BufferCreateFlags},
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
        // TODO: should use a BumpAllocator for allocating (buffer_allocator) the primitives buffers, release it when finished
        for elem in &self.chunks[..] {
            let chunk = unsafe{ elem.assume_init_ref() };
/*
            let data_buffer =
                Buffer::new_slice::<crate::cs_globalsdf::SDFPrimitive>(
                    buffer_allocator,
                    BufferCreateInfo{ usage: BufferUsage::STORAGE_BUFFER, ..Default::default() },
                    AllocationCreateInfo{ usage: MemoryUsage::Upload, ..Default::default() },
                    prims.len() as u64)
                    .unwrap();
*/
            let primitives_buffer = Buffer::from_iter(
                buffer_allocator,
                BufferCreateInfo{ usage: BufferUsage::STORAGE_BUFFER, ..Default::default() },
                AllocationCreateInfo{ usage: MemoryUsage::Upload, ..Default::default() },
                chunk.primitives.send_to_gpu()
            )
            .unwrap();

            let chunk_data_buffer = Buffer::from_data(
                buffer_allocator,
                BufferCreateInfo{ usage: BufferUsage::UNIFORM_BUFFER, ..Default::default() },
                AllocationCreateInfo { usage: MemoryUsage::Upload, ..Default::default() },
                cs_globalsdf::ChunkData {
                    voxel_size: self.voxel_size.into(),
                    aabb_min: *chunk.aabb.min.as_ref(),
                    primitives_count: chunk.primitives.count()
                }
            )
            .unwrap();

            let layout = pipeline.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                set_allocator,
                layout.clone(),
                [ WriteDescriptorSet::buffer(0, primitives_buffer)
                , WriteDescriptorSet::buffer(2, chunk_data_buffer)],
            )
            .unwrap();
        }
    }
}
