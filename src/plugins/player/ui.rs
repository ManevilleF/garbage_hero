use std::f32::consts::FRAC_PI_6;

use super::{assets::PlayerAssets, Player, PlayerInput};
use crate::{plugins::ui::input_icons::InputMapIcons, GameState, Health};
use bevy::{prelude::*, utils::HashMap};

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiState>()
            .register_type::<HealthUi>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                PostUpdate,
                (create_player_ui, update_health, update_input_icons),
            )
            .add_systems(OnEnter(GameState::Pause), toggle_controls)
            .add_systems(OnExit(GameState::Pause), toggle_controls);
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct UiState {
    bottom_root_node: Entity,
    controls_root_node: Entity,
}

fn toggle_controls(
    state: Res<State<GameState>>,
    ui: Res<UiState>,
    mut items: Query<&mut Visibility, With<Node>>,
) {
    let Ok(mut visibility) = items.get_mut(ui.controls_root_node) else {
        return;
    };
    *visibility = if state.get() == &GameState::Pause {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

#[derive(Component, Reflect)]
#[reflect(Component)]
// Player -> Ui
struct HealthUi(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
// Player -> Ui
struct PlayerInputUI(HashMap<PlayerInput, Entity>);

fn setup_ui(mut commands: Commands) {
    let controls_root_node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(125.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            Name::new("Player Controls Root"),
        ))
        .id();
    let bottom_root_node = commands
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
    commands.insert_resource(UiState {
        bottom_root_node,
        controls_root_node,
    });
}

fn update_input_icons(
    players: Query<(&PlayerInputUI, &InputMapIcons), Changed<InputMapIcons>>,
    mut ui: Query<&mut UiImage>,
) {
    for (ui_entities, icons) in &players {
        for (input, image_entity) in &ui_entities.0 {
            let Ok(image) = ui.get_mut(*image_entity) else {
                return;
            };
            let Some(handle) = icons.input_icons.get(input) else {
                continue;
            };
            let mut texture = image.map_unchanged(|i| &mut i.texture);
            texture.set_if_neq(handle.clone_weak());
        }
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

fn create_player_ui(
    mut commands: Commands,
    new_players: Query<(Entity, &Player, &InputMapIcons), Added<Player>>,
    state: Res<UiState>,
    assets: Res<PlayerAssets>,
) {
    for (entity, player, icons) in &new_players {
        let color = assets.colors[player.id as usize];

        // BOTTOM
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
                Name::new(format!("Player {} Bar Root node", player.id)),
            ))
            .set_parent(state.bottom_root_node)
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

        // CONTROLS
        let root = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Px(120.0),
                        bottom: Val::Px(0.0),
                        margin: UiRect::horizontal(Val::Px(15.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::BLACK.with_alpha(0.8)),
                    border_color: BorderColor(color),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                Name::new(format!("Player {} Controls Root node", player.id)),
            ))
            .set_parent(state.controls_root_node)
            .id();
        let mut controls_map = HashMap::with_capacity(icons.input_icons.len());
        for (input, icon) in &icons.input_icons {
            let input_root = commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            height: Val::Px(50.0),
                            width: Val::Percent(100.0),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        ..default()
                    },
                    Name::new(format!("{input}")),
                ))
                .set_parent(root)
                .id();
            commands
                .spawn((
                    TextBundle {
                        style: Style {
                            width: Val::Px(80.0),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection {
                                value: format!("{}", input),
                                style: TextStyle {
                                    font_size: 15.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            }],
                            justify: JustifyText::Left,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Text"),
                ))
                .set_parent(input_root);
            let image = commands
                .spawn((
                    ImageBundle {
                        style: Style {
                            height: Val::Px(40.0),
                            width: Val::Px(40.0),
                            right: Val::Px(0.0),
                            ..default()
                        },
                        image: UiImage {
                            texture: icon.clone_weak(),
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Icon"),
                ))
                .set_parent(input_root)
                .id();
            controls_map.insert(*input, image);
        }
        commands.entity(entity).insert(PlayerInputUI(controls_map));
    }
}
