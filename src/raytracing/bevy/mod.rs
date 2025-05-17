mod pipeline;
pub mod types;

pub use crate::raytracing::bevy::types::{
    BoxTreeSpyGlass, Viewport,
};
use crate::{
    contree::{contree_gpu_serialization::BakedContree, types::Albedo}, raytracing::bevy::{
        pipeline::prepare_bind_groups,
        types::{RaymarchingLabel, RaymarchingRenderNode, RaymarchingRenderPipeline},
    }, spatial::math::vector::V3cf32
};
use bevy::{
    app::{App, Plugin},
    prelude::{
        Assets, Handle, Image, IntoSystemConfigs, ResMut,
        Update, Vec4,
    },
    render::{
        extract_component::ExtractComponentPlugin, extract_resource::ExtractResourcePlugin, render_asset::RenderAssetUsages, render_graph::RenderGraph, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, Render, RenderApp, RenderSet
    },
};
use pipeline::write_to_gpu;
use types::{BoxTreeGPUView, RaymarchingViewSet};

impl From<Vec4> for Albedo {
    fn from(vec: Vec4) -> Self {
        Albedo::default()
            .with_red((vec.x * 255.).min(255.) as u8)
            .with_green((vec.y * 255.).min(255.) as u8)
            .with_blue((vec.z * 255.).min(255.) as u8)
            .with_alpha((vec.w * 255.).min(255.) as u8)
    }
}

impl From<Albedo> for Vec4 {
    fn from(color: Albedo) -> Self {
        Vec4::new(
            color.r as f32 / 255.,
            color.g as f32 / 255.,
            color.b as f32 / 255.,
            color.a as f32 / 255.,
        )
    }
}

impl BoxTreeGPUView {
    /// Erases the whole view to be uploaded to the GPU again
    pub fn reload(&mut self) {
        self.reload = true;
    }

    /// Provides the handle to the output texture
    /// Warning! Handle will no longer being updated after resolution change
    pub fn output_texture(&self) -> &Handle<Image> {
        &self.spyglass.output_texture
    }

    /// Updates the resolution on which the view operates on.
    /// It will make a new output texture if size is larger, than the current output texture
    pub fn set_resolution(
        &mut self,
        resolution: [u32; 2],
        images: &mut ResMut<Assets<Image>>,
    ) -> Handle<Image> {
        if self.resolution != resolution {
            self.new_resolution = Some(resolution);
            self.new_output_texture = Some(create_output_texture(resolution, images));
            self.new_depth_texture = Some(create_depth_texture(resolution, images));
            self.rebuild = true;
            self.new_output_texture.as_ref().unwrap().clone_weak()
        } else {
            self.spyglass.output_texture.clone_weak()
        }
    }

    /// Provides currently used resolution for the view
    pub fn resolution(&self) -> [u32; 2] {
        self.resolution
    }
}

impl BoxTreeSpyGlass {
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }
    pub fn view_frustum(&self) -> &V3cf32 {
        &self.viewport.frustum
    }
    pub fn view_fov(&self) -> f32 {
        self.viewport.fov
    }
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        self.viewport_changed = true;
        &mut self.viewport
    }
}

impl Viewport {
    pub fn new(origin: V3cf32, direction: V3cf32, frustum: V3cf32, fov: f32) -> Self {
        Self {
            origin,
            direction,
            frustum,
            fov,
        }
    }
}

pub(crate) fn create_output_texture(
    resolution: [u32; 2],
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    let mut output_texture = Image::new_fill(
        Extent3d {
            width: resolution[0],
            height: resolution[1],
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    output_texture.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    images.add(output_texture)
}

/// Create a depth texture for the given output resolutions
/// Depth texture resolution should cover a single voxel
pub(crate) fn create_depth_texture(
    resolution: [u32; 2],
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    let mut depth_texture = Image::new_fill(
        Extent3d {
            width: resolution[0] / 2,
            height: resolution[1] / 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    depth_texture.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    images.add(depth_texture)
}

pub(crate) fn handle_resolution_updates(
    //images: ResMut<Assets<Image>>,
    //server: Res<AssetServer>,
) {
    // todo
}

#[derive(Default)]
pub struct RenderBevyPlugin;

impl Plugin for RenderBevyPlugin
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<BakedContree>::default(),
            ExtractResourcePlugin::<RaymarchingViewSet>::default()
        ));
        app.add_systems(Update, handle_resolution_updates);
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                write_to_gpu.in_set(RenderSet::PrepareResources),
                prepare_bind_groups.in_set(RenderSet::PrepareBindGroups),
                //handle_gpu_readback.in_set(RenderSet::Cleanup),
            ),
        );
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(RaymarchingLabel, RaymarchingRenderNode { ready: false });
        render_graph.add_node_edge(RaymarchingLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<RaymarchingRenderPipeline>();
    }
}
