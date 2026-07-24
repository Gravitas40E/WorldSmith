use std::collections::HashSet;

use crate::renderers::bevy::BodyMarker;
use bevy::input::mouse::{MouseButton, MouseWheel};
use bevy::prelude::*;

const DEFAULT_CAMERA_DISTANCE: f32 = 111.803;
const DEFAULT_CAMERA_ELEVATION: f32 = 0.4636;
const DEFAULT_SELECT_DISTANCE: f32 = 20.0;
const DEFAULT_LABEL_OFFSET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
const LABEL_FONT_SIZE: f32 = 12.0;
const FPS_FONT_SIZE: f32 = 16.0;
const FRAME_LERP_FACTOR: f32 = 0.1;
const ELEVATION_CLAMP: f32 = 1.5;

#[derive(Resource, Default, Debug)]
pub struct ViewerState {
    pub paused: bool,
    pub show_labels: bool,
    pub show_orbits: bool,
    pub selected_body_id: Option<String>,
    pub frame_target: Option<Vec3>,
    pub target_distance: Option<f32>,
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub pan_speed: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

#[derive(Component)]
pub struct BodyLabel {
    pub body_id: String,
}

#[derive(Component)]
pub struct FpsLabel;

#[derive(Component)]
pub struct OrbitPathMarker;

/// Interactive viewer plugin.
///
/// Adds orbit camera, pan/zoom, labeling, pause, FPS overlay, and
/// camera framing on top of the existing visualization renderer.
///
/// Assumes [`BevyVisualizationPlugin`] has already been added. If the
/// default camera does not exist, orbit camera systems will skip
/// themselves; see [`WorldSmithVisualizationPlugin`] for an ordered
/// all-in-one alternative.
pub struct BevyViewerPlugin;

impl Plugin for BevyViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ViewerState::default());
        app.add_systems(Startup, setup_fps_overlay);
        app.add_systems(PreUpdate, handle_input);
        app.add_systems(
            Update,
            (
                update_orbit_camera,
                update_labels,
                frame_selected_body,
                reset_camera,
                update_fps,
            ),
        );
    }
}

fn setup_fps_overlay(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "FPS: --",
            TextStyle {
                font_size: FPS_FONT_SIZE,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        FpsLabel,
    ));
}

fn handle_input(
    mut viewer: ResMut<ViewerState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut orbit_cameras: Query<&mut OrbitCamera>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    bodies: Query<(&BodyMarker, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        viewer.show_labels = !viewer.show_labels;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        viewer.paused = !viewer.paused;
    }
    if keyboard.just_pressed(KeyCode::KeyO) {
        viewer.show_orbits = !viewer.show_orbits;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok(mut orbit) = orbit_cameras.get_single_mut() {
            *orbit = OrbitCamera {
                target: Vec3::ZERO,
                distance: DEFAULT_CAMERA_DISTANCE,
                azimuth: 0.0,
                elevation: DEFAULT_CAMERA_ELEVATION,
                ..default()
            };
        }
        viewer.selected_body_id = None;
        viewer.frame_target = None;
        viewer.target_distance = None;
    }

    if mouse.just_pressed(MouseButton::Left) {
        select_body(&mut viewer, &windows, &camera, &bodies, &mouse);
    }
}

fn select_body(
    viewer: &mut ResMut<ViewerState>,
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
    bodies: &Query<(&BodyMarker, &GlobalTransform)>,
    mouse: &Res<ButtonInput<MouseButton>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let Some(mouse_pos) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = match camera.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut closest = None;
    let mut min_dist = DEFAULT_SELECT_DISTANCE;
    let mut closest_pos = None;

    for (marker, global_transform) in bodies.iter() {
        let pos = global_transform.translation();
        if let Some(screen_pos) = camera.world_to_viewport(camera_transform, pos) {
            let dist = screen_pos.distance(mouse_pos);
            if dist < min_dist {
                min_dist = dist;
                closest = Some(marker.body_id.clone());
                closest_pos = Some(pos);
            }
        }
    }

    viewer.selected_body_id = closest;
    if let Some(pos) = closest_pos {
        viewer.frame_target = Some(pos);
        viewer.target_distance = Some(DEFAULT_SELECT_DISTANCE);
    }
}

fn update_orbit_camera(
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut scroll: EventReader<MouseWheel>,
    mut prev_cursor: Local<Option<Vec2>>,
) {
    let Ok((mut orbit, mut transform)) = query.get_single_mut() else {
        return;
    };

    for event in scroll.read() {
        orbit.distance += event.y * orbit.zoom_speed;
        orbit.distance = orbit.distance.clamp(orbit.min_distance, orbit.max_distance);
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let current = window.cursor_position();
    let delta = if let (Some(prev), Some(curr)) = (*prev_cursor, current) {
        curr - prev
    } else {
        Vec2::ZERO
    };
    *prev_cursor = current;

    if mouse.pressed(MouseButton::Right) {
        orbit.azimuth -= delta.x * orbit.rotate_speed;
        orbit.elevation = (orbit.elevation + delta.y * orbit.rotate_speed)
            .clamp(-ELEVATION_CLAMP, ELEVATION_CLAMP);
    }

    if mouse.pressed(MouseButton::Middle) {
        let forward = *transform.forward();
        let right = *transform.right();
        let pan = orbit.pan_speed;
        orbit.target += forward * -delta.y * pan;
        orbit.target += right * delta.x * pan;
    }

    let pos = spherical_to_cartesian(orbit.distance, orbit.azimuth, orbit.elevation);
    transform.translation = orbit.target + pos;
    transform.look_at(orbit.target, Vec3::Y);
}

fn spherical_to_cartesian(distance: f32, azimuth: f32, elevation: f32) -> Vec3 {
    Vec3::new(
        distance * elevation.cos() * azimuth.sin(),
        distance * elevation.sin(),
        distance * elevation.cos() * azimuth.cos(),
    )
}

fn update_labels(
    mut commands: Commands,
    viewer: Res<ViewerState>,
    q_bodies: Query<(Entity, &BodyMarker)>,
    q_labels: Query<(Entity, &BodyLabel)>,
) {
    if !viewer.show_labels {
        for (entity, _) in q_labels.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // Borrow string slices into the set instead of cloning every
    // body ID. This avoids N heap allocations per frame when labels
    // are enabled.
    let mut labeled_bodies = HashSet::with_capacity(q_labels.iter().len());
    for (_, label) in q_labels.iter() {
        labeled_bodies.insert(label.body_id.as_str());
    }

    for (entity, marker) in q_bodies.iter() {
        if labeled_bodies.contains(marker.body_id.as_str()) {
            continue;
        }
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        marker.body_id.clone(),
                        TextStyle {
                            font_size: LABEL_FONT_SIZE,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(DEFAULT_LABEL_OFFSET),
                    ..default()
                },
                BodyLabel {
                    body_id: marker.body_id.clone(),
                },
            ));
        });
    }
}

fn frame_selected_body(
    viewer: Res<ViewerState>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Some(target) = viewer.frame_target else {
        return;
    };
    let Some(distance) = viewer.target_distance else {
        return;
    };

    let Ok((mut orbit, mut transform)) = query.get_single_mut() else {
        return;
    };

    orbit.target = orbit.target.lerp(target, FRAME_LERP_FACTOR);
    orbit.distance = orbit.distance.lerp(distance, FRAME_LERP_FACTOR);

    let pos = spherical_to_cartesian(orbit.distance, orbit.azimuth, orbit.elevation);
    transform.translation = orbit.target + pos;
    transform.look_at(orbit.target, Vec3::Y);
}

fn reset_camera(
    mut viewer: ResMut<ViewerState>,
    mut orbit_cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    if viewer.frame_target.is_none() {
        return;
    }

    let Ok((mut orbit, mut transform)) = orbit_cameras.get_single_mut() else {
        return;
    };

    *orbit = OrbitCamera {
        target: Vec3::ZERO,
        distance: DEFAULT_CAMERA_DISTANCE,
        azimuth: 0.0,
        elevation: DEFAULT_CAMERA_ELEVATION,
        ..default()
    };

    let pos = spherical_to_cartesian(orbit.distance, orbit.azimuth, orbit.elevation);
    transform.translation = orbit.target + pos;
    transform.look_at(orbit.target, Vec3::Y);
    viewer.frame_target = None;
    viewer.target_distance = None;
}

fn update_fps(
    time: Res<Time>,
    mut frames: Local<u32>,
    mut accumulator: Local<f32>,
    mut query: Query<&mut Text, With<FpsLabel>>,
) {
    *frames += 1;
    *accumulator += time.delta_seconds();

    if *accumulator >= 1.0 {
        let fps = *frames as f32 / *accumulator;
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].value = format!("FPS: {:.1}", fps);
        }
        *frames = 0;
        *accumulator = 0.0;
    }
}
