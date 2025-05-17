use std::sync::{Arc, Mutex};

use crate::spatial::math::vector::V3cf32;
use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_graph::RenderLabel, render_resource::*, renderer::RenderQueue}};

use super::{create_depth_texture, create_output_texture};

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
}

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

#[derive(Resource, Clone, ExtractResource)]
pub struct RaymarchingViewSet {
    pub view: Arc<Mutex<BoxTreeGPUView>>,
    pub(crate) resources: Option<ContreeRenderDataResources>,
}

impl RaymarchingViewSet {
    pub fn new(
        viewport: Viewport,
        resolution: [u32; 2],
        mut images: ResMut<Assets<Image>>
    ) -> Self {
        let output_texture = create_output_texture(resolution, &mut images);
        let view = BoxTreeGPUView {
            resolution,
            reload: false,
            rebuild: false,
            init_data_sent: false,
            data_ready: false,
            new_resolution: None,
            new_output_texture: None,
            new_depth_texture: None,
            spyglass: BoxTreeSpyGlass {
                depth_texture: create_depth_texture(resolution, &mut images),
                output_texture,
                viewport_changed: true,
                viewport,
            },
        };
        Self {
            view: Arc::new(Mutex::new(view)),
            resources: None
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct RaymarchingLabel;

pub(crate) struct RaymarchingRenderNode {
    pub(crate) ready: bool,
}
