use std::f32::consts::{FRAC_PI_2, PI};

use crate::plugins::common::Health;
use crate::ObjectLayer;
use avian3d::prelude::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Copy, Clone, Component, EnumIter, Reflect, Display)]
#[non_exhaustive]
#[repr(u8)]
#[reflect(Component)]
pub enum GarbageItem {
    WoodenCrate,
    Barrel,
    ExplosiveBarrel,
    SmallRock,
    MediumRock,
    BigRock,
    Gear,
    Pipe,
    Bottle,
    PoisonVial,
    FirePot,
}

impl GarbageItem {
    pub const fn base_health(self) -> Health {
        match self {
            Self::WoodenCrate => Health::new(10),
            Self::Barrel => Health::new(5),
            Self::ExplosiveBarrel => Health::new(5),
            Self::SmallRock => Health::new(10),
            Self::MediumRock => Health::new(20),
            Self::BigRock => Health::new(30),
            Self::Gear => Health::new(50),
            Self::Pipe => Health::new(50),
            Self::Bottle => Health::new(1),
            Self::PoisonVial => Health::new(1),
            Self::FirePot => Health::new(1),
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::WoodenCrate => BURLYWOOD.into(),
            Self::Barrel => SIENNA.into(),
            Self::ExplosiveBarrel => FIRE_BRICK.into(),
            Self::SmallRock => DIM_GRAY.into(),
            Self::MediumRock => DARK_SLATE_GRAY.into(),
            Self::BigRock => SLATE_GRAY.into(),
            Self::Gear => STEEL_BLUE.into(),
            Self::Pipe => LIGHT_SLATE_GRAY.into(),
            Self::Bottle => AZURE.into(),
            Self::PoisonVial => MEDIUM_SPRING_GREEN.into(),
            Self::FirePot => ORANGE_RED.into(),
        }
    }

    pub fn mesh(self) -> Mesh {
        match self {
            Self::WoodenCrate => Cuboid::from_size(Vec3::ONE).into(),
            Self::Barrel | Self::ExplosiveBarrel => Cylinder::new(0.5, 1.0).into(),
            Self::SmallRock => Sphere::new(0.2).mesh().ico(10).unwrap(),
            Self::MediumRock => Sphere::new(0.6).mesh().ico(12).unwrap(),
            Self::BigRock => Sphere::new(1.0).mesh().ico(15).unwrap(),
            Self::Gear => Extrusion::new(Annulus::new(0.8, 1.0), 0.5).into(),
            Self::Pipe => Cylinder::new(0.2, 1.0).into(),
            Self::Bottle => Capsule3d::new(0.15, 0.4).into(),
            Self::PoisonVial | Self::FirePot => Sphere::new(0.1).into(),
        }
    }

    pub fn collider(self) -> Collider {
        match self {
            Self::WoodenCrate => Collider::cuboid(1.0, 1.0, 1.0),
            Self::Barrel | Self::ExplosiveBarrel => Collider::cylinder(0.5, 1.0),
            Self::SmallRock => Collider::sphere(0.2),
            Self::MediumRock => Collider::sphere(0.6),
            Self::BigRock => Collider::sphere(1.0),
            Self::Gear => Collider::compound(vec![(
                Vec3::ZERO,
                Quat::from_rotation_x(FRAC_PI_2),
                Collider::cylinder(1.0, 0.5),
            )]),
            Self::Pipe => Collider::cylinder(0.1, 1.0),
            Self::Bottle => Collider::capsule(0.1, 0.4),
            Self::PoisonVial | Self::FirePot => Collider::sphere(0.1),
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
            ang_damping: AngularDamping(1.0),
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
