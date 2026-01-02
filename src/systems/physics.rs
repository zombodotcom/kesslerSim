use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::integrators::*;

/// Main physics system implementing 2-body orbital mechanics
pub fn physics_system(
    mut orbital_query: Query<&mut OrbitalState>,
    mut sim_time: ResMut<SimulationTime>,
    time: Res<Time>,
    integrator_config: Option<Res<IntegratorConfig>>,
    constants: Res<Constants>,
) {
    // Update simulation time
    sim_time.advance(time.delta_secs());

    // Don't run physics if paused
    if sim_time.paused {
        return;
    }

    let dt = sim_time.timestep;
    let gm = constants.gravitational_parameter;
    let use_rk4 = integrator_config.map(|c| c.use_rk4).unwrap_or(false);

    for mut orbital_state in orbital_query.iter_mut() {
        let (new_position, new_velocity) = if use_rk4 {
            RK4Integrator::integrate(
                orbital_state.position,
                orbital_state.velocity,
                dt,
                gm,
            )
        } else {
            EulerIntegrator::integrate(
                orbital_state.position,
                orbital_state.velocity,
                dt,
                gm,
            )
        };

        orbital_state.position = new_position;
        orbital_state.velocity = new_velocity;
    }
}

/// System to handle simulation time controls
pub fn time_control_system(
    mut sim_time: ResMut<SimulationTime>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if sim_time.paused {
            sim_time.resume();
        } else {
            sim_time.pause();
        }
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        sim_time.set_speed(1.0); // Real time
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        sim_time.set_speed(60.0); // 1 minute per second
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        sim_time.set_speed(3600.0); // 1 hour per second
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        sim_time.set_speed(86400.0); // 1 day per second
    }
}

/// Debug system to print orbital information summary
pub fn debug_orbital_system(
    orbital_query: Query<(&OrbitalState, &Satellite)>,
    constants: Res<Constants>,
    sim_time: Res<SimulationTime>,
    mut last_print: Local<f64>,
) {
    // Print debug info every 30 seconds (reduced frequency)
    if sim_time.current - *last_print > 30.0 {
        *last_print = sim_time.current;
        
        let total_satellites = orbital_query.iter().count();
        if total_satellites > 0 {
            debug!("Orbital System: {} active satellites in simulation", total_satellites);
            
            // Only show detailed info for first few satellites in debug builds
            #[cfg(debug_assertions)]
            {
                let mut count = 0;
                for (orbital_state, satellite) in orbital_query.iter() {
                    if count >= 3 { // Limit to first 3 satellites
                        break;
                    }
                    let altitude = orbital_state.altitude() - constants.earth_radius;
                    let speed = orbital_state.speed();
                    let energy = orbital_state.total_energy(constants.gravitational_parameter);
                    
                    trace!(
                        "{}: Alt={:.1}km, Speed={:.2}km/s, Energy={:.2e}J",
                        satellite.name, altitude, speed, energy
                    );
                    count += 1;
                }
                if total_satellites > 3 {
                    trace!("... and {} more satellites", total_satellites - 3);
                }
            }
        }
    }
}