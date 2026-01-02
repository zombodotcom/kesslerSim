// Shared test utilities and fixtures for Kessler Syndrome Simulator tests

use bevy::prelude::*;
use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use kessler_simulator::systems::collision::{OctreeNode, SpatialOctree};
use kessler_simulator::utils::*;

/// Create a test orbital state at a given altitude (km above Earth surface)
/// Returns a circular orbit with proper velocity for that altitude
pub fn create_test_orbital_state(altitude_km: f64) -> OrbitalState {
    let constants = Constants::default();
    let radius_km = constants.earth_radius + altitude_km;
    let orbital_velocity = constants.circular_velocity(altitude_km);

    // Position at positive x-axis, velocity in positive y direction (circular orbit)
    OrbitalState::new(
        Vec3::new(radius_km as f32, 0.0, 0.0),
        Vec3::new(0.0, orbital_velocity as f32, 0.0),
        1000.0, // Default mass of 1000 kg
    )
}

/// Create a test TLE record with specified parameters
pub fn create_test_tle(name: &str, norad_id: u32) -> TleRecord {
    TleRecord {
        name: name.to_string(),
        norad_id,
        classification: 'U',
        international_designator: "00000A".to_string(),
        epoch_year: 23,
        epoch_day: 1.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6, // ISS-like inclination
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5, // ~90 minute orbit
        revolution_number: 1,
        line1: format!(
            "1 {:05}U 23001A   23001.00000000  .00000000  00000-0  00000-0 0  9999",
            norad_id
        ),
        line2: format!(
            "2 {:05}  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01",
            norad_id
        ),
    }
}

/// Assert that energy is conserved within a tolerance
/// This validates that the physics system maintains energy conservation
pub fn assert_energy_conserved(before: f64, after: f64, tolerance: f64) {
    let difference = (before - after).abs();
    let relative_error = if before.abs() > 0.0 {
        difference / before.abs()
    } else {
        difference
    };

    assert!(
        relative_error < tolerance,
        "Energy not conserved: before={:.6e}, after={:.6e}, difference={:.6e}, relative_error={:.6e}",
        before, after, difference, relative_error
    );
}

/// Assert that an orbital state is physically valid
/// Checks that position and velocity are reasonable and energy is negative (bound orbit)
pub fn assert_orbital_valid(state: &OrbitalState, constants: &Constants) {
    // Position should be above Earth surface
    assert!(
        state.altitude() > constants.earth_radius,
        "Orbital state position is below Earth surface: altitude={:.2} km",
        state.altitude() - constants.earth_radius
    );

    // Velocity should be positive
    assert!(
        state.speed() > 0.0,
        "Orbital state has zero or negative velocity: speed={:.2} km/s",
        state.speed()
    );

    // For a bound orbit, total energy should be negative
    let total_energy = state.total_energy(constants.gravitational_parameter);
    assert!(
        total_energy < 0.0,
        "Orbital state has positive total energy (unbound orbit): energy={:.6e} J",
        total_energy
    );
}

/// Run physics calculations manually for testing
/// This simulates a single physics step without Bevy ECS
pub fn run_physics_step(state: &mut OrbitalState, constants: &Constants, dt: f64) {
    use bevy::prelude::*;

    // Work with f64 precision for physics calculations
    let pos_x = state.position.x as f64;
    let pos_y = state.position.y as f64;
    let pos_z = state.position.z as f64;

    let vel_x = state.velocity.x as f64;
    let vel_y = state.velocity.y as f64;
    let vel_z = state.velocity.z as f64;

    // Calculate gravitational acceleration: a = -GM * r / |r|³
    let r_magnitude_km = (pos_x * pos_x + pos_y * pos_y + pos_z * pos_z).sqrt();
    let r_magnitude_m = r_magnitude_km * 1000.0; // Convert km to m

    if r_magnitude_m > 0.0 {
        let gm = constants.gravitational_parameter;
        let acc_magnitude = -gm / (r_magnitude_m * r_magnitude_m);

        // Unit vector components
        let r_unit_x = pos_x / r_magnitude_km;
        let r_unit_y = pos_y / r_magnitude_km;
        let r_unit_z = pos_z / r_magnitude_km;

        // Acceleration in km/s²
        let acc_km_s2 = acc_magnitude / 1000.0;
        let acc_x = r_unit_x * acc_km_s2;
        let acc_y = r_unit_y * acc_km_s2;
        let acc_z = r_unit_z * acc_km_s2;

        // Simple Euler integration
        let new_vel_x = vel_x + acc_x * dt;
        let new_vel_y = vel_y + acc_y * dt;
        let new_vel_z = vel_z + acc_z * dt;

        let new_pos_x = pos_x + new_vel_x * dt;
        let new_pos_y = pos_y + new_vel_y * dt;
        let new_pos_z = pos_z + new_vel_z * dt;

        // Update orbital state
        state.velocity = Vec3::new(new_vel_x as f32, new_vel_y as f32, new_vel_z as f32);
        state.position = Vec3::new(new_pos_x as f32, new_pos_y as f32, new_pos_z as f32);
    }
}

/// Create a known ISS TLE record for validation tests
pub fn create_iss_tle() -> TleRecord {
    TleRecord {
        name: "ISS (ZARYA)".to_string(),
        norad_id: 25544,
        classification: 'U',
        international_designator: "98067A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0001,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6442,
        right_ascension: 123.4567,
        eccentricity: 0.0001234,
        argument_of_perigee: 45.6789,
        mean_anomaly: 123.4567,
        mean_motion: 15.49000000,
        revolution_number: 12345,
        line1: "1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999".to_string(),
        line2: "2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345".to_string(),
    }
}

/// Create a known Hubble TLE record for validation tests
pub fn create_hubble_tle() -> TleRecord {
    TleRecord {
        name: "HUBBLE SPACE TELESCOPE".to_string(),
        norad_id: 20580,
        classification: 'U',
        international_designator: "90037A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0001,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 28.4692,
        right_ascension: 234.5678,
        eccentricity: 0.0002345,
        argument_of_perigee: 56.7890,
        mean_anomaly: 234.5678,
        mean_motion: 15.09000000,
        revolution_number: 67890,
        line1: "1 20580U 90037A   23200.12345678  .00001234  00000+0  12345-4 0  9999".to_string(),
        line2: "2 20580  28.4692 234.5678 0002345  56.7890 234.5678 15.09000000 67890".to_string(),
    }
}

/// Calculate expected orbital period from semi-major axis
pub fn calculate_orbital_period(semi_major_axis_km: f64) -> f64 {
    let constants = Constants::default();
    let mu = constants.gravitational_parameter / 1e9; // Convert to km³/s²
    let a = semi_major_axis_km;
    2.0 * std::f64::consts::PI * (a * a * a / mu).sqrt()
}

/// Calculate semi-major axis from orbital period
pub fn calculate_semi_major_axis(period_seconds: f64) -> f64 {
    let constants = Constants::default();
    let mu = constants.gravitational_parameter / 1e9; // Convert to km³/s²
    (mu * period_seconds * period_seconds / (4.0 * std::f64::consts::PI * std::f64::consts::PI))
        .powf(1.0 / 3.0)
}
