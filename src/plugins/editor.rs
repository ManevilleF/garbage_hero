use bevy::{
    asset::{ReflectAsset, UntypedAssetId},
    log,
    prelude::*,
    reflect::TypeRegistry,
    render::camera::Viewport,
    tasks::IoTaskPool,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiSet};
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_mod_picking::backends::egui::EguiPointer;
use bevy_mod_picking::prelude::*;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::any::TypeId;
use std::{fs::File, io::Write};
use strum::IntoEnumIterator;
use transform_gizmo_bevy::prelude::*;

use crate::GameCamera;

use super::{
    garbage::{spawn_some_garbage, GarbageAssets, GarbageBundle, GarbageItem},
    map::{MapAssets, MapElementBundle},
};

pub struct DebugEditorPlugin;

impl Plugin for DebugEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_framepace::FramepacePlugin) // reduces input lag
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(bevy_egui::EguiPlugin)
            .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
            .add_plugins(TransformGizmoPlugin)
            .insert_resource(UiState::new())
            .add_systems(
                PostUpdate,
                (
                    show_ui_system
                        .before(EguiSet::ProcessOutput)
                        .before(bevy::transform::TransformSystem::TransformPropagate),
                    set_camera_viewport,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (set_gizmo_mode, auto_add_raycast_target, handle_pick_events),
            )
            .register_type::<Option<Handle<Image>>>()
            .register_type::<AlphaMode>();
    }
}

fn auto_add_raycast_target(
    mut commands: Commands,
    query: Query<Entity, (Without<Pickable>, With<Handle<Mesh>>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(PickableBundle::default());
    }
}

fn handle_pick_events(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut egui: EguiContexts,
    egui_entity: Query<&EguiPointer>,
    mut pointer_evr: EventReader<Pointer<Click>>,
    mut previous_entity: Local<Option<Entity>>,
) {
    let egui_context = egui.ctx_mut();

    for click in pointer_evr.read() {
        let entity = click.target();
        if egui_entity.get(entity).is_ok() {
            continue;
        };

        let modifiers = egui_context.input(|i| i.clone()).modifiers;
        let add = modifiers.ctrl || modifiers.shift;

        if let Some(prev) = *previous_entity {
            commands.entity(prev).remove::<GizmoTarget>();
        }
        commands.entity(entity).insert(GizmoTarget::default());
        *previous_entity = Some(entity);

        ui_state.selected_entities.select_maybe_add(entity, add);
    }
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<GameCamera>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    if !viewport_pos.is_finite()
        || !viewport_size.is_finite()
        || viewport_pos.x + viewport_size.x > window.resolution.physical_width() as f32
        || viewport_pos.y + viewport_size.y > window.resolution.physical_height() as f32
    {
        return;
    }

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

fn set_gizmo_mode(input: Res<ButtonInput<KeyCode>>, mut gizmo_options: ResMut<GizmoOptions>) {
    for (key, modes) in [
        (KeyCode::KeyR, GizmoMode::all_rotate()),
        (KeyCode::KeyT, GizmoMode::all_translate()),
        (KeyCode::KeyS, GizmoMode::all_scale()),
    ] {
        if input.just_pressed(key) {
            gizmo_options.gizmo_modes = modes;
        }
    }
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
struct UiState {
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] =
            tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy, EguiWindow::Commands]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_close_buttons(false)
            .show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Commands,
    Resources,
    Assets,
    Inspector,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Commands => commands_ui(ui, self.world),
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn commands_ui(ui: &mut egui::Ui, world: &mut World) {
    if ui.button("Spawn Map Cube").clicked() {
        let assets = world.resource::<MapAssets>();
        world.spawn(MapElementBundle::new_cube(assets));
    }
    egui::ComboBox::from_label("Spawn Garbage Item")
        .selected_text("Spawn Garbage")
        .show_ui(ui, |ui| {
            for item in GarbageItem::iter() {
                if ui.button(format!("{item:?}")).clicked() {
                    let assets = world.resource::<GarbageAssets>();
                    world.spawn(GarbageBundle::new(item, assets));
                }
            }
        });
    if ui.button("Spawn 50 garbage items").clicked() {
        let shape = Cuboid::new(50.0, 5.0, 50.0);
        spawn_some_garbage(50, Vec3::Y * 5.0, shape)(world);
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

pub fn save_scene(name: String) -> impl FnOnce(&mut bevy::prelude::World) {
    move |world: &mut World| {
        // With our sample world ready to go, we can now create our scene using DynamicScene or DynamicSceneBuilder.
        // For simplicity, we will create our scene using DynamicScene:
        let scene = DynamicScene::from_world(world);

        // Scenes can be serialized like this:
        let serialized_scene = {
            let type_registry = world.resource::<AppTypeRegistry>();
            let type_registry = type_registry.read();
            scene.serialize(&type_registry).unwrap()
        };

        // Showing the scene in the console
        log::info!("{}", serialized_scene);

        // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
        // as they are blocking
        // This can't work in WASM as there is no filesystem access
        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                // Write the scene RON data to file
                File::create(format!("assets/{name}"))
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}
