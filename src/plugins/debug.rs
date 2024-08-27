use bevy::{dev_tools::ui_debug_overlay::UiDebugOptions, prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Widget},
    EguiContexts,
};
use strum::IntoEnumIterator;

use crate::{clear_all, Health, StartGame};

use super::{
    enemies::{SpawnTurret, SpawnWorm},
    garbage::{
        spawn_builds, spawn_some_garbage, AvailableItemBuilds, GarbageAssets, GarbageBundle,
        GarbageItem, SpawnBuild,
    },
    player::{ActiveSkill, GameController, GamepadCategory, Player, PlayerConnected, SkillState},
    ui::input_icons::InputMapIcons,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_egui::EguiPlugin,
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
            bevy::dev_tools::ui_debug_overlay::DebugUiPlugin,
        ))
        .add_systems(Update, (commands_ui, players_ui, debug_ui));
    }
}

fn debug_ui(mut context: EguiContexts, mut ui_opts: ResMut<UiDebugOptions>) {
    let ctx = context.ctx_mut();
    egui::Window::new("Debug").show(ctx, |ui| {
        ui.checkbox(&mut ui_opts.enabled, "Debug Ui Overlay");
    });
}

#[allow(clippy::too_many_arguments)]
fn commands_ui(
    mut commands: Commands,
    mut context: EguiContexts,
    assets: Res<GarbageAssets>,
    builds: Res<AvailableItemBuilds>,
    mut pos: Local<Vec2>,
    mut rot: Local<f32>,
    mut worm_size: Local<usize>,
    mut start_game: Local<StartGame>,
    mut worm_evw: EventWriter<SpawnWorm>,
    mut turret_evw: EventWriter<SpawnTurret>,
) {
    if *worm_size == 0 {
        *worm_size = 5;
    }
    let ctx = context.ctx_mut();
    egui::Window::new("Commands").show(ctx, |ui| {
        if ui.button("Clear Map").clicked() {
            commands.add(clear_all());
        }
        ui.heading("Game start");
        ui.horizontal(|ui| {
            ui.label("worms");
            egui::Slider::new(&mut start_game.worm_count, 0..=20).ui(ui);
        });
        ui.horizontal(|ui| {
            ui.label("turrets");
            egui::Slider::new(&mut start_game.turret_count, 0..=20).ui(ui);
        });
        if ui.button("Start").clicked() {
            commands.add(*start_game);
        }
        ui.heading("Garbage");
        egui::ComboBox::from_id_source("Spawn Garbage Item")
            .selected_text("Spawn Garbage")
            .show_ui(ui, |ui| {
                for item in GarbageItem::iter() {
                    if ui.button(format!("{item:?}")).clicked() {
                        commands.spawn(GarbageBundle::new(item, &assets));
                    }
                }
            });
        if ui.button("Spawn 50 garbage items").clicked() {
            commands.add(spawn_some_garbage(50, None, None));
        }
        ui.heading("Builds");
        ui.horizontal(|ui| {
            ui.label("Position");
            egui::DragValue::new(&mut pos.x).ui(ui);
            egui::DragValue::new(&mut pos.y).ui(ui);
        });
        ui.drag_angle(&mut rot);
        egui::ComboBox::from_label("Spawn Item Build")
            .selected_text("Spawn Build")
            .show_ui(ui, |ui| {
                for (label, handle) in builds.iter() {
                    if ui.button(label).clicked() {
                        commands.add(SpawnBuild {
                            handle: handle.clone_weak(),
                            position: Vec3::new(pos.x, 1.0, pos.y),
                            angle: *rot,
                        });
                    }
                }
            });

        if ui.button("Spawn 10 builds").clicked() {
            commands.add(spawn_builds(10, None, None));
        }

        if ui.button("Spawn 50 builds").clicked() {
            commands.add(spawn_builds(50, None, None));
        }

        ui.heading("Enemies");
        ui.horizontal(|ui| {
            ui.label("Worm Size");
            egui::Slider::new(&mut *worm_size, 5..=20).ui(ui);
        });
        if ui.button("Spawn Worm").clicked() {
            worm_evw.send(SpawnWorm {
                size: *worm_size,
                position: *pos,
            });
        }
        if ui.button("Spawn Turret").clicked() {
            turret_evw.send(SpawnTurret { position: *pos });
        }
    });
}

fn players_ui(
    mut player_connected_evw: EventWriter<PlayerConnected>,
    mut context: EguiContexts,
    mut players: Query<(&Player, &ActiveSkill, &SkillState, &mut Health)>,
) {
    let ctx = context.ctx_mut();
    let mut player_count = 0_usize;
    egui::Window::new("Players").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (player, skill, state, mut health) in &mut players {
                egui::Grid::new(format!("Player {} Grid", player.id)).show(ui, |ui| {
                    ui.label(format!("{}", player.id));
                    ui.label(format!("{}", player.controller));
                    ui.end_row();
                    ui.label("Health");
                    egui::DragValue::new(&mut health.current).ui(ui);
                    ui.end_row();
                    ui.label("Skill");
                    if let Some(skill) = skill.active {
                        ui.label(format!("{}", skill));
                    }
                    ui.end_row();
                });
                egui::CollapsingHeader::new("Skills")
                    .id_source(format!("Skills {}", player.id))
                    .show(ui, |ui| {
                        egui::Grid::new("cooldowns").show(ui, |ui| {
                            for (skill, cooldown) in &state.cooldowns {
                                ui.label(format!("{}", skill));
                                ui.label(format!("{}", *cooldown));
                                ui.end_row();
                            }
                        });
                    });
                player_count += 1;
            }
        });
        ui.spacing();
        if ui.button("Spawn fake player").clicked() {
            player_connected_evw.send(PlayerConnected(Player {
                id: player_count as u8,
                controller: GameController::Gamepad {
                    category: GamepadCategory::Unknown,
                    gamepad: Gamepad { id: player_count },
                },
            }));
        }
    });
}
