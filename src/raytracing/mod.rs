#[cfg(feature = "bevy_wgpu")]
pub mod bevy;

#[cfg(feature = "bevy_wgpu")]
pub use bevy::types::{
    ContreeGPUHost, BoxTreeGPUView, ContreeRenderData, BoxTreeSpyGlass, RenderBevyPlugin,
    VhxViewSet, Viewport,
};
