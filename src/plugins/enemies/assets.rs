use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::Health;

use super::Enemy;

pub struct EnemyAssetsPlugin;

impl Plugin for EnemyAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyAssets>()
            .add_systems(PostUpdate, update_materials);
    }
}

#[derive(Resource)]
pub struct EnemyAssets {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; 5],
    pub collider: Collider,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let worm_head_mesh = meshes.add(Sphere::new(1.0));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let worm_head_mat = [255, 154, 103, 52, 1].map(|r| {
            materials.add(StandardMaterial {
                base_color: Color::srgb_u8(r, 0, 0),
                unlit: true,
                ..default()
            })
        });
        let worm_head_collider = Collider::sphere(1.0);
        Self {
            mesh: worm_head_mesh,
            materials: worm_head_mat,
            collider: worm_head_collider,
        }
    }
}

fn update_materials(
    mut materials: Query<(&mut Handle<StandardMaterial>, &Health), (With<Enemy>, Changed<Health>)>,
    assets: Res<EnemyAssets>,
) {
    for (mut mat, health) in &mut materials {
        let ratio = health.ratio();
        let index = (ratio * (assets.materials.len().saturating_sub(1) as f32)).round() as usize;
        *mat = assets.materials[index].clone_weak();
    }
}
