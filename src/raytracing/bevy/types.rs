use std::sync::{Arc, Mutex};

use crate::spatial::math::vector::V3cf32;
use bevy::{
    asset::Handle, ecs::system::Resource, math::UVec2, prelude::Image, render::{
        extract_resource::ExtractResource, render_graph::RenderLabel, render_resource::{
            BindGroup, BindGroupLayout, CachedComputePipelineId, ShaderType
        }, renderer::RenderQueue
    }
};

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Viewport {
    /// The origin of the viewport, think of it as the position the eye
    pub origin: V3cf32,

    /// The direction the raycasts are based upon, think of it as wherever the eye looks
    pub direction: V3cf32,

    /// The volume the viewport reaches to
    /// * `x` - looking glass width
    /// * `y` - looking glass height
    /// * `z` - the max depth of the viewport
    pub frustum: V3cf32,

    /// Field of View: how scattered will the rays in the viewport are
    pub fov: f32,
}

/// The Camera responsible for storing frustum and view related data
#[derive(Debug, Clone)]
pub struct BoxTreeSpyGlass {
    // The texture used to store depth information in the scene
    pub(crate) depth_texture: Handle<Image>,

    /// The currently used output texture
    pub(crate) output_texture: Handle<Image>,

    // Set to true, if the viewport changed
    pub(crate) viewport_changed: bool,

    // The viewport containing display information
    pub(crate) viewport: Viewport,

    // The nodes requested by the raytracing algorithm to be displayed
    pub(crate) node_requests: Vec<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct VictimPointer {
    pub(crate) max_meta_len: usize,
    pub(crate) loop_count: usize,
    pub(crate) stored_items: usize,
    pub(crate) meta_index: usize,
    pub(crate) child: usize,
}

pub(crate) const VHX_PREPASS_STAGE_ID: u32 = 01;
pub(crate) const VHX_RENDER_STAGE_ID: u32 = 02;

#[derive(Debug, Clone, Copy, ShaderType)]
pub(crate) struct RenderStageData {
    pub(crate) stage: u32,
    pub(crate) output_resolution: UVec2,
}

#[derive(Resource)]
pub(crate) struct RaymarchingRenderPipeline {
    pub update_tree: bool,
    pub(crate) render_queue: RenderQueue,
    pub(crate) update_pipeline: CachedComputePipelineId,
    pub(crate) render_stage_bind_group_layout: BindGroupLayout,
}

#[derive(Clone)]
pub(crate) struct ContreeRenderDataResources {
    pub(crate) render_stage_prepass_bind_group: BindGroup,
    pub(crate) render_stage_main_bind_group: BindGroup
}

#[derive(Resource, Clone)]
pub struct BoxTreeGPUView {
    /// The camera for casting the rays
    pub spyglass: BoxTreeSpyGlass,

    /// Set to true if the view needs to be reloaded
    pub(crate) reload: bool,

    /// Set to true if the view needs to be refreshed, e.g. by a resolution change
    pub(crate) rebuild: bool,

    /// True if the initial data already sent to GPU
    pub init_data_sent: bool,

    /// Sets to true if related data on the GPU matches with CPU
    pub data_ready: bool,

    /// The currently used resolution the raycasting dimensions are based for the base ray
    pub(crate) resolution: [u32; 2],

    /// The new resolution to be set if any
    pub(crate) new_resolution: Option<[u32; 2]>,

    /// The new depth texture to be used, if any
    pub(crate) new_depth_texture: Option<Handle<Image>>,

    /// The new output texture to be used, if any
    pub(crate) new_output_texture: Option<Handle<Image>>,
}

#[derive(Default, Resource, Clone, ExtractResource)]
pub struct RaymarchingViewSet {
    //pub views: Arc<Mutex<BoxTreeGPUView>>,
    pub(crate) resources: Option<ContreeRenderDataResources>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct RaymarchingLabel;

pub(crate) struct RaymarchingRenderNode {
    pub(crate) ready: bool,
}
