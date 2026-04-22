use bevy::prelude::*;

pub struct SplashScreenPlugin;

const SPLASH_DURATION: f32 = 3.0;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SplashScreen>()
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                stop_splash_screen.run_if(resource_exists::<SplashScreen>),
            );
    }
}

#[derive(Resource, Reflect)]
struct SplashScreen(Entity);

pub fn setup(mut commands: Commands, server: Res<AssetServer>) {
    let image = server.load("splash_screen.png");
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|cmd| {
            cmd.spawn((
                Node {
                    width: Val::Px(700.0),
                    ..default()
                },
                ImageNode {
                    image,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        })
        .id();
    commands.insert_resource(SplashScreen(entity));
}

fn stop_splash_screen(mut commands: Commands, screen: Res<SplashScreen>, time: Res<Time>) {
    if time.elapsed_secs() > SPLASH_DURATION {
        commands.entity(screen.0).despawn();
        commands.remove_resource::<SplashScreen>()
    }
}
