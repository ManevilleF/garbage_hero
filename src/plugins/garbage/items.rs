use std::f32::consts::FRAC_PI_2;

use crate::plugins::common::Health;
use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy_mod_outline::OutlineMeshExt;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Copy, Clone, Component, EnumIter, Reflect, Display)]
#[non_exhaustive]
#[repr(u8)]
#[reflect(Component)]
pub enum GarbageItem {
    /// 1x1
    Cube,
    /// 2x0.5
    Plank,
    /// 3x0.5
    LargePlank,
    /// 2x1
    Column,
    /// 3x1
    LargeColumn,
    /// 2x2
    Gear,
    /// 2x1
    Block,
    /// 2x1
    Cone,
    /// 3x1
    LargeBlock,
    /// 1x1
    Ball,
}

impl GarbageItem {
    pub const fn base_health(self) -> Health {
        match self {
            Self::Cube => Health::new(10),
            Self::Plank => Health::new(10),
            Self::LargePlank => Health::new(15),
            Self::Column => Health::new(20),
            Self::LargeColumn => Health::new(30),
            Self::Gear => Health::new(20),
            Self::Block => Health::new(20),
            Self::Cone => Health::new(15),
            Self::LargeBlock => Health::new(30),
            Self::Ball => Health::new(20),
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Cube => SANDY_BROWN.into(),
            Self::Plank => PERU.into(),
            Self::LargePlank => CHOCOLATE.into(),
            Self::Column => SIENNA.into(),
            Self::LargeColumn => SADDLE_BROWN.into(),
            Self::Gear => GOLDENROD.into(),
            Self::Block => DARK_GOLDENROD.into(),
            Self::Cone => DARK_ORANGE.into(),
            Self::LargeBlock => CORAL.into(),
            Self::Ball => TOMATO.into(),
        }
    }

    pub fn mesh(self) -> Mesh {
        match self {
            Self::Cube => Cuboid::from_size(Vec3::ONE).into(),
            Self::Plank => Cuboid::new(2.0, 0.5, 1.0).into(),
            Self::LargePlank => Cuboid::new(3.0, 0.5, 1.0).into(),
            Self::Column => Cylinder::new(0.5, 2.0).into(),
            Self::LargeColumn => Cylinder::new(0.5, 3.0).into(),
            Self::Gear => Extrusion::new(Annulus::new(0.8, 1.0), 0.5).into(),
            Self::Cone => Cone {
                radius: 1.0,
                height: 1.5,
            }
            .into(),
            Self::Block => Cuboid::new(1.0, 2.0, 1.0).into(),
            Self::LargeBlock => Cuboid::new(1.0, 3.0, 1.0).into(),
            Self::Ball => Sphere::new(1.0).mesh().ico(24).unwrap(),
        }
    }

    pub fn collider(self) -> Collider {
        match self {
            Self::Cube => Collider::cuboid(1.0, 1.0, 1.0),
            Self::Plank => Collider::cuboid(2.0, 0.5, 1.0),
            Self::LargePlank => Collider::cuboid(3.0, 0.5, 1.0),
            Self::Column => Collider::cylinder(0.5, 2.0),
            Self::LargeColumn => Collider::cylinder(0.5, 3.0),
            Self::Gear => Collider::compound(vec![(
                Vec3::ZERO,
                Quat::from_rotation_x(FRAC_PI_2),
                Collider::cylinder(1.0, 0.5),
            )]),
            Self::Cone => Collider::cone(1.0, 1.5),
            Self::Block => Collider::cuboid(1.0, 2.0, 1.0),
            Self::LargeBlock => Collider::cuboid(1.0, 3.0, 1.0),
            Self::Ball => Collider::sphere(1.0),
        }
    }
}

#[derive(Bundle)]
pub struct GarbageBundle {
    pub collectible: GarbageItem,
    pub health: Health,
    pub pbr: PbrBundle,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub margin: CollisionMargin,
    pub layer: CollisionLayers,
    pub ang_damping: AngularDamping,
    pub gravity_scale: GravityScale,
    pub name: Name,
}

impl GarbageBundle {
    pub fn new(collectible: GarbageItem, assets: &GarbageAssets) -> Self {
        Self {
            health: collectible.base_health(),
            collectible,
            pbr: PbrBundle {
                mesh: assets.meshes[collectible as usize].clone_weak(),
                material: assets.materials[collectible as usize].clone_weak(),
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            collider: assets.colliders[collectible as usize].clone(),
            margin: CollisionMargin(0.05),
            ang_damping: AngularDamping(1.5),
            layer: CollisionLayers::new(ObjectLayer::Collectible, LayerMask::ALL),
            gravity_scale: GravityScale(1.0),
            name: Name::new(format!("{collectible}")),
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GarbageAssets {
    pub meshes: Vec<Handle<Mesh>>,
    #[reflect(ignore)]
    pub colliders: Vec<Collider>,
    pub materials: Vec<Handle<StandardMaterial>>,
}

impl FromWorld for GarbageAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let base_material = StandardMaterial {
            fog_enabled: true,
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        };
        let materials = GarbageItem::iter()
            .map(|c| {
                materials.add(StandardMaterial {
                    base_color: c.color(),
                    ..base_material.clone()
                })
            })
            .collect();
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let meshes = GarbageItem::iter().map(|c| meshes.add(c.mesh())).collect();
        let colliders = GarbageItem::iter().map(GarbageItem::collider).collect();
        Self {
            meshes,
            materials,
            colliders,
        }
    }
}
