#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code, clippy::type_complexity, clippy::option_if_let_else)]
use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, ecs::world::Command, prelude::*,
    time::common_conditions::on_timer,
};
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

#[derive(PhysicsLayer, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectLayer {
    Player,
    Enemy,
    Map,
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
            canvas: Some("#game-canvas".to_owned()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .init_state::<GameState>()
    .add_event::<PauseGame>()
    .register_type::<StartGame>()
    // Built in
    .add_plugins((
        PhysicsPlugins::default(),
        OutlinePlugin,
        AsyncSceneInheritOutlinePlugin,
        AutoGenerateOutlineNormalsPlugin,
        TemporalAntiAliasPlugin,
    ))
    // Physics config
    .insert_resource(SubstepCount(3))
    // Custom
    .add_plugins((
        LightPlugin,
        GarbagePlugin,
        PlayerPlugin,
        CommonPlugin,
        CameraPlugin,
        MapPlugin,
        ParticlesPlugin,
        EnemiesPlugin,
        #[cfg(not(feature = "debug"))]
        SplashScreenPlugin,
    ))
    .add_systems(PostUpdate, handle_pause)
    .add_systems(
        Last,
        handle_game_end
            .run_if(resource_exists::<StartGame>)
            .run_if(on_timer(Duration::from_secs(5))),
    );
    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);
    #[cfg(feature = "debug_world")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
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

pub fn clear_all() -> impl FnOnce(&mut World) {
    |world| {
        let mut items_q = world.query_filtered::<Entity, With<GarbageItem>>();
        let mut entities: Vec<_> = items_q.iter(world).collect();
        let mut enemies_q = world.query_filtered::<Entity, With<Enemy>>();
        entities.extend(enemies_q.iter(world));
        let mut starters_q = world.query_filtered::<Entity, With<StartGame>>();
        entities.extend(starters_q.iter(world));
        let mut commands = world.commands();
        for entity in entities {
            commands.entity(entity).despawn_recursive();
        }
        let mut animations = world.query::<&mut AnimationPlayer>();
        for mut anim in animations.iter_mut(world) {
            anim.rewind_all();
        }
        world.remove_resource::<StartGame>();
    }
}

#[derive(Clone, Copy, Component, Resource, Reflect)]
pub struct StartGame {
    worm_count: usize,
    turret_count: usize,
}

impl Default for StartGame {
    fn default() -> Self {
        Self {
            worm_count: 2,
            turret_count: 5,
        }
    }
}

impl Command for StartGame {
    fn apply(self, world: &mut World) {
        clear_all()(world);
        // players
        reset_players(world);
        // enemies
        spawn_enemies(self.worm_count, self.turret_count, world);
        // items
        spawn_builds(100, None, None)(world);
        // Setup current game
        world.insert_resource(self)
    }
}

pub fn handle_game_end(
    mut commands: Commands,
    enemies: Query<(), With<Enemy>>,
    players: Query<(), (With<Player>, Without<Dead>)>,
) {
    let ended = {
        if enemies.iter().count() == 0 {
            println!("WIN");
            true
        } else if players.iter().count() == 0 {
            println!("LOOSE");
            true
        } else {
            false
        }
    };
    if ended {
        commands.add(clear_all());
        commands.add(reset_players);
        commands.add(spawn_game_starters);
    }
}
