// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    a11y::AccessibilityPlugin,
    core_pipeline::CorePipelinePlugin,
    diagnostic::DiagnosticsPlugin,
    input::InputPlugin,
    prelude::*,
    render::{pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin},
    scene::ScenePlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    time::TimePlugin,
    ui::UiPlugin,
    window::PrimaryWindow,
    winit::{WinitPlugin, WinitWindows},
};
use sandboxed::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins((
            (
                TaskPoolPlugin::default(),
                TypeRegistrationPlugin::default(),
                FrameCountPlugin::default(),
                TimePlugin::default(),
                TransformPlugin::default(),
                HierarchyPlugin::default(),
                DiagnosticsPlugin::default(),
                AssetPlugin::default(),
            ),
            InputPlugin::default(),
            AccessibilityPlugin,
            RenderPlugin::default(),
            ImagePlugin::default(),
            PipelinedRenderingPlugin::default(),
            WinitPlugin::default(),
            ScenePlugin::default(),
            CorePipelinePlugin::default(),
            SpritePlugin::default(),
            TextPlugin::default(),
            UiPlugin::default(),
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Sandboxed".to_string(),
                    resolution: (800., 600.).into(),
                    // Bind to canvas included in `index.html`
                    canvas: Some("#bevy".to_owned()),
                    // Tells wasm not to override default event handling, like F5 and Ctrl+R
                    prevent_default_event_handling: false,
                    //                        present_mode: PresentMode::AutoNoVsync, // TODO allow for a timed system within the shaders
                    ..default()
                }),
                ..default()
            },
        ))
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
