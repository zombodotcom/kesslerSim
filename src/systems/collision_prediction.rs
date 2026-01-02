use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Component to mark objects with collision warnings
#[derive(Component)]
pub struct CollisionWarning {
    pub risk_level: RiskLevel,
    pub time_to_collision: f64,
    pub other_entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,    // >1000 seconds
    Medium, // 100-1000 seconds
    High,   // 10-100 seconds
    Critical, // <10 seconds
}

/// System to predict collisions
/// TEMPORARILY DISABLED - causes crashes and performance issues
/// The entity lifecycle issue is complex - entities can be despawned between query and command application
#[allow(dead_code, unused_variables)]
pub fn predict_collisions_system(
    mut _commands: Commands,
    _orbital_query: Query<(Entity, &OrbitalState), (With<PhysicsObject>, Changed<OrbitalState>)>,
    mut warning_query: Query<(Entity, &mut CollisionWarning)>,
    _constants: Res<Constants>,
    _sim_time: Res<SimulationTime>,
) {
    // Clear all existing warnings - only operate on entities that still exist
    let warning_entities: Vec<Entity> = warning_query.iter().map(|(e, _)| e).collect();
    for entity in warning_entities {
        // This will only succeed if entity still exists
        if let Ok(mut entity_commands) = _commands.get_entity(entity) {
            entity_commands.remove::<CollisionWarning>();
        }
    }
    
    // System disabled - collision prediction causes crashes and is too expensive
    // TODO: Re-implement with proper entity lifecycle management
}

/// Predict closest approach between two objects
fn predict_closest_approach(
    pos1: Vec3,
    vel1: Vec3,
    pos2: Vec3,
    vel2: Vec3,
    _gm: f64,
) -> Option<(f64, f64)> {
    // Simplified prediction: linear extrapolation
    // In a full implementation, this would use orbital mechanics
    
    let rel_pos = pos2 - pos1;
    let rel_vel = vel2 - vel1;
    
    // Time to closest approach (minimize relative distance)
    let rel_vel_sq = rel_vel.length_squared();
    if rel_vel_sq < 1e-10 {
        return None; // Objects are moving in parallel, no closest approach
    }
    
    let t = -rel_pos.dot(rel_vel) / rel_vel_sq;
    
    if t > 0.0 && t < 1000.0 {
        let future_rel_pos = rel_pos + rel_vel * t as f32;
        let distance = future_rel_pos.length() as f64;
        Some((t as f64, distance))
    } else {
        None
    }
}

/// System to visualize collision warnings
pub fn visualize_collision_warnings_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    warning_query: Query<(Entity, &CollisionWarning, &OrbitalState), (With<CollisionWarning>, Without<CollisionWarningVisual>)>,
    existing_visuals: Query<Entity, With<CollisionWarningVisual>>,
) {
    // Remove old visuals (only if they still exist)
    let visual_entities: Vec<Entity> = existing_visuals.iter().collect();
    for entity in visual_entities {
        if commands.get_entity(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }

    // Create new visuals
    for (entity, warning, orbital_state) in warning_query.iter() {
        let color = match warning.risk_level {
            RiskLevel::Low => Color::srgb(0.5, 0.5, 0.0),
            RiskLevel::Medium => Color::srgb(1.0, 0.8, 0.0),
            RiskLevel::High => Color::srgb(1.0, 0.4, 0.0),
            RiskLevel::Critical => Color::srgb(1.0, 0.0, 0.0),
        };

        let warning_mesh = meshes.add(Sphere::new(0.2).mesh().ico(5).unwrap());
        let warning_material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
            unlit: true,
            alpha_mode: bevy::render::alpha::AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Mesh3d(warning_mesh),
            MeshMaterial3d(warning_material),
            Transform::from_translation(orbital_state.position / 1000.0),
            CollisionWarningVisual { parent: entity },
        ));
    }
}

/// Marker component for collision warning visuals
#[derive(Component)]
pub struct CollisionWarningVisual {
    pub parent: Entity,
}

