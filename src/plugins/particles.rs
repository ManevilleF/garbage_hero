use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::Dead;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .register_type::<ParticleConfig>()
            .init_resource::<ParticleConfig>()
            .register_type::<DeathEffect>()
            .register_type::<DestructionInstructions>()
            .init_resource::<DestructionInstructions>()
            .add_systems(
                Update,
                (trigger_destroy_particles, apply_destruction_particles),
            );

        // #[cfg(feature = "debug")]
        // app.add_systems(
        //     PostUpdate,
        //     draw_gizmos
        //         .after(avian3d::prelude::PhysicsSet::Sync)
        //         .before(TransformSystem::TransformPropagate),
        // );
    }
}

/// Differed particle instructions to support same frame triggers
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct DestructionInstructions {
    /// Position, color as a u32
    data: Vec<(Vec3, DeathEffect)>,
}

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct DeathEffect {
    pub color: Color,
    pub radius: f32,
}

fn trigger_destroy_particles(
    targets: Query<(&GlobalTransform, &DeathEffect), Added<Dead>>,
    mut instructions: ResMut<DestructionInstructions>,
) {
    for (gtr, effect) in &targets {
        instructions.data.push((gtr.translation(), *effect));
    }
}

fn apply_destruction_particles(
    mut instructions: ResMut<DestructionInstructions>,
    emitters: Res<ParticleConfig>,
    mut particles: Query<(&mut EffectProperties, &mut EffectSpawner, &mut Transform)>,
) {
    let Some((pos, effect)) = instructions.data.pop() else {
        return;
    };
    let Ok((mut properties, mut spawner, mut particle_tr)) =
        particles.get_mut(emitters.destruction_emitter)
    else {
        return;
    };
    particle_tr.translation = pos;
    properties.set("color", ParticleConfig::color_to_value(effect.color));
    properties.set("radius", effect.radius.into());
    spawner.reset();
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ParticleConfig {
    pub destruction_emitter: Entity,
    pub collector_effect: Handle<EffectAsset>, // pub damage: Entity,
}

impl ParticleConfig {
    pub fn color_to_value(color: Color) -> Value {
        let [r, g, b, a] = color.to_srgba().to_u8_array();
        ((a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32)).into()
    }

    pub fn color_from_value(value: Value) -> Color {
        let color = value.as_scalar().as_u32();
        let r = (color & 0x000000FF) as u8;
        let g = ((color & 0x0000FF00) >> 8) as u8;
        let b = ((color & 0x00FF0000) >> 16) as u8;
        let a = ((color & 0xFF000000) >> 24) as u8;
        Color::Srgba(Srgba::from_u8_array([r, g, b, a]))
    }

    fn collector_effect() -> EffectAsset {
        let mut size_gradient = Gradient::new();
        size_gradient.add_key(0.0, Vec2::new(0.0, 0.0));
        size_gradient.add_key(0.3, Vec2::new(1.0, 0.05));
        size_gradient.add_key(1.0, Vec2::splat(0.0));

        let writer = ExprWriter::new();

        let spawn_color = writer.add_property("color", 0xFFFFFFFFu32.into());
        let color = writer.prop(spawn_color).expr();
        let init_color = SetAttributeModifier::new(Attribute::COLOR, color);

        let radius_prop = writer.add_property("radius", 3.0_f32.into());
        let radius = writer.prop(radius_prop).expr();

        let init_pos = SetPositionCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            radius,
            dimension: ShapeDimension::Surface,
        };

        let age = writer.lit(0.).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        // Give a bit of variation by randomizing the lifetime per particle
        let lifetime = writer.lit(1.0).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

        let init_tangent = SetVelocityTangentModifier {
            origin: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            speed: writer.lit(-5.0).expr(),
        };
        // Create some effect by adding some radial acceleration pointing at
        // the origin (0,0,0) and some upward acceleration (alongside Y).
        let speed = writer
            .prop(radius_prop)
            .mul(writer.lit(-1.0))
            .add(writer.lit(-10.0));
        let radial = writer.attr(Attribute::POSITION).normalized().mul(speed);
        let accel = radial;
        let update_accel = AccelModifier::new(accel.expr());

        // Add drag to make particles slow down a bit after the initial acceleration
        let drag = writer.lit(1.).expr();
        let update_drag = LinearDragModifier::new(drag);

        EffectAsset::new(vec![16384], Spawner::rate(100.0.into()), writer.finish())
            .with_name("Collector")
            .init(init_color)
            .init(init_pos)
            .init(init_age)
            .init(init_lifetime)
            .init(init_tangent)
            .update(update_drag)
            .update(update_accel)
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            })
            .render(OrientModifier::new(OrientMode::AlongVelocity))
            .with_simulation_space(SimulationSpace::Local)
    }

    fn destruction_effect() -> EffectAsset {
        // Set `spawn_immediately` to false to spawn on command with Spawner::reset()
        let spawner = Spawner::once(100.0.into(), false);
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

        let radius_prop = writer.add_property("radius", 1.0_f32.into());
        let radius = writer.prop(radius_prop).expr();

        let init_pos = SetPositionSphereModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            radius,
            dimension: ShapeDimension::Volume,
        };
        let init_vel = SetVelocitySphereModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            speed: writer.lit(5.0).expr(),
        };
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        // Give a bit of variation by randomizing the lifetime per particle
        let lifetime = writer.lit(1.0).uniform(writer.lit(2.0)).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
        let update_size = SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        };
        let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, -9.8, 0.0)).expr());

        let texture_slot = writer.lit(0u32).expr();
        let render_texture = ParticleTextureModifier {
            texture_slot,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        };

        let mut module = writer.finish();
        module.add_texture("texture");

        EffectAsset::new(vec![32768], spawner, module)
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
            })
    }
}

impl FromWorld for ParticleConfig {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        let texture = server.load("kenney_particle-pack/png/circle_05.png");
        let mut assets = world.resource_mut::<Assets<EffectAsset>>();

        let collector_effect = assets.add(Self::collector_effect());
        let destruction_handle = assets.add(Self::destruction_effect());
        let destruction_emitter = world
            .spawn((
                ParticleEffectBundle::new(destruction_handle),
                EffectMaterial {
                    images: vec![texture],
                },
                Name::new("Destruction Emitter"),
            ))
            .id();

        Self {
            destruction_emitter,
            collector_effect,
        }
    }
}

#[cfg(feature = "debug")]
pub fn draw_gizmos(mut gizmos: Gizmos, effects: Query<(&GlobalTransform, &EffectProperties)>) {
    for (gtr, props) in &effects {
        let color = props
            .get_stored("color")
            .map(ParticleConfig::color_from_value)
            .unwrap_or(Color::BLACK);
        gizmos.sphere(gtr.translation(), Quat::IDENTITY, 1.0, color);
    }
}
