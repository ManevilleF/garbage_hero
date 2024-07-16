#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code)]
use avian3d::prelude::PhysicsLayer;
use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};

mod plugins;

use plugins::*;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(States, Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
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

pub fn run() -> AppExit {
    println!("Running {APP_NAME} v{APP_VERSION}");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1920.0, 1080.0).into(),
            ..default()
        }),
        ..default()
    }))
    .init_state::<GameState>()
    .add_plugins(TemporalAntiAliasPlugin)
    .add_plugins((LightPlugin, GarbagePlugin, PlayerPlugin, CommonPlugin));
    #[cfg(feature = "debug")]
    app.add_plugins((
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        avian3d::debug_render::PhysicsDebugPlugin::default(),
    ));
    app.run()
}
