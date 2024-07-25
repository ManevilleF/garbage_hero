#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code, clippy::type_complexity)]
use avian3d::prelude::*;
use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};
use bevy_mod_outline::{
    AsyncSceneInheritOutlinePlugin, AutoGenerateOutlineNormalsPlugin, OutlinePlugin,
};

mod plugins;

use plugins::*;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(States, Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Running,
    Pause,
}

#[derive(PhysicsLayer)]
pub enum ObjectLayer {
    Player,
    Enemy,
    Map,
    Item,
    Bullet,
    Collectible,
    Collector,
}

#[derive(Event, Clone, Copy, Default)]
pub struct PauseGame;

pub fn run() -> AppExit {
    println!("Running {APP_NAME} v{APP_VERSION}");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1920.0, 1080.0).into(),
            name: Some(APP_NAME.to_string()),
            ..default()
        }),
        ..default()
    }))
    .init_state::<GameState>()
    .add_event::<PauseGame>()
    // Built in
    .add_plugins((
        PhysicsPlugins::default(),
        OutlinePlugin,
        AsyncSceneInheritOutlinePlugin,
        AutoGenerateOutlineNormalsPlugin,
        TemporalAntiAliasPlugin,
    ))
    // Custom
    .add_plugins((
        LightPlugin,
        GarbagePlugin,
        PlayerPlugin,
        CommonPlugin,
        CameraPlugin,
        MapPlugin,
        ParticlesPlugin,
    ))
    .add_systems(PostUpdate, handle_pause);
    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);
    #[cfg(feature = "editor")]
    app.add_plugins(DebugEditorPlugin);
    #[cfg(feature = "debug_physics")]
    app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin::default());
    app.run()
}

fn handle_pause(
    state: Res<State<GameState>>,
    mut nextstate: ResMut<NextState<GameState>>,
    mut events: EventReader<PauseGame>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    let new_state = match state.get() {
        GameState::Running => {
            physics_time.pause();
            GameState::Pause
        }
        GameState::Pause => {
            physics_time.unpause();
            GameState::Running
        }
    };
    nextstate.set(new_state);
}
