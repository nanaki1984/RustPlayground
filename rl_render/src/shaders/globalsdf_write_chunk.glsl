#version 450

#include "sdfcommon.glsl"
/*
#ifndef GROUP_SIZE
#   define GROUP_SIZE 4
#endif
*/
#define THREADS_NUM (GROUP_SIZE * GROUP_SIZE * GROUP_SIZE)

layout(local_size_x = GROUP_SIZE, local_size_y = GROUP_SIZE, local_size_z = GROUP_SIZE) in;

layout(set = 0, binding = 0) readonly buffer ChunkPrimitives {
    SDFPrimitive chunk_primitives[];
};

layout(r16f, binding = 1) uniform writeonly image3D globalsdf_output;

layout(constant_id = 0) const int GLOBALSDF_MAX_DIST_VOXELS = 4;

layout(set = 0, binding = 2) uniform ChunkData {
    float voxel_size;
    vec3 aabb_min;
    uint primitives_count;
} chunk_data;

shared uint group_primitives_index[1024];
shared uint group_primitives_count;

void main() {
    uvec3 global_id = gl_GlobalInvocationID;
    uvec3 group_id = gl_WorkGroupID;
    uint local_id = gl_LocalInvocationIndex;

    // Cull shapes for this group
    if (0 == local_id) {
        group_primitives_count = 0;
    }

    groupMemoryBarrier();

    float max_sdf_dist = chunk_data.voxel_size * GLOBALSDF_MAX_DIST_VOXELS;

    float group_cull_size = chunk_data.voxel_size * GROUP_SIZE;
    float group_cull_half_size = group_cull_size * 0.5;
    float group_cull_dist_threshold = group_cull_half_size + max_sdf_dist;

    vec3 group_cull_center = chunk_data.aabb_min + vec3(1.0, 1.0, 1.0) * group_cull_half_size;
    group_cull_center.x += group_id.x * group_cull_size;
    group_cull_center.y += group_id.y * group_cull_size;
    group_cull_center.z += group_id.z * group_cull_size;

    uint prims_per_thread = (chunk_data.primitives_count + THREADS_NUM - 1) / THREADS_NUM;
    for (uint i = 0; i < prims_per_thread; ++i) {
        uint prim_index = (local_id * prims_per_thread) + i;
        if (prim_index < chunk_data.primitives_count) {
            if (SDF_primitive(chunk_primitives[prim_index], group_cull_center) < group_cull_dist_threshold) {
                uint offset = atomicAdd(group_primitives_count, 1);
                group_primitives_index[offset] = prim_index;
            }
        }
    }

    groupMemoryBarrier();

    // Compute actual SDF for chunk voxel
    float voxel_half_size = chunk_data.voxel_size * 0.5;

    vec3 world_position = chunk_data.aabb_min + vec3(1.0, 1.0, 1.0) * voxel_half_size;
    world_position.x += global_id.x * chunk_data.voxel_size;
    world_position.y += global_id.y * chunk_data.voxel_size;
    world_position.z += global_id.z * chunk_data.voxel_size;

    float sdf = 1e8;
    for (uint i = 0; i < group_primitives_count; ++i) {
        // TODO: blends (per groupid)
        sdf = min(sdf, SDF_primitive(chunk_primitives[group_primitives_index[i]], world_position));
    }
/*
    if (abs(sdf) > voxel_half_size) {
        // write sdf, scaled by max_sdf_dist, no brick with more data
    } else {
        // output brick data, will be processed next dispatch
    }
*/
    float voxel_output = clamp(sdf / max_sdf_dist, -1.0, 1.0);
    voxel_output = (voxel_output + 1.0) * 0.5;
    imageStore(globalsdf_output, ivec3(global_id), vec4(voxel_output));
}
