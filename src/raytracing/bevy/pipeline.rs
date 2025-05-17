use crate::
    raytracing::bevy::types::{
        RenderStageData,
        RaymarchingRenderNode, RaymarchingRenderPipeline,
    }
;
use bevy::{
    asset::AssetServer,
    ecs::world::{FromWorld, World},
    render::{
        render_graph::{self},
        render_resource::{
            BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, CachedPipelineState, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache,
            ShaderSize, ShaderStages, ShaderType, StorageTextureAccess, TextureFormat,
            TextureViewDimension,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};
use std::{borrow::Cow, ops::Range};

use super::types::RaymarchingViewSet;

impl FromWorld for RaymarchingRenderPipeline {
    //##############################################################################
    // ███████████  █████ ██████   █████ ██████████
    // ░░███░░░░░███░░███ ░░██████ ░░███ ░░███░░░░███
    //  ░███    ░███ ░███  ░███░███ ░███  ░███   ░░███
    //  ░██████████  ░███  ░███░░███░███  ░███    ░███
    //  ░███░░░░░███ ░███  ░███ ░░██████  ░███    ░███
    //  ░███    ░███ ░███  ░███  ░░█████  ░███    ███
    //  ███████████  █████ █████  ░░█████ ██████████
    // ░░░░░░░░░░░  ░░░░░ ░░░░░    ░░░░░ ░░░░░░░░░░
    //    █████████  ███████████      ███████    █████  █████ ███████████
    //   ███░░░░░███░░███░░░░░███   ███░░░░░███ ░░███  ░░███ ░░███░░░░░███
    //  ███     ░░░  ░███    ░███  ███     ░░███ ░███   ░███  ░███    ░███
    // ░███          ░██████████  ░███      ░███ ░███   ░███  ░██████████
    // ░███    █████ ░███░░░░░███ ░███      ░███ ░███   ░███  ░███░░░░░░
    // ░░███  ░░███  ░███    ░███ ░░███     ███  ░███   ░███  ░███
    //  ░░█████████  █████   █████ ░░░███████░   ░░████████   █████
    //   ░░░░░░░░░  ░░░░░   ░░░░░    ░░░░░░░      ░░░░░░░░   ░░░░░
    //  █████         █████████   █████ █████    ███████    █████  █████ ███████████
    // ░░███         ███░░░░░███ ░░███ ░░███   ███░░░░░███ ░░███  ░░███ ░█░░░███░░░█
    //  ░███        ░███    ░███  ░░███ ███   ███     ░░███ ░███   ░███ ░   ░███  ░
    //  ░███        ░███████████   ░░█████   ░███      ░███ ░███   ░███     ░███
    //  ░███        ░███░░░░░███    ░░███    ░███      ░███ ░███   ░███     ░███
    //  ░███      █ ░███    ░███     ░███    ░░███     ███  ░███   ░███     ░███
    //  ███████████ █████   █████    █████    ░░░███████░   ░░████████      █████
    // ░░░░░░░░░░░ ░░░░░   ░░░░░    ░░░░░       ░░░░░░░      ░░░░░░░░      ░░░░░
    //##############################################################################

    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_stage_bind_group_layout = render_device.create_bind_group_layout(
            "RenderStageBindGroup",
            &[
                BindGroupLayoutEntry {
                    binding: 0u32, // @group(0) @binding(0) var<uniform> stage_data: RenderStageData;
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(<RenderStageData as ShaderType>::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry { // @group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, read_write>;
                    binding: 1u32,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry { // @group(0) @binding(2) var depth_texture: texture_storage_2d<r32float, read_write>;
                    binding: 2u32,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::R32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/viewport_render.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            zero_initialize_workgroup_memory: false,
            label: Some(std::borrow::Cow::Borrowed("MainRenderComputeBindGroup")),
            layout: vec![
                render_stage_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        RaymarchingRenderPipeline {
            render_queue: world.resource::<RenderQueue>().clone(),
            update_tree: true,
            render_stage_bind_group_layout,
            update_pipeline,
        }
    }
}

//##############################################################################
//  ███████████   █████  █████ ██████   █████
// ░░███░░░░░███ ░░███  ░░███ ░░██████ ░░███
//  ░███    ░███  ░███   ░███  ░███░███ ░███
//  ░██████████   ░███   ░███  ░███░░███░███
//  ░███░░░░░███  ░███   ░███  ░███ ░░██████
//  ░███    ░███  ░███   ░███  ░███  ░░█████
//  █████   █████ ░░████████   █████  ░░█████
// ░░░░░   ░░░░░   ░░░░░░░░   ░░░░░    ░░░░░
//##############################################################################
const WORKGROUP_SIZE: u32 = 8;
impl render_graph::Node for RaymarchingRenderNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<RaymarchingRenderPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        if !self.ready {
            if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline) {
                //self.ready = !world.resource::<VhxViewSet>().views.is_empty();
                self.ready = true;
            }
        }
    }

    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.ready { return Ok(()); }

        let pipeline = world.resource::<RaymarchingRenderPipeline>();
        let vhx_view_set = world.resource::<RaymarchingViewSet>();
        //let current_view = vhx_view_set.views[0].lock().unwrap();
        let resolution = [1024, 768];

        if let Some(resources) = &vhx_view_set.resources {
            let pipeline_cache = world.resource::<PipelineCache>();
            let command_encoder = render_context.command_encoder();
            
            {
                let mut prepass =
                    command_encoder.begin_compute_pass(&ComputePassDescriptor::default());

                prepass.set_bind_group(0, &resources.render_stage_prepass_bind_group, &[]);
                let pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                prepass.set_pipeline(pipeline);
                prepass.dispatch_workgroups(
                    (resolution[0] / 2) / WORKGROUP_SIZE,
                    (resolution[1] / 2) / WORKGROUP_SIZE,
                    1,
                );
            }

            let mut main_pass =
                command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
            main_pass.set_bind_group(0, &resources.render_stage_main_bind_group, &[]);
            let pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.update_pipeline)
                .unwrap();
            main_pass.set_pipeline(pipeline);
            main_pass.dispatch_workgroups(
                resolution[0] / WORKGROUP_SIZE,
                resolution[1] / WORKGROUP_SIZE,
                1,
            );
        }
        Ok(())
    }
}

//##############################################################################
//   █████████  ███████████   █████████     █████████  ██████████
//  ███░░░░░███░█░░░███░░░█  ███░░░░░███   ███░░░░░███░░███░░░░░█
// ░███    ░░░ ░   ░███  ░  ░███    ░███  ███     ░░░  ░███  █ ░
// ░░█████████     ░███     ░███████████ ░███          ░██████
//  ░░░░░░░░███    ░███     ░███░░░░░███ ░███    █████ ░███░░█
//  ███    ░███    ░███     ░███    ░███ ░░███  ░░███  ░███ ░   █
// ░░█████████     █████    █████   █████ ░░█████████  ██████████
//  ░░░░░░░░░     ░░░░░    ░░░░░   ░░░░░   ░░░░░░░░░  ░░░░░░░░░░

//    █████████  ███████████      ███████    █████  █████ ███████████   █████████
//   ███░░░░░███░░███░░░░░███   ███░░░░░███ ░░███  ░░███ ░░███░░░░░███ ███░░░░░███
//  ███     ░░░  ░███    ░███  ███     ░░███ ░███   ░███  ░███    ░███░███    ░░░
// ░███          ░██████████  ░███      ░███ ░███   ░███  ░██████████ ░░█████████
// ░███    █████ ░███░░░░░███ ░███      ░███ ░███   ░███  ░███░░░░░░   ░░░░░░░░███
// ░░███  ░░███  ░███    ░███ ░░███     ███  ░███   ░███  ░███         ███    ░███
//  ░░█████████  █████   █████ ░░░███████░   ░░████████   █████       ░░█████████
//   ░░░░░░░░░  ░░░░░   ░░░░░    ░░░░░░░      ░░░░░░░░   ░░░░░         ░░░░░░░░░
//##############################################################################
///
fn create_bind_groups(
    
) {

}

//##############################################################################
//  ███████████  ███████████   ██████████ ███████████    █████████   ███████████   ██████████
// ░░███░░░░░███░░███░░░░░███ ░░███░░░░░█░░███░░░░░███  ███░░░░░███ ░░███░░░░░███ ░░███░░░░░█
//  ░███    ░███ ░███    ░███  ░███  █ ░  ░███    ░███ ░███    ░███  ░███    ░███  ░███  █ ░
//  ░██████████  ░██████████   ░██████    ░██████████  ░███████████  ░██████████   ░██████
//  ░███░░░░░░   ░███░░░░░███  ░███░░█    ░███░░░░░░   ░███░░░░░███  ░███░░░░░███  ░███░░█
//  ░███         ░███    ░███  ░███ ░   █ ░███         ░███    ░███  ░███    ░███  ░███ ░   █
//  █████        █████   █████ ██████████ █████        █████   █████ █████   █████ ██████████
// ░░░░░        ░░░░░   ░░░░░ ░░░░░░░░░░ ░░░░░        ░░░░░   ░░░░░ ░░░░░   ░░░░░ ░░░░░░░░░░
//  ███████████  █████ ██████   █████ ██████████
// ░░███░░░░░███░░███ ░░██████ ░░███ ░░███░░░░███
//  ░███    ░███ ░███  ░███░███ ░███  ░███   ░░███
//  ░██████████  ░███  ░███░░███░███  ░███    ░███
//  ░███░░░░░███ ░███  ░███ ░░██████  ░███    ░███
//  ░███    ░███ ░███  ░███  ░░█████  ░███    ███
//  ███████████  █████ █████  ░░█████ ██████████
// ░░░░░░░░░░░  ░░░░░ ░░░░░    ░░░░░ ░░░░░░░░░░
//    █████████  ███████████      ███████    █████  █████ ███████████   █████████
//   ███░░░░░███░░███░░░░░███   ███░░░░░███ ░░███  ░░███ ░░███░░░░░███ ███░░░░░███
//  ███     ░░░  ░███    ░███  ███     ░░███ ░███   ░███  ░███    ░███░███    ░░░
// ░███          ░██████████  ░███      ░███ ░███   ░███  ░██████████ ░░█████████
// ░███    █████ ░███░░░░░███ ░███      ░███ ░███   ░███  ░███░░░░░░   ░░░░░░░░███
// ░░███  ░░███  ░███    ░███ ░░███     ███  ░███   ░███  ░███         ███    ░███
//  ░░█████████  █████   █████ ░░░███████░   ░░████████   █████       ░░█████████
//   ░░░░░░░░░  ░░░░░   ░░░░░    ░░░░░░░      ░░░░░░░░   ░░░░░         ░░░░░░░░░
//##############################################################################
/// Constructs buffers, bing groups and uploads rendering data at initialization and whenever prompted
pub(crate) fn prepare_bind_groups(

) {

}

//##############################################################################
//    █████████  ███████████  █████  █████
//   ███░░░░░███░░███░░░░░███░░███  ░░███
//  ███     ░░░  ░███    ░███ ░███   ░███
// ░███          ░██████████  ░███   ░███
// ░███    █████ ░███░░░░░░   ░███   ░███
// ░░███  ░░███  ░███         ░███   ░███
//  ░░█████████  █████        ░░████████
//   ░░░░░░░░░  ░░░░░          ░░░░░░░░

//  █████   ███   █████ ███████████   █████ ███████████ ██████████
// ░░███   ░███  ░░███ ░░███░░░░░███ ░░███ ░█░░░███░░░█░░███░░░░░█
//  ░███   ░███   ░███  ░███    ░███  ░███ ░   ░███  ░  ░███  █ ░
//  ░███   ░███   ░███  ░██████████   ░███     ░███     ░██████
//  ░░███  █████  ███   ░███░░░░░███  ░███     ░███     ░███░░█
//   ░░░█████░█████░    ░███    ░███  ░███     ░███     ░███ ░   █
//     ░░███ ░░███      █████   █████ █████    █████    ██████████
//      ░░░   ░░░      ░░░░░   ░░░░░ ░░░░░    ░░░░░    ░░░░░░░░░░
//##############################################################################

/// Converts the given array to `&[u8]` on the given range,
/// and schedules it to be written to the given buffer in the GPU
fn write_range_to_buffer<U>(
    array: &[U],
    index_range: Range<usize>,
    buffer: &Buffer,
    render_queue: &RenderQueue,
) where
    U: Send + Sync + 'static + ShaderSize,
{
    if !index_range.is_empty() {
        let element_size = std::mem::size_of_val(&array[0]);
        let byte_offset = (index_range.start * element_size) as u64;
        let slice = array.get(index_range.clone()).unwrap_or_else(|| {
            panic!(
                "{}",
                format!(
                    "Expected range {:?} to be in bounds of {:?}",
                    index_range,
                    array.len(),
                )
                .to_owned()
            )
        });
        unsafe {
            render_queue.write_buffer(buffer, byte_offset, slice.align_to::<u8>().1);
        }
    }
}

/// Handles Data Streaming to the GPU based on incoming requests from the view(s)
pub(crate) fn write_to_gpu(
    //tree_gpu_host: Option<ResMut<ContreeGPUHost<T>>>,
    //vhx_pipeline: Option<ResMut<VhxRenderPipeline>>,
    //vhx_view_set: ResMut<VhxViewSet>,
) {
    /*
    // Write out the initial data package
    write_range_to_buffer(
        &view.data_handler.render_data.used_bits,
        0..1,
        &resources.used_bits_buffer,
        &pipeline.render_queue,
    );
    write_range_to_buffer(
        &view.data_handler.render_data.node_metadata,
        0..1,
        &resources.node_metadata_buffer,
        &pipeline.render_queue,
    );
    write_range_to_buffer(
        &view.data_handler.render_data.node_children,
        0..BOX_NODE_CHILDREN_COUNT,
        &resources.node_children_buffer,
        &pipeline.render_queue,
    );
    write_range_to_buffer(
        &view.data_handler.render_data.node_ocbits,
        0..2,
        &resources.node_ocbits_buffer,
        &pipeline.render_queue,
    );
    */
}