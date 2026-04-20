use bevy::{dev_tools::fps_overlay::FpsOverlayConfig, prelude::*};
use bevy_egui::{
    EguiContexts,
    egui::{self, Widget},
};
use strum::IntoEnumIterator;

use crate::{Health, StartGame, clear_all};

use super::{
    enemies::{SpawnTurret, SpawnWorm},
    garbage::{
        AvailableItemBuilds, GarbageAssets, GarbageBundle, GarbageItem, SpawnBuild, spawn_builds,
        spawn_some_garbage,
    },
    player::{ActiveSkill, Player, SkillState},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
            bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default(),
        ))
        .add_systems(Update, (commands_ui, players_ui, debug_ui));
    }
}

fn debug_ui(mut context: EguiContexts, mut ui_opts: ResMut<FpsOverlayConfig>) {
    let ctx = context.ctx_mut().unwrap();
    egui::Window::new("Debug").show(ctx, |ui| {
        ui.checkbox(&mut ui_opts.enabled, "FPS Overlay");
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
    mut worm_evw: MessageWriter<SpawnWorm>,
    mut turret_evw: MessageWriter<SpawnTurret>,
) {
    if *worm_size == 0 {
        *worm_size = 5;
    }
    let ctx = context.ctx_mut().unwrap();
    egui::Window::new("Commands").show(ctx, |ui| {
        if ui.button("Clear Map").clicked() {
            commands.queue(clear_all());
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
            commands.queue(*start_game);
        }
        ui.heading("Garbage");
        egui::ComboBox::from_id_salt("Spawn Garbage Item")
            .selected_text("Spawn Garbage")
            .show_ui(ui, |ui| {
                for item in GarbageItem::iter() {
                    if ui.button(format!("{item:?}")).clicked() {
                        commands.spawn(GarbageBundle::new(item, &assets));
                    }
                }
            });
        if ui.button("Spawn 50 garbage items").clicked() {
            commands.queue(spawn_some_garbage(50, None, None));
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
                        commands.queue(SpawnBuild {
                            handle: handle.clone(),
                            position: Vec3::new(pos.x, 1.0, pos.y),
                            angle: *rot,
                        });
                    }
                }
            });

        if ui.button("Spawn 10 builds").clicked() {
            commands.queue(spawn_builds(10, None, None));
        }

        if ui.button("Spawn 50 builds").clicked() {
            commands.queue(spawn_builds(50, None, None));
        }

        ui.heading("Enemies");
        ui.horizontal(|ui| {
            ui.label("Worm Size");
            egui::Slider::new(&mut *worm_size, 5..=20).ui(ui);
        });
        if ui.button("Spawn Worm").clicked() {
            worm_evw.write(SpawnWorm {
                size: *worm_size,
                position: *pos,
            });
        }
        if ui.button("Spawn Turret").clicked() {
            turret_evw.write(SpawnTurret { position: *pos });
        }
    });
}

fn players_ui(
    // mut player_connected_evw: MessageWriter<PlayerConnected>,
    mut context: EguiContexts,
    mut players: Query<(&Player, &ActiveSkill, &SkillState, &mut Health)>,
) {
    let ctx = context.ctx_mut().unwrap();
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
                    .id_salt(format!("Skills {}", player.id))
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
        // if ui.button("Spawn fake player").clicked() {
        //     player_connected_evw.send(PlayerConnected(Player {
        //         id: player_count as u8,
        //         controller: GameController::Gamepad {
        //             category: GamepadCategory::Unknown,
        //             gamepad: Gamepad {
        //                 vendor_id: Some(u16::MAX),
        //                 product_id: Some(player_count as u16),
        //                 digital: ButtonInput::default(),
        //                 analog: Axis::default(),
        //             },
        //         },
        //     }));
        // }
    });
}
