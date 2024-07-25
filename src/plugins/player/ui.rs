use std::f32::consts::FRAC_PI_6;

use super::{assets::PlayerAssets, Player};
use crate::{plugins::ui::input_icons::InputMapIcons, Health};
use bevy::prelude::*;

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiState>()
            .register_type::<HealthUi>()
            .add_systems(Startup, setup_ui)
            .add_systems(PostUpdate, (create_player_ui, update_health));
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct UiState {
    root_node: Entity,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
// Player -> Ui
struct HealthUi(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
// Ui -> Player
struct PlayerUI(Entity);

fn setup_ui(mut commands: Commands) {
    let root_node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    height: Val::Px(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                ..default()
            },
            Name::new("Player UI Root"),
        ))
        .id();
    commands.insert_resource(UiState { root_node });
}

fn create_player_ui(
    mut commands: Commands,
    new_players: Query<(Entity, &Player, &InputMapIcons), Added<Player>>,
    state: Res<UiState>,
    assets: Res<PlayerAssets>,
) {
    for (entity, player, icons) in &new_players {
        let color = assets.colors[player.id as usize];
        let root = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexEnd,
                        height: Val::Percent(80.0),
                        width: Val::Px(100.0),
                        margin: UiRect::horizontal(Val::Px(25.0)),
                        ..default()
                    },
                    ..default()
                },
                Name::new(format!("Player {} Root node", player.id)),
                PlayerUI(entity),
            ))
            .set_parent(state.root_node)
            .id();
        commands
            .spawn((
                TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(45.0),
                        bottom: Val::Px(30.0),
                        left: Val::Px(0.0),

                        ..default()
                    },
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("P{}", player.id),
                            style: TextStyle {
                                font_size: 20.0,
                                color,
                                ..default()
                            },
                        }],
                        justify: JustifyText::Left,
                        ..default()
                    },
                    ..default()
                },
                Name::new("Player text"),
            ))
            .set_parent(root);
        let health_root = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(5.0)),
                        bottom: Val::Px(10.0),
                        left: Val::Px(0.0),
                        ..default()
                    },
                    border_color: BorderColor(color),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    z_index: ZIndex::Local(10),
                    ..default()
                },
                Name::new("Health Root"),
            ))
            .set_parent(root)
            .id();
        let health_ui = commands
            .spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    image: UiImage { color, ..default() },
                    background_color: BackgroundColor(Color::WHITE),
                    ..default()
                },
                Name::new("Health"),
            ))
            .set_parent(health_root)
            .id();
        commands.entity(entity).insert(HealthUi(health_ui));
        commands
            .spawn((
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        height: Val::Px(80.0),
                        width: Val::Px(80.0),
                        left: Val::Percent(45.0),
                        ..default()
                    },
                    image: UiImage {
                        color,
                        texture: icons.controller_icon.clone_weak(),
                        ..default()
                    },
                    transform: Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_6)),
                    ..default()
                },
                Name::new("Controller Icon"),
            ))
            .set_parent(root);
    }
}

fn update_health(
    health: Query<(&Health, &HealthUi), Changed<Health>>,
    mut ui: Query<&mut Style, With<UiImage>>,
) {
    for (health, HealthUi(ui_entity)) in &health {
        let Ok(mut style) = ui.get_mut(*ui_entity) else {
            return;
        };
        style.width = Val::Percent(health.ratio() * 100.0);
    }
}
