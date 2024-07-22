use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::IntoEnumIterator;

use super::{
    garbage::{spawn_some_garbage, GarbageAssets, GarbageBundle, GarbageItem},
    player::{ActiveSkill, Player, SkillState},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin)
            .add_systems(Update, (commands_ui, players_ui));
    }
}

fn commands_ui(mut commands: Commands, mut context: EguiContexts, assets: Res<GarbageAssets>) {
    let ctx = context.ctx_mut();
    egui::Window::new("Commands").show(ctx, |ui| {
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
    });
}

fn players_ui(mut context: EguiContexts, players: Query<(&Player, &ActiveSkill)>) {
    let ctx = context.ctx_mut();
    egui::Window::new("Players").show(ctx, |ui| {
        egui::Grid::new("Player Grid").show(ui, |ui| {
            for (player, skill) in &players {
                ui.label(format!("{}", player.id));
                ui.label(format!("{}", player.controller));
                ui.end_row();
                ui.label("Skill");
                if let Some(skill) = skill.active {
                    ui.label(format!("{}", skill));
                }
                ui.end_row();
            }
        });
    });
}
