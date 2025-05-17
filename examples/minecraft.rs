#[cfg(feature = "bevy_wgpu")]
use bevy::{prelude::*, window::WindowPlugin};

#[cfg(feature = "bevy_wgpu")]
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

#[cfg(feature = "bevy_wgpu")]
use iyes_perf_ui::{
    entries::diagnostics::{PerfUiEntryFPS, PerfUiEntryFPSWorst},
    ui::root::PerfUiRoot,
    PerfUiPlugin,
};

#[cfg(feature = "bevy_wgpu")]
const DISPLAY_RESOLUTION: [u32; 2] = [1024, 768];

#[cfg(feature = "bevy_wgpu")]
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            voxelhex::raytracing::bevy::RenderBevyPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            PanOrbitCameraPlugin,
            PerfUiPlugin,
        ))
        .add_systems(Startup, setup)
        //.add_systems(Update, set_viewport_for_camera)
        //.add_systems(Update, handle_zoom)
        .run();
}

#[cfg(feature = "bevy_wgpu")]
fn setup(mut commands: Commands, images: ResMut<Assets<Image>>) {
    use voxelhex::{raytracing::{bevy::types::RaymarchingViewSet, Viewport}, spatial::math::vector::V3c};

    std::env::set_var("RUST_BACKTRACE", "1");
    /*let tree: Contree;
    let tree_path = "example_junk_minecraft";
    if std::path::Path::new(tree_path).exists() {
        tree = Contree::load(&tree_path).ok().unwrap();
    } else {
        println!("Loading minecraft.vox");
        tree = match voxelhex::contree::Contree::load_vox_file("assets/models/minecraft.vox") {
            Ok(tree_) => tree_,
            Err(message) => panic!("Parsing model file failed with message: {message}"),
        };
        println!("Loaded minecraft.vox");
        tree.save(&tree_path).ok().unwrap();
    }*/

    let view = RaymarchingViewSet::new(
        Viewport::new(
            V3c {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            V3c {
                x: 0.,
                y: 0.,
                z: -1.,
            },
            V3c::new(10., 10., 200.),
            6.,
        ),
        DISPLAY_RESOLUTION,
        images,
    );

    let mut display = Sprite::from_image(
        view.view
            .lock()
            .unwrap()
            .output_texture()
            .clone(),
    );
    commands.insert_resource(view);
    display.custom_size = Some(Vec2::new(1024., 768.));
    commands.spawn(display);
    commands.spawn((
        Camera {
            is_active: false,
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::new(0., 300., 0.),
            ..default()
        },
    ));
    commands.spawn(Camera2d::default());
    commands.spawn((
        PerfUiRoot::default(),
        PerfUiEntryFPS {
            label: "Frame Rate (current)".into(),
            threshold_highlight: Some(60.0),
            digits: 5,
            precision: 2,
            ..default()
        },
        PerfUiEntryFPSWorst {
            label: "Frame Rate (worst)".into(),
            threshold_highlight: Some(60.0),
            digits: 5,
            precision: 2,
            ..default()
        },
    ));
}

#[cfg(feature = "bevy_wgpu")]
fn direction_from_cam(cam: &PanOrbitCamera) -> Option<voxelhex::spatial::math::vector::V3cf32> {
    use voxelhex::spatial::math::vector::V3c;

    if let Some(radius) = cam.radius {
        Some(
            V3c::new(
                radius / 2. + cam.yaw.unwrap().sin() * radius,
                radius + cam.pitch.unwrap().sin() * radius * 2.,
                radius / 2. + cam.yaw.unwrap().cos() * radius,
            )
            .normalized(),
        )
    } else {
        None
    }
}

#[cfg(not(feature = "bevy_wgpu"))]
fn main() {
    println!("You probably forgot to enable the bevy_wgpu feature!");
    //nothing to do when the feature is not enabled
}
