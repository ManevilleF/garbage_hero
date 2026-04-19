use bevy::{color::palettes::css::*, light::DirectionalLightShadowMap, prelude::*};

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
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: light_consts::lux::OVERCAST_DAY,
        affects_lightmapped_meshes: true,
    });
    // Light
    commands.spawn((
        Transform::from_xyz(15.0, 50.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Name::new("Sun Light"),
    ));
}
