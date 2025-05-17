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

const detail = 32;
const dist = 700.0;
const steps = 100;
const emptycells = 0.5;

//random function from https://www.shadertoy.com/view/MlsXDf
fn rnd(v: vec4f) -> f32 { return fract(4e4*sin(dot(v,vec4f(13.46,41.74,-73.36,14.24))+17.34)); }

fn getvoxel(pp: vec3<i32>, size: i32) -> bool {
    var p = pp;
    p /= size;
    if (p.x == 0 && p.y == 0) {
        return false;
    }
    let ppp = vec4f(f32(p.x), f32(p.y), f32(p.z), f32(size));

    return rnd(ppp) > emptycells;
}

fn LP2D(i: f32) -> i32 {
    let j = i32(round(i));
    if (j == 0)
    {
        return detail;
    }
    return min(j & -j, detail);
}

@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    if stage_data.stage == RENDER_STAGE_MAIN {
        let uv: vec2f = vec2f(invocation_id.xy) / vec2f(stage_data.output_resolution);
        
        let rayDir: vec3f = normalize(vec3(uv,1.0));
        
        var size = detail;
        let rayOrg: vec3f = vec3f(1.,1.,1.);
        var rayPos: vec3f = rayOrg;
        let deltaDist: vec3f = 1.0/rayDir;
        var hit: vec3f;
        let rayOff: vec3f = sign(rayDir) * 0.01;
        let positive: vec3f = step(-rayDir, vec3f(0.0));
        
        for (var i = 0; i < steps; i += 1)
        {
            if(getvoxel(vec3i(rayPos), size)) {
                if (size <= 1) { break; }
                size /= 2;
                continue;
            }
            
            hit = (positive - fract(rayPos / f32(size))) * f32(size) * deltaDist;

            let min_hit = min(hit.x,min(hit.y,hit.z));
            rayPos += min_hit * rayDir;
            
                if (min_hit == hit.x) {size = LP2D(rayPos.x); rayPos.x += rayOff.x; }
            else if (min_hit == hit.y) {size = LP2D(rayPos.y); rayPos.y += rayOff.y; }
            else if (min_hit == hit.z) {size = LP2D(rayPos.z); rayPos.z += rayOff.z; }

        }
        
        let val: f32 = fract(dot(floor(rayPos),vec3(15.23,754.345,3.454)));
        var fragColor = vec4f(sin(val*vec3f(39.896,57.3225,48.25))*0.5+0.5, 1.0);
        fragColor = sqrt(fragColor) - distance(rayOrg, rayPos)/dist;
        textureStore(output_texture, vec2u(invocation_id.xy), fragColor);
    }
}