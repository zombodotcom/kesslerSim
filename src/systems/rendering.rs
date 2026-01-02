use crate::components::*;
use crate::systems::materials::MaterialsCache;
use crate::systems::render_mode::RenderMode;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::primitives::Sphere;
use bevy::prelude::*;

/// Marker component to track objects that have been rendered
#[derive(Component)]
pub struct RenderedObject;

/// System for handling mouse camera controls
pub fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Mouse rotation
    if mouse_buttons.pressed(MouseButton::Left) {
        for event in mouse_motion_events.read() {
            let delta = event.delta;

            // Horizontal rotation (around Y axis)
            camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-delta.x * 0.005));

            // Vertical rotation (around local X axis)
            let right = camera_transform.rotation * Vec3::X;
            camera_transform
                .rotate_around(Vec3::ZERO, Quat::from_axis_angle(right, -delta.y * 0.005));
        }
    }

    // Mouse zoom
    for event in mouse_wheel_events.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y * 0.5,
            MouseScrollUnit::Pixel => event.y * 0.01,
        };

        // Move camera towards/away from center
        let direction = camera_transform.translation.normalize();
        let new_distance =
            (camera_transform.translation.length() - scroll_amount).clamp(8.0, 100.0); // Min/max zoom distances

        camera_transform.translation = direction * new_distance;
    }
}

/// System to render satellites as small spheres
pub fn satellite_rendering_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials_cache: Res<MaterialsCache>,
    render_mode: Res<RenderMode>,
    satellites_without_mesh: Query<
        (Entity, &OrbitalState, &Satellite),
        (With<RenderAsSatellite>, Without<RenderedObject>),
    >,
    mut satellite_counter: Local<usize>,
) {
    let total_satellites = satellites_without_mesh.iter().count();
    let max_render = (total_satellites as f32 * render_mode.render_fraction) as usize;

    let mut count = 0;
    for (entity, orbital_state, _satellite) in satellites_without_mesh.iter() {
        if count >= max_render {
            break;
        }

        let scaled_position = orbital_state.position / 1000.0;
        let mesh = meshes.add(Sphere::new(0.05).mesh().ico(5).unwrap());

        commands
            .entity(entity)
            .insert(Mesh3d(mesh))
            .insert(MeshMaterial3d(materials_cache.satellite_material.clone()))
            .insert(Transform::from_translation(scaled_position))
            .insert(RenderedObject);

        count += 1;
    }
    *satellite_counter = total_satellites;
}

/// System to render enhanced debris with visual effects
pub fn debris_rendering_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials_cache: Res<MaterialsCache>,
    render_mode: Res<RenderMode>,
    debris_query: Query<
        (Entity, &OrbitalState, &EnhancedDebris),
        (With<RenderAsDebris>, Without<RenderedObject>),
    >,
    mut debris_counter: Local<usize>,
) {
    let total_debris = debris_query.iter().count();
    let max_render = (total_debris as f32 * render_mode.render_fraction) as usize;

    let mut count = 0;
    for (entity, orbital_state, enhanced_debris) in debris_query.iter() {
        if count >= max_render {
            break;
        }

        let base_size = 0.3;
        let scaled_size = base_size * enhanced_debris.size_multiplier;
        let mesh = meshes.add(Sphere::new(scaled_size).mesh().ico(4).unwrap());

        commands
            .entity(entity)
            .insert(Mesh3d(mesh))
            .insert(MeshMaterial3d(materials_cache.debris_material.clone()))
            .insert(Transform::from_translation(orbital_state.position / 1000.0))
            .insert(RenderedObject);

        count += 1;
    }
    *debris_counter = total_debris;
}

/// System to render collision flash effects
pub fn collision_flash_rendering_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials_cache: Res<MaterialsCache>,
    flash_query: Query<(Entity, &CollisionFlash, &Transform), Without<RenderedObject>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();

    for (entity, flash, transform) in flash_query.iter() {
        if !flash.is_active(current_time) {
            continue;
        }

        let intensity = flash.current_intensity(current_time);
        if intensity <= 0.0 {
            continue;
        }

        let flash_size = 0.5 + (current_time - flash.start_time) as f32 * 2.0;
        let mesh = meshes.add(Sphere::new(flash_size).mesh().ico(5).unwrap());

        commands
            .entity(entity)
            .insert(Mesh3d(mesh))
            .insert(MeshMaterial3d(materials_cache.flash_material.clone()))
            .insert(*transform)
            .insert(RenderedObject);
    }
}

/// System to clean up expired collision flash effects
pub fn cleanup_expired_flash_system(
    mut commands: Commands,
    flash_query: Query<(Entity, &CollisionFlash), With<RenderedObject>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();

    for (entity, flash) in flash_query.iter() {
        if !flash.is_active(current_time) {
            // Flash is expired, remove it safely
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                debug!(
                    "cleanup_expired_flash_system: Despawning expired flash entity {:?}",
                    entity
                );
                entity_commands.despawn();
            } else {
                debug!(
                    "cleanup_expired_flash_system: Entity {:?} already despawned",
                    entity
                );
            }
        }
    }
}

/// System to update debris visual properties over time
pub fn update_debris_effects_system(mut debris_query: Query<&mut EnhancedDebris>, time: Res<Time>) {
    let current_time = time.elapsed_secs_f64();

    for mut enhanced_debris in debris_query.iter_mut() {
        enhanced_debris.update_age(current_time);
    }
}

/// System to update positions of rendered objects
pub fn update_positions_system(
    mut query: Query<
        (&mut Transform, &OrbitalState),
        (With<RenderedObject>, Changed<OrbitalState>),
    >,
) {
    for (mut transform, orbital_state) in query.iter_mut() {
        // Scale down position to match rendering scale (km to render units)
        transform.translation = orbital_state.position / 1000.0;
    }
}
