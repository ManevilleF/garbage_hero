use avian3d::prelude::Collider;
use bevy::{color::palettes::css::RED, prelude::*};

#[derive(Resource)]
pub struct EnemyAssets {
    pub worm_head_mesh: Handle<Mesh>,
    pub worm_head_mat: Handle<StandardMaterial>,
    pub worm_head_collider: Collider,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let worm_head_mesh = meshes.add(Sphere::new(1.0));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let worm_head_mat = materials.add(StandardMaterial {
            base_color: Color::Srgba(RED),
            emissive: LinearRgba::rgb(20.0, 0.0, 5.0),
            unlit: true,
            ..default()
        });
        let worm_head_collider = Collider::sphere(1.0);
        Self {
            worm_head_mesh,
            worm_head_mat,
            worm_head_collider,
        }
    }
}
