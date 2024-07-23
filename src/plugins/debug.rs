use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Widget},
    EguiContexts,
};
use strum::IntoEnumIterator;

use crate::Health;

use super::{
    garbage::{
        spawn_builds, spawn_some_garbage, AvailableItemBuilds, GarbageAssets, GarbageBundle,
        GarbageItem, SpawnBuild,
    },
    player::{ActiveSkill, Player, SkillState},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin)
            .add_systems(Update, (commands_ui, players_ui));
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
        egui::ComboBox::from_label("Spawn Garbage Item")
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
        for (label, handle) in builds.iter() {
            if ui.button(label).clicked() {
                commands.add(SpawnBuild {
                    handle: handle.clone_weak(),
                    position: *pos,
                    angle: *rot,
                });
            }
        }

        if ui.button("Spawn 20 builds").clicked() {
            commands.add(spawn_builds(20, *pos, 50.0));
        }
    });
}

fn players_ui(
    mut context: EguiContexts,
    players: Query<(&Player, &ActiveSkill, &SkillState, &Health)>,
) {
    let ctx = context.ctx_mut();
    egui::Window::new("Players").show(ctx, |ui| {
        egui::Grid::new("Player Grid").show(ui, |ui| {
            for (player, skill, state, health) in &players {
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
                ui.label("Skills");
                ui.end_row();
                for (skill, cooldown) in &state.cooldowns {
                    ui.label(format!("{}", skill));
                    ui.label(format!("{}", *cooldown));
                    ui.end_row();
                }
            }
            ui.spacing();
        });
    });
}
