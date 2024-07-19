use super::player::Player;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::camera::ScalingMode,
};

const CAM_SCALE_COEF: f32 = 1.0;
const CAM_SCALE_MARGIN: f32 = 1.0;
const CAM_MIN_SCALE: f32 = 20.0;
const CAM_Y_OFFSET: f32 = 20.0;
/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameCamera>()
            .add_systems(Startup, spawn_camera)
            .add_systems(PostUpdate, follow_players);
    }
}

#[derive(Component, Reflect)]
pub struct GameCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, CAM_Y_OFFSET, 10.0)
                .looking_at(Vec3::ZERO, Dir3::Y),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::WindowSize(1.0),
                scale: CAM_MIN_SCALE,
                ..default()
            }),
            tonemapping: Tonemapping::ReinhardLuminance,
            ..default()
        },
        BloomSettings::default(),
        ScreenSpaceAmbientOcclusionBundle::default(),
        Name::new("Game Camera"),
        GameCamera,
    ));
}

pub fn follow_players(
    time: Res<Time>,
    players: Query<&GlobalTransform, With<Player>>,
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection), With<GameCamera>>,
) {
    let Ok((mut cam_tr, mut projection)) = cameras.get_single_mut() else {
        return;
    };
    let mut min = Vec2::MAX;
    let mut max = Vec2::MIN;
    for gtr in &players {
        let pos = gtr.translation().xz();
        min = min.min(pos);
        max = max.max(pos);
    }
    let dt = time.delta_seconds();
    // Translation
    let center = (max + min) / 2.0;
    let target = Vec3::new(center.x, cam_tr.translation.y, center.y);
    cam_tr.translation = cam_tr.translation.lerp(target, dt * CAMERA_DECAY_RATE);
    // Projection
    let size = max - min;
    let target = size
        .max_element()
        .max(CAM_MIN_SCALE)
        .mul_add(CAM_SCALE_COEF, CAM_SCALE_MARGIN);
    projection.scale = projection.scale.lerp(target, dt * CAMERA_DECAY_RATE);
}
