use bevy::{asset::AssetEvents, prelude::*, reflect::Typed, utils::HashMap, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, ColorImage, Widget},
    EguiContext, EguiContexts,
};
use strum::IntoEnumIterator;

use crate::Health;

use super::{
    garbage::{
        spawn_builds, spawn_some_garbage, AvailableItemBuilds, GarbageAssets, GarbageBundle,
        GarbageItem, SpawnBuild,
    },
    player::{ActiveSkill, GameController, Player, PlayerConnected, SkillState},
    ui::input_icons::InputMapIcons,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_egui::EguiPlugin,
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
        ))
        .init_resource::<ImageToEgui>()
        .add_systems(Update, (update_images, commands_ui, players_ui));
    }
}

fn commands_ui(
    mut commands: Commands,
    mut context: EguiContexts,
    assets: Res<GarbageAssets>,
    builds: Res<AvailableItemBuilds>,
    mut pos: Local<Vec3>,
    mut rot: Local<f32>,
) {
    let ctx = context.ctx_mut();
    egui::Window::new("Commands").show(ctx, |ui| {
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
            let shape = Cuboid::new(50.0, 5.0, 50.0);
            commands.add(spawn_some_garbage(50, Vec3::Y * 5.0, shape));
        }
        ui.heading("Builds");
        ui.horizontal(|ui| {
            ui.label("Position");
            egui::DragValue::new(&mut pos.x).ui(ui);
            egui::DragValue::new(&mut pos.y).ui(ui);
            egui::DragValue::new(&mut pos.z).ui(ui);
        });
        ui.drag_angle(&mut rot);
        egui::ComboBox::from_label("Spawn Item Build")
            .selected_text("Spawn Build")
            .show_ui(ui, |ui| {
                for (label, handle) in builds.iter() {
                    if ui.button(label).clicked() {
                        commands.add(SpawnBuild {
                            handle: handle.clone_weak(),
                            position: *pos,
                            angle: *rot,
                        });
                    }
                }
            });

        if ui.button("Spawn 10 builds").clicked() {
            commands.add(spawn_builds(10, *pos, 10.0));
        }

        if ui.button("Spawn 50 builds").clicked() {
            commands.add(spawn_builds(50, *pos, 50.0));
        }
    });
}

#[derive(Resource, Default)]
struct ImageToEgui(HashMap<Handle<Image>, egui::TextureId>);

fn update_images(
    new_icons: Query<&InputMapIcons, Added<InputMapIcons>>,
    mut images: ResMut<ImageToEgui>,
    mut context: EguiContexts,
) {
    for icons in &new_icons {
        for handle in icons.values() {
            let texture = context.add_image(handle.clone_weak());
            images.0.insert(handle.clone_weak(), texture);
        }
    }
}

fn players_ui(
    mut player_connected_evw: EventWriter<PlayerConnected>,
    mut context: EguiContexts,
    textures: Res<ImageToEgui>,
    players: Query<(&Player, &ActiveSkill, &SkillState, &Health, &InputMapIcons)>,
) {
    let ctx = context.ctx_mut();
    let mut player_count = 0_usize;
    egui::Window::new("Players").show(ctx, |ui| {
        for (player, skill, state, health, icons) in &players {
            egui::Grid::new(format!("Player {} Grid", player.id)).show(ui, |ui| {
                ui.label(format!("{}", player.id));
                ui.label(format!("{}", player.controller));
                ui.end_row();
                ui.label("Health");
                ui.label(format!("{}", health.current));
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
            egui::CollapsingHeader::new("Icons")
                .id_source(format!("Icons {}", player.id))
                .show(ui, |ui| {
                    egui::Grid::new("icons").show(ui, |ui| {
                        for (input, icon) in icons.iter() {
                            ui.label(format!("{input:?}"));
                            if let Some(texture) = textures.0.get(icon) {
                                ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                                    *texture,
                                    [48.0, 48.0],
                                )));
                            }
                            ui.end_row();
                        }
                    });
                });
            player_count += 1;
        }
        ui.spacing();
        if ui.button("Spawn fake player").clicked() {
            player_connected_evw.send(PlayerConnected(Player {
                id: player_count as u8,
                controller: GameController::Gamepad(Gamepad { id: player_count }),
            }));
        }
    });
}
