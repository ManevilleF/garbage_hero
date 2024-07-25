use std::ops::DerefMut;

use super::player::Player;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    ecs::system::SystemParam,
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::camera::ScalingMode,
    window::PrimaryWindow,
};

const CAM_SCALE_COEF: f32 = 0.001;
const CAM_MIN_SCALE: f32 = 0.05;
const CAM_OFFSET: Vec3 = Vec3::new(0.0, 30.0, -30.0);
/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameCamera>()
            .add_systems(Startup, spawn_camera)
            .add_systems(PostUpdate, follow_players);

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, draw_gizmos);
    }
}

#[derive(Component, Reflect)]
pub struct GameCamera;

#[derive(SystemParam)]
pub struct CameraParams<'w, 's> {
    pub camera: Query<'w, 's, (&'static GlobalTransform, &'static Camera), With<GameCamera>>,
    pub window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

impl<'w, 's> CameraParams<'w, 's> {
    pub fn mouse_ray(&self) -> Option<Ray3d> {
        let (cam_gtr, camera) = self.camera.single();
        self.window
            .get_single()
            .ok()
            .and_then(|w| w.cursor_position())
            .and_then(|p| camera.viewport_to_world(cam_gtr, p))
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(CAM_OFFSET).looking_at(Vec3::ZERO, Dir3::Y),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::WindowSize(1.0),
                scale: CAM_MIN_SCALE,
                near: -100.0,
                ..default()
            }),
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        BloomSettings::default(),
        ScreenSpaceAmbientOcclusionBundle::default(),
        Name::new("Game Camera"),
        GameCamera,
        IsDefaultUiCamera,
        #[cfg(feature = "editor")]
        transform_gizmo_bevy::GizmoCamera,
    ));
}

pub fn follow_players(
    time: Res<Time>,
    players: Query<&GlobalTransform, With<Player>>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<GameCamera>>,
) {
    let Ok((mut cam_tr, mut projection)) = cameras.get_single_mut() else {
        return;
    };
    let Projection::Orthographic(projection) = projection.deref_mut() else {
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
    let target = Vec3::new(center.x, 0.0, center.y) + CAM_OFFSET;
    cam_tr.translation = cam_tr.translation.lerp(target, dt * CAMERA_DECAY_RATE);
    // Projection
    let size = max - min;
    let target = (size.max_element() * CAM_SCALE_COEF).max(CAM_MIN_SCALE);
    projection.scale = projection.scale.lerp(target, dt * CAMERA_DECAY_RATE);
}

#[cfg(feature = "debug")]
fn draw_gizmos(mut gizmos: Gizmos, camera: CameraParams) {
    use bevy::color::palettes::css::GREEN;

    let Some(ray) = camera.mouse_ray() else {
        return;
    };
    gizmos.ray(ray.origin, *ray.direction, Color::Srgba(GREEN))
}
