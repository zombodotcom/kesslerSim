use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use bevy::prelude::*;

/// Component to mark selected satellites
#[derive(Component)]
pub struct SelectedSatellite {
    pub selection_time: f64,
}

/// Resource to track currently selected satellite
#[derive(Resource, Default)]
pub struct SatelliteSelection {
    pub selected_entity: Option<Entity>,
    pub selected_name: Option<String>,
}

/// System to handle mouse clicks for satellite selection
pub fn satellite_selection_system(
    mut commands: Commands,
    mut selection: ResMut<SatelliteSelection>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut satellite_query: Query<
        (Entity, &OrbitalState, &Satellite, &Transform),
        (With<RenderAsSatellite>, Without<SelectedSatellite>),
    >,
    sim_time: Res<SimulationTime>,
) {
    // Only handle left mouse button clicks
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get mouse position
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen coordinates to world ray
        // This is a simplified version - full implementation would use proper ray casting
        let screen_size = Vec2::new(window.width(), window.height());
        let mut ndc = (cursor_pos / screen_size) * 2.0 - Vec2::ONE;
        ndc.y = -ndc.y; // Flip Y axis

        // Find closest satellite to click position
        let mut closest_entity: Option<Entity> = None;
        let mut closest_distance = f32::MAX;

        for (entity, _orbital_state, _satellite, transform) in satellite_query.iter() {
            // Simple distance check (in screen space approximation)
            let screen_pos = transform.translation;
            let distance = (screen_pos - Vec3::new(ndc.x * 10.0, ndc.y * 10.0, 0.0)).length();

            if distance < closest_distance && distance < 1.0 {
                closest_distance = distance;
                closest_entity = Some(entity);
            }
        }

        // Deselect previous selection
        if let Some(prev_entity) = selection.selected_entity {
            if let Ok(mut entity_commands) = commands.get_entity(prev_entity) {
                entity_commands.remove::<SelectedSatellite>();
            }
        }

        // Select new satellite
        if let Some(entity) = closest_entity {
            if let Ok((_, _, satellite, _)) = satellite_query.get(entity) {
                selection.selected_entity = Some(entity);
                selection.selected_name = Some(satellite.name.clone());

                commands.entity(entity).insert(SelectedSatellite {
                    selection_time: sim_time.current,
                });

                info!("Selected satellite: {}", satellite.name);
            }
        } else {
            // Deselect if clicking on empty space
            selection.selected_entity = None;
            selection.selected_name = None;
        }
    }
}

/// System to display satellite information (would be used with HTML overlay)
pub fn satellite_info_display_system(
    selection: Res<SatelliteSelection>,
    satellite_query: Query<(&OrbitalState, &Satellite, &TleData), With<SelectedSatellite>>,
    constants: Res<Constants>,
    sim_time: Res<SimulationTime>,
) {
    if selection.selected_entity.is_none() {
        return;
    }

    if let Ok((orbital_state, satellite, tle_data)) = satellite_query.get_single() {
        let altitude = orbital_state.altitude() - constants.earth_radius;
        let speed = orbital_state.speed();
        let energy = orbital_state.total_energy(constants.gravitational_parameter);

        // Log satellite info (in a real implementation, this would update HTML elements)
        debug!(
            "=== SATELLITE INFO ===\n\
            Name: {}\n\
            NORAD ID: {}\n\
            Altitude: {:.2} km\n\
            Speed: {:.2} km/s\n\
            Mass: {:.2} kg\n\
            Total Energy: {:.2e} J\n\
            Simulation Time: {:.2} s",
            satellite.name,
            tle_data.norad_id,
            altitude,
            speed,
            orbital_state.mass,
            energy,
            sim_time.current
        );
    }
}
