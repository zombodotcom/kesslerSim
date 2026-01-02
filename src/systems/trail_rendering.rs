use bevy::prelude::*;
use crate::components::*;
use crate::components::trails::*;

/// System to update trail positions
pub fn update_trail_system(
    mut trail_query: Query<(&mut Trail, &OrbitalState)>,
    trail_config: Res<TrailConfig>,
) {
    if !trail_config.enabled {
        return;
    }

    for (mut trail, orbital_state) in trail_query.iter_mut() {
        // Convert position from km to render units
        let scaled_position = orbital_state.position / 1000.0;
        
        // Add position to trail
        trail.add_position(scaled_position);
        
        // Update altitude band if needed
        let altitude_km = orbital_state.altitude();
        trail.update_altitude_band(altitude_km);
    }
}

/// System to render trails using Bevy's line rendering
pub fn render_trails_system(
    mut commands: Commands,
    trail_query: Query<(Entity, &Trail), (With<OrbitalState>, Changed<Trail>)>,
    trail_config: Res<TrailConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_trails: Query<Entity, With<TrailMesh>>,
) {
    if !trail_config.enabled {
        // Remove existing trail meshes if trails are disabled
        for entity in existing_trails.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Remove old trail meshes
    for entity in existing_trails.iter() {
        commands.entity(entity).despawn();
    }

    // Create new trail meshes
    for (entity, trail) in trail_query.iter() {
        let points_with_alpha = trail.get_points_with_alpha();
        
        if points_with_alpha.len() < 2 {
            continue; // Need at least 2 points for a line
        }

        // Create line mesh from trail points
        let mut trail_mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineStrip,
            bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );

        // Extract positions
        let positions: Vec<[f32; 3]> = points_with_alpha.iter().map(|(pos, _)| [pos.x, pos.y, pos.z]).collect();
        
        // Create colors with alpha fade
        let colors: Vec<[f32; 4]> = points_with_alpha
            .iter()
            .map(|(_, alpha)| {
                {
                    let srgba = trail.color.to_srgba();
                    [srgba.red * *alpha, srgba.green * *alpha, srgba.blue * *alpha, *alpha]
                }
            })
            .collect();

        trail_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        trail_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

        // Create material for trail (unlit, emissive)
        let trail_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: bevy::render::alpha::AlphaMode::Blend,
            ..default()
        });

        // Spawn trail mesh entity
        commands.spawn((
            Mesh3d(meshes.add(trail_mesh)),
            MeshMaterial3d(trail_material),
            Transform::default(),
            TrailMesh { parent_entity: entity },
        ));
    }
}

/// Marker component for trail mesh entities
#[derive(Component)]
pub struct TrailMesh {
    pub parent_entity: Entity,
}

/// System to toggle trails on/off (key: T)
pub fn toggle_trails_system(
    mut trail_config: ResMut<TrailConfig>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        trail_config.enabled = !trail_config.enabled;
        info!("Trails {}", if trail_config.enabled { "enabled" } else { "disabled" });
    }
}

/// System to adjust trail length (keys: [ and ])
pub fn adjust_trail_length_system(
    mut trail_query: Query<&mut Trail>,
    trail_config: ResMut<TrailConfig>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let length_change = if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        -100
    } else if keyboard_input.just_pressed(KeyCode::BracketRight) {
        100
    } else {
        return;
    };

    let new_length = (trail_config.default_length as i32 + length_change).max(100).min(2000) as usize;
    
    for mut trail in trail_query.iter_mut() {
        trail.max_length = new_length;
        // Trim positions if needed
        let current_len = trail.positions.len();
        if current_len > new_length {
            let remove_count = current_len - new_length;
            trail.positions.drain(0..remove_count);
        }
    }
    
    info!("Trail length set to {}", new_length);
}

