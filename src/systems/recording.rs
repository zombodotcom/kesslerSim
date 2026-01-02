use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::resources::recording::*;
use crate::systems::collision::CollisionPairs;
use std::time::{SystemTime, UNIX_EPOCH};

/// System to record simulation state
pub fn record_simulation_system(
    mut recorder: ResMut<SimulationRecorder>,
    sim_time: Res<SimulationTime>,
    orbital_query: Query<(Entity, &OrbitalState, Option<&Satellite>, Option<&Debris>)>,
    collision_pairs: Res<CollisionPairs>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Toggle recording with R key
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if recorder.is_recording {
            recorder.stop();
            // Auto-save when stopping
            if let Err(e) = recorder.save() {
                warn!("Failed to save recording: {}", e);
            }
        } else {
            // Generate filename with timestamp
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let file_path = format!("recordings/simulation_{}.json", timestamp);
            recorder.start(file_path);
        }
    }

    if !recorder.is_recording {
        return;
    }

    // Only record at specified intervals
    if sim_time.current - recorder.last_recorded_time < recorder.frame_interval {
        return;
    }

    recorder.last_recorded_time = sim_time.current;

    // Collect object snapshots
    let mut objects = Vec::new();
    let mut entity_id_map = std::collections::HashMap::new();
    let mut next_id = 0u64;

    for (entity, orbital_state, satellite, debris) in orbital_query.iter() {
        let id = *entity_id_map.entry(entity).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });

        let name = if let Some(sat) = satellite {
            sat.name.clone()
        } else if debris.is_some() {
            "Debris".to_string()
        } else {
            "Unknown".to_string()
        };

        objects.push(ObjectSnapshot {
            entity_id: id,
            position: [
                orbital_state.position.x as f64,
                orbital_state.position.y as f64,
                orbital_state.position.z as f64,
            ],
            velocity: [
                orbital_state.velocity.x as f64,
                orbital_state.velocity.y as f64,
                orbital_state.velocity.z as f64,
            ],
            mass: orbital_state.mass,
            is_satellite: satellite.is_some(),
            name,
        });
    }

    // Collect collision events (simplified - would need to track collision history)
    let collisions = Vec::new(); // TODO: Track collision events

    // Collect debris creation events (simplified)
    let debris_created = Vec::new(); // TODO: Track debris creation

    let frame = SimulationFrame {
        timestamp: sim_time.current,
        objects,
        collisions,
        debris_created,
    };

    recorder.add_frame(frame);
}

/// System to create recordings directory
pub fn setup_recording_directory() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        if let Err(e) = fs::create_dir_all("recordings") {
            warn!("Failed to create recordings directory: {}", e);
        }
    }
}

