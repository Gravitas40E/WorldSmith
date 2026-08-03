use bevy::prelude::*;
use bevy::render::mesh::{SphereKind, SphereMeshBuilder};

use std::collections::{HashMap, HashSet};

use crate::{
    renderer::SceneRenderer,
    renderers::viewer::{OrbitCamera, ViewerState},
    scene::{BodyCategory, ColorHint, RenderScene, SceneBody},
};

const MIN_BODY_RADIUS: f32 = 0.001;
const SPHERE_SECTORS: usize = 32;
const SPHERE_STACKS: usize = 18;

/// Marker component attached to each spawned body so the viewer can
/// identify it when updating labels, orbit paths, or selection state.
#[derive(Component)]
pub struct BodyMarker {
    pub body_id: String,
}

/// Bevy-backed scene renderer.
///
/// Implements the renderer-agnostic [`SceneRenderer`] trait by queuing
/// scene bodies in memory. Actual ECS entity creation happens inside the
/// Bevy schedule via [`flush_bodies`].
///
/// Keeping the renderer stateless from the Bevy app's perspective allows
/// it to be stored as a resource and flushed on a schedule.
#[derive(Resource, Default)]
pub struct BevySceneRenderer {
    /// Latest scene queued by `render()` for ECS synchronization.
    pub(in crate::renderers) scene: Option<RenderScene>,
    /// Stable mapping from `SceneBody.id` to spawned Bevy entity.
    pub(in crate::renderers) id_map: HashMap<String, Entity>,
    /// Per-body last-known state and generated asset handles to avoid
    /// unnecessary mesh/material regeneration.
    pub(in crate::renderers) body_states:
        HashMap<String, (SceneBody, Handle<Mesh>, Handle<StandardMaterial>)>,
}

impl SceneRenderer for BevySceneRenderer {
    fn render(&mut self, scene: &RenderScene) {
        self.scene = Some(scene.clone());
    }
}

/// Bevy plugin that registers the visualization systems.
///
/// Installs the [`BevySceneRenderer`] resource and spawns a default
/// camera plus point light. This plugin owns the ECS side of the
/// renderer-agnostic pipeline buffer. For interactive controls, add
/// [`BevyViewerPlugin`] after this plugin so it can query the camera
/// and renderer state.
///
/// Prefer [`WorldSmithVisualizationPlugin`] when you want both at once.
pub struct BevyVisualizationPlugin;

impl Plugin for BevyVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BevySceneRenderer::default());
        app.add_systems(Startup, setup_camera_and_light);
        app.add_systems(Update, flush_bodies);
    }
}

fn setup_camera_and_light(mut commands: Commands) {
    // Perspective camera positioned to see a typical simulation scale.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera {
            target: Vec3::ZERO,
            distance: 111.803,
            azimuth: 0.0,
            elevation: 0.4636,
            ..default()
        },
    ));

    // Single point light above origin.
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 60.0, 0.0),
        ..default()
    });
}

