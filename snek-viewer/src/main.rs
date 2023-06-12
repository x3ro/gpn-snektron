mod data;
mod socketio;
mod viewer;

use std::sync::{Arc, Mutex};
use bevy::log::LogPlugin;

use bevy::prelude::*;
use bevy::window::PresentMode;

use crate::data::{ArcGameState, GameState, ServerGameState};
use crate::socketio::client_thread;
use crate::viewer::SnekViewerPlugin;

fn main() {
    let game_state: ArcGameState = Arc::new(Mutex::new(GameState::default()));
    client_thread(game_state.clone());

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snek Viewer!".into(),
                resolution: (500., 500.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }).set(
        LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,snek_viewer=debug".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugin(SnekViewerPlugin)
        .add_startup_system(setup)
        .insert_resource(ServerGameState::new(game_state.clone()))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
