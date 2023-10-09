mod buffers;
pub mod image;
mod pipeline;

use bevy::{
    diagnostic::*,
    prelude::*,
    render::{extract_resource::ExtractResourcePlugin, Render, RenderApp, RenderSet, render_graph::RenderGraph, renderer::RenderDevice},
    window::*,
};

use crate::{GameState, engine::buffers::VoxelPipelineBuffers};

use self::{image::VoxelsRenderImage, pipeline::{VoxelShadersPipeline, VoxelSandboxNode}};

const SIM_SIZE: (u32, u32) = (800, 600);
const SIM_VOXELS: usize = 800 * 600;
const WORKGROUP_SIZE: u32 = 8;

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, window_fps)
            .add_plugins(ExtractResourcePlugin::<VoxelsRenderImage>::default())
            .add_systems(PostStartup, setup);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<VoxelShadersPipeline>()
            .add_systems(Render, pipeline::queue_bind_group.in_set(RenderSet::Queue));
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("sanbox", VoxelSandboxNode::default());
        render_graph.add_node_edge(
            "sanbox",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
}

fn window_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                window.title = format!("Sandboxed! FPS={fps_smoothed:.2}");
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    device: Res<RenderDevice>,
) {
    let render_image_res = VoxelsRenderImage::new(SIM_SIZE.0, SIM_SIZE.1, images);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: render_image_res.0.clone(),
        ..default()
    });

    let buffers = VoxelPipelineBuffers::new(device);

    debug!("[Resource inserted]{:?}", render_image_res);
    commands.insert_resource(render_image_res);
    debug!("[Resource inserted]{:?}", buffers);
    commands.insert_resource(buffers);
}