use super::{GarbageAssets, GarbageBundle, GarbageItem};
use bevy::{
    asset::{AssetLoader, AsyncReadExt, LoadedFolder, RecursiveDependencyLoadState},
    ecs::world::Command,
    prelude::*,
    utils::HashMap,
};
use thiserror::Error;

pub struct ItemBuildsPlugin;

impl Plugin for ItemBuildsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ItemBuild>()
            .preregister_asset_loader::<BuildLoader>(&["build"])
            .init_asset_loader::<BuildLoader>()
            .init_resource::<AvailableItemBuilds>()
            .add_systems(Startup, setup_builds)
            .add_systems(
                Update,
                tick_loading_builds.run_if(resource_exists::<LoadingFolder>),
            );
    }
}

#[derive(Clone, Asset, TypePath, Debug)]
pub struct ItemBuild {
    items: Vec<(GarbageItem, Vec2)>,
}

impl ItemBuild {
    pub fn parse(bytes: &[u8]) -> Result<Self, BuildAssetError> {
        let mut items = Vec::new();
        let content = std::str::from_utf8(bytes)?;
        let layers = content.lines().rev().enumerate();

        for (y, layer) in layers {
            for (x, c) in layer.chars().enumerate() {
                if let Some(item) = GarbageItem::from_char(c) {
                    items.push((item, Vec2::new(x as f32, y as f32)));
                }
            }
        }

        Ok(Self { items })
    }

    pub fn spawn(self, position: Vec3, angle: f32) -> impl FnOnce(&mut World) {
        let transform =
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(angle));
        move |world| {
            let assets = world.resource::<GarbageAssets>();
            let bundles: Vec<_> = self
                .items
                .iter()
                .map(|(item, pos)| {
                    let pos = Vec3::new(pos.x * 1.01, pos.y * 1.01, 0.0);
                    let mut bundle = GarbageBundle::new(*item, assets);
                    bundle.pbr.transform.translation = transform.transform_point(pos);
                    bundle.pbr.transform.rotation = transform.rotation;
                    bundle
                })
                .collect();
            world.spawn_batch(bundles);
        }
    }
}

#[derive(Debug)]
pub struct SpawnBuild {
    pub handle: Handle<ItemBuild>,
    pub position: Vec3,
    pub angle: f32,
}

impl Command for SpawnBuild {
    fn apply(self, world: &mut World) {
        let builds = world.resource::<Assets<ItemBuild>>();
        let build = builds.get(self.handle.id()).unwrap().clone();
        build.spawn(self.position, self.angle)(world);
    }
}

#[derive(Resource, Deref, Default)]
pub struct AvailableItemBuilds(HashMap<String, Handle<ItemBuild>>);

#[derive(Resource)]
struct LoadingFolder(Handle<LoadedFolder>);

fn setup_builds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingFolder(asset_server.load_folder("builds")));
}

fn tick_loading_builds(
    mut commands: Commands,
    mut builds: ResMut<AvailableItemBuilds>,
    loading: Res<LoadingFolder>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<LoadedFolder>>,
) {
    if Some(RecursiveDependencyLoadState::Loaded)
        == asset_server.get_recursive_dependency_load_state(loading.0.id())
    {
        let folder: &LoadedFolder = assets.get(&loading.0).unwrap();
        builds.0.extend(folder.handles.iter().cloned().map(|h| {
            let path = h.path().unwrap().to_string();
            let build = h.typed::<ItemBuild>();
            (path, build)
        }));
        commands.remove_resource::<LoadingFolder>();
    }
}

#[derive(Default)]
pub struct BuildLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BuildAssetError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl AssetLoader for BuildLoader {
    type Asset = ItemBuild;
    type Settings = ();
    type Error = BuildAssetError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ItemBuild::parse(&bytes)?;
        Ok(asset)
    }
}

impl GarbageItem {
    pub const fn from_char(c: char) -> Option<Self> {
        let item = match c {
            'c' => Self::Cube,
            'p' => Self::Plank,
            'P' => Self::LargePlank,
            '|' => Self::Column,
            'I' => Self::LargeColumn,
            'g' => Self::Gear,
            'b' => Self::Block,
            'B' => Self::LargeBlock,
            '^' => Self::Cone,
            'o' => Self::Ball,
            _ => return None,
        };
        Some(item)
    }
}