fn flush_bodies(
    mut renderer: ResMut<BevySceneRenderer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    viewer: Res<ViewerState>,
) {
    if viewer.paused {
        return;
    }

    let scene = match renderer.scene.take() {
        Some(scene) => scene,
        None => return,
    };

    // Borrow string slices for the membership set. Cloning every body ID
    // into a `HashSet<String>` forces N heap allocations per frame; a
    // borrowed `&str` set avoids that entirely.
    let current_ids: HashSet<&str> = scene.bodies.iter().map(|b| b.id.as_str()).collect();

    // Despawn entities that no longer appear in the current scene.
    renderer.id_map.retain(|id, entity| {
        if current_ids.contains(id.as_str()) {
            true
        } else {
            commands.entity(*entity).despawn_recursive();
            false
        }
    });

    // Prune cached mesh/material entries for removed bodies so Bevy can
    // GC handles that are no longer referenced.
    renderer
        .body_states
        .retain(|id, _| current_ids.contains(id.as_str()));

    for body in scene.bodies {
        let body_id = &body.id;
        let pos = Vec3::new(
            body.position_m[0] as f32,
            body.position_m[1] as f32,
            body.position_m[2] as f32,
        );
        let radius = body.radius_m.max(MIN_BODY_RADIUS.into()) as f32;
        let base_color = match (body.category, body.color_hint) {
            (_, ColorHint::Rgb { r, g, b }) => Color::rgb(r, g, b),
            (_, ColorHint::Temperature) => Color::rgb(1.0, 0.4, 0.0),
            (_, ColorHint::Classification) => match body.category {
                BodyCategory::Star => Color::rgb(1.0, 0.9, 0.3),
                BodyCategory::Planet => Color::rgb(0.3, 0.5, 1.0),
                BodyCategory::Moon => Color::rgb(0.7, 0.7, 0.7),
                BodyCategory::StellarSystem => Color::rgb(0.0, 1.0, 1.0),
            },
        };

        // Snapshot the previous cached position before any appearance
        // updates so we can compare position independently.
        let prev_position_m = renderer
            .body_states
            .get(body_id)
            .map(|(prev, _, _)| prev.position_m);

        let transform_changed = prev_position_m
            .map(|prev| prev != body.position_m)
            .unwrap_or(true);

        let appearance_changed = match renderer.body_states.get(body_id) {
            Some((prev, _, _)) => {
                prev.radius_m != body.radius_m
                    || prev.color_hint != body.color_hint
                    || prev.category != body.category
            }
            None => true,
        };

        // Only regenerate mesh/material when appearance actually changed.
        // This avoids `Assets<Mesh>` and `Assets<StandardMaterial>` churn
        // for bodies that only moved.
        if appearance_changed {
            let mesh = meshes.add(Mesh::from(SphereMeshBuilder::new(
                radius,
                SphereKind::Uv {
                    sectors: SPHERE_SECTORS,
                    stacks: SPHERE_STACKS,
                },
            )));
            let material = materials.add(Into::<StandardMaterial>::into(base_color));
            renderer.body_states.insert(
                body_id.clone(),
                (body.clone(), mesh.clone(), material.clone()),
            );
        }

        let mesh_handle;
        let material_handle;
        if let Some((_, mesh, material)) = renderer.body_states.get(body_id) {
            mesh_handle = mesh.clone();
            material_handle = material.clone();
        } else {
            // In debug builds we want to know if the cache invariant broke.
            // In release builds we recover gracefully instead of aborting.
            debug_assert!(
                false,
                "body_states cache invariant violated for body_id='{}'",
                body_id
            );
            let mesh = meshes.add(Mesh::from(SphereMeshBuilder::new(
                radius,
                SphereKind::Uv {
                    sectors: SPHERE_SECTORS,
                    stacks: SPHERE_STACKS,
                },
            )));
            let material = materials.add(Into::<StandardMaterial>::into(base_color));
            renderer.body_states.insert(
                body_id.clone(),
                (body.clone(), mesh.clone(), material.clone()),
            );
            mesh_handle = mesh;
            material_handle = material;
        }

        let is_new = !renderer.id_map.contains_key(body_id);

        if is_new {
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: mesh_handle,
                        material: material_handle,
                        transform: Transform::from_translation(pos),
                        ..default()
                    },
                    BodyMarker {
                        body_id: body_id.clone(),
                    },
                ))
                .id();
            renderer.id_map.insert(body_id.clone(), entity);
        } else if transform_changed || appearance_changed {
            // Rewrite components only when something changed. Skipping
            // unchanged bodies avoids component-change events and
            // archetype migrations, the biggest win for static scenes.
            let entity = *renderer.id_map.get(body_id).unwrap();
            commands.entity(entity).insert(PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform::from_translation(pos),
                ..default()
            });
        }
    }
}
