use bevy::{color::palettes::css::*, pbr::DirectionalLightShadowMap, prelude::*};

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // UI cam
    commands.insert_resource(ClearColor(Color::from(ANTIQUE_WHITE)));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: light_consts::lux::OVERCAST_DAY,
    });
    // Light
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_xyz(-15.0, 50.0, -15.0).looking_at(Vec3::ZERO, Vec3::Y),
            directional_light: DirectionalLight {
                illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
        Name::new("Sun Light"),
    ));
}
