use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::Dead;

use super::garbage::GarbageItem;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .register_type::<ParticleConfig>()
            .init_resource::<ParticleConfig>()
            .register_type::<DestructionInstructions>()
            .init_resource::<DestructionInstructions>()
            .add_systems(
                Update,
                (trigger_destroy_particles, apply_destruction_particles),
            );

        #[cfg(feature = "debug")]
        app.add_systems(PostUpdate, draw_gizmos);
    }
}

/// Differed particle instructions to support same frame triggers
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct DestructionInstructions {
    /// Position, color as a u32
    data: Vec<(Vec3, u32)>,
}

fn trigger_destroy_particles(
    targets: Query<(&GlobalTransform, &GarbageItem), Added<Dead>>,
    mut instructions: ResMut<DestructionInstructions>,
) {
    for (gtr, item) in &targets {
        let [r, g, b, a] = item.color().to_srgba().to_u8_array();
        let color = (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32);
        instructions.data.push((gtr.translation(), color));
    }
}

fn apply_destruction_particles(
    mut instructions: ResMut<DestructionInstructions>,
    emitters: Res<ParticleConfig>,
    mut particles: Query<(&mut EffectProperties, &mut EffectSpawner, &mut Transform)>,
) {
    let Some((pos, color)) = instructions.data.pop() else {
        return;
    };
    let Ok((mut properties, mut spawner, mut particle_tr)) =
        particles.get_mut(emitters.destruction_emitter)
    else {
        return;
    };
    particle_tr.translation = pos;
    properties.set("color", color.into());
    spawner.reset();
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ParticleConfig {
    pub destruction_emitter: Entity,
    // pub damage: Entity,
}

impl FromWorld for ParticleConfig {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        let texture = server.load("kenney_particle-pack/png/circle_05.png");
        let mut assets = world.resource_mut::<Assets<EffectAsset>>();
        // Set `spawn_immediately` to false to spawn on command with Spawner::reset()
        let spawner = Spawner::once(50.0.into(), false);
        let mut size_gradient = Gradient::new();
        size_gradient.add_key(0.0, Vec2::splat(0.05)); // Start size
        size_gradient.add_key(0.1, Vec2::splat(0.8)); // Start size
        size_gradient.add_key(1.0, Vec2::splat(0.0)); // End size

        let writer = ExprWriter::new();
        // Bind the initial particle color to the value of the 'spawn_color' property
        // when the particle spawns. The particle will keep that color afterward,
        // even if the property changes, because the color will be saved
        // per-particle (due to the Attribute::COLOR).
        let spawn_color = writer.add_property("color", 0xFFFFFFFFu32.into());
        let color = writer.prop(spawn_color).expr();
        let init_color = SetAttributeModifier::new(Attribute::COLOR, color);

        let init_pos = SetPositionSphereModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            radius: writer.lit(0.5).expr(),
            dimension: ShapeDimension::Volume,
        };
        let init_vel = SetVelocitySphereModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            speed: writer.lit(3.0).expr(),
        };
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        // Give a bit of variation by randomizing the lifetime per particle
        let lifetime = writer.lit(0.5).uniform(writer.lit(1.5)).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
        let update_size = SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        };
        let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, -9.8, 0.0)).expr());
        let render_texture = ParticleTextureModifier {
            texture,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        };

        let effect = EffectAsset::new(vec![32768], spawner, writer.finish())
            .with_name("Object Destruction")
            .init(init_color)
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_accel)
            .render(update_size)
            .render(render_texture)
            .render(OrientModifier {
                mode: OrientMode::FaceCameraPosition,
                rotation: None,
            });

        let effect_handle = assets.add(effect);
        let destruction = world.spawn(ParticleEffectBundle::new(effect_handle)).id();
        Self {
            destruction_emitter: destruction,
        }
    }
}

#[cfg(feature = "debug")]
pub fn draw_gizmos(mut gizmos: Gizmos, effects: Query<(&GlobalTransform, &EffectProperties)>) {
    for (gtr, props) in &effects {
        let color = props
            .get_stored("color")
            .map(|v| {
                let color = v.as_scalar().as_u32();
                let r = (color & 0x000000FF) as u8;
                let g = ((color & 0x0000FF00) >> 8) as u8;
                let b = ((color & 0x00FF0000) >> 16) as u8;
                let a = ((color & 0xFF000000) >> 24) as u8;
                Color::Srgba(Srgba::from_u8_array([r, g, b, a]))
            })
            .unwrap_or(Color::BLACK);
        gizmos.sphere(gtr.translation(), Quat::IDENTITY, 1.0, color);
    }
}
