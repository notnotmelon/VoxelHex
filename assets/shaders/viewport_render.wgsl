@group(0) @binding(0) var<uniform> stage_data: RenderStageData;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2) var depth_texture: texture_storage_2d<r32float, read_write>;

const RENDER_STAGE_DEPTH_PREPASS = 0u;
const RENDER_STAGE_MAIN = 1u;

struct RenderStageData {
    stage: u32,
    output_resolution: vec2u
}

/// In preprocess, a small resolution depth texture is rendered.
/// After a certain distance in the ray, the result becomes ambigious,
/// because the pixel ( source of raycast ) might cover multiple voxels at the same time.
/// The estimate distance before the ambigiutiy is still adequate is calculated based on:
/// texture_resolution / voxels_count(distance) >= minimum_size_of_voxel_in_pixels
/// wherein:
/// voxels_count: the number of voxel estimated to take up the viewport at a given distance
/// minimum_size_of_voxel_in_pixels: based on the depth texture half the size of the output
/// --> the size of a voxel to be large enough to be always contained by
/// --> at least one pixel in the depth texture
/// No need to continue iteration if one voxel becomes too small to be covered by a pixel completely
/// In these cases, there were no hits so far, which is valuable information
/// even if no useful data can be collected moving forward.

@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    textureStore(output_texture, vec2u(invocation_id.xy), vec4f(1., 0., 1., 1.));
}