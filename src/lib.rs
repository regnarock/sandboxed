#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod engine;
mod loading;
mod menu;
mod player;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::{app::App, render::extract_resource::ExtractResource};
use engine::EnginePlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

#[derive(Resource, Default, Clone, ExtractResource)]
pub struct GameStateRes {
    state: GameState,
}
impl GameStateRes {
    fn new(state: GameState) -> GameStateRes {
        GameStateRes { state }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                EnginePlugin,
                ExtractResourcePlugin::<GameStateRes>::default(),
            ))
            .init_resource::<GameStateRes>()
            .add_systems(PostUpdate, extract_game_state);

        #[cfg(not(debug_assertions))]
        app.add_plugins(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        });

        #[cfg(debug_assertions)]
        {
            app.add_plugins(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "debug,wgpu_core=warn,wgpu_hal=warn,sandboxed=debug".into(),
            });
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

/// Manual extract of [`GameState`] to [`GameStateRes`] (for later exactraction with [`ExtractResourcePlugin`]) since [`State`] cannot be extracted.
///
/// To be ran during [`PostUpdate`] to make sure [`GameStateRes`] is updated in renderer as soon as possible
///
pub fn extract_game_state(
    mut commands: Commands,
    new_state: Res<State<GameState>>,
    current_state_res: Option<ResMut<GameStateRes>>,
) {
    if new_state.is_changed() {
        if let Some(mut current_state) = current_state_res {
            current_state.state = new_state.get().to_owned();
        }
    } else {
        let current_state_res = GameStateRes::new(new_state.get().to_owned());
        commands.insert_resource(current_state_res);
    }
}
