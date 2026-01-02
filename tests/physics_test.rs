// Unit tests for physics system functionality
// Tests gravitational force, orbital velocity, energy calculations, Euler integration, and energy conservation

use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use kessler_simulator::systems::*;
mod common;
use approx::assert_relative_eq;
use bevy::prelude::*;
use common::*;

#[test]
fn test_gravitational_force_calculation() {
    // Test that gravitational force calculation is correct
    let constants = Constants::default();

    // At ISS altitude (~400 km), gravitational acceleration should be approximately 8.7 m/s²
    let altitude_km = 400.0;
    let radius_km = constants.earth_radius + altitude_km;
    let acceleration = constants.gravity_acceleration(radius_km);

    // g = GM/r², at 400 km: g ≈ 8.7 m/s²
    let expected_acceleration = 8.7; // m/s²
    assert_relative_eq!(acceleration, expected_acceleration, epsilon = 0.1);
}

#[test]
fn test_circular_velocity() {
    // Test circular orbital velocity calculation
    let constants = Constants::default();

    // At ISS altitude (~400 km), circular velocity should be approximately 7.66 km/s
    let altitude_km = 400.0;
    let velocity = constants.circular_velocity(altitude_km);

    let expected_velocity = 7.66; // km/s
    assert_relative_eq!(velocity, expected_velocity, epsilon = 0.1);

    // At GEO altitude (35786 km), circular velocity should be approximately 3.07 km/s
    let geo_altitude = 35786.0;
    let geo_velocity = constants.circular_velocity(geo_altitude);

    let expected_geo_velocity = 3.07; // km/s
    assert_relative_eq!(geo_velocity, expected_geo_velocity, epsilon = 0.1);
}

#[test]
fn test_escape_velocity() {
    // Test escape velocity calculation
    let constants = Constants::default();

    // Escape velocity should be sqrt(2) times circular velocity
    let altitude_km = 400.0;
    let circular_vel = constants.circular_velocity(altitude_km);
    let escape_vel = constants.escape_velocity(constants.earth_radius + altitude_km);

    let expected_escape = circular_vel * 2.0_f64.sqrt();
    assert_relative_eq!(escape_vel, expected_escape, epsilon = 0.1);
}

#[test]
fn test_kinetic_energy() {
    // Test kinetic energy calculation
    let position = Vec3::new(6771.0, 0.0, 0.0); // 400 km altitude
    let velocity = Vec3::new(0.0, 7.66, 0.0); // 7.66 km/s
    let mass = 1000.0; // 1000 kg

    let orbital_state = OrbitalState::new(position, velocity, mass);
    let ke = orbital_state.kinetic_energy();

    // KE = 0.5 * m * v²
    // v = 7.66 km/s = 7660 m/s
    // KE = 0.5 * 1000 * 7660² = 29,337,800,000 J ≈ 2.93e10 J
    let expected_ke = 0.5 * mass * (7.66_f64 * 1000.0).powi(2);
    assert_relative_eq!(ke, expected_ke, epsilon = 1e8);
}

#[test]
fn test_potential_energy() {
    // Test potential energy calculation
    let constants = Constants::default();
    let position = Vec3::new(6771.0, 0.0, 0.0); // 400 km altitude
    let velocity = Vec3::new(0.0, 7.66, 0.0);
    let mass = 1000.0;

    let orbital_state = OrbitalState::new(position, velocity, mass);
    let pe = orbital_state.potential_energy(constants.gravitational_parameter);

    // PE = -GMm/r
    // Should be negative (bound orbit)
    assert!(
        pe < 0.0,
        "Potential energy should be negative for bound orbit"
    );

    // At higher altitude, potential energy should be less negative (closer to zero)
    let higher_position = Vec3::new(10000.0, 0.0, 0.0);
    let higher_state = OrbitalState::new(higher_position, velocity, mass);
    let higher_pe = higher_state.potential_energy(constants.gravitational_parameter);

    assert!(
        higher_pe > pe,
        "Higher altitude should have less negative potential energy"
    );
}

#[test]
fn test_total_energy() {
    // Test total energy calculation
    let constants = Constants::default();
    let position = Vec3::new(6771.0, 0.0, 0.0); // 400 km altitude
    let velocity = Vec3::new(0.0, 7.66, 0.0);
    let mass = 1000.0;

    let orbital_state = OrbitalState::new(position, velocity, mass);
    let total_energy = orbital_state.total_energy(constants.gravitational_parameter);

    // For a bound circular orbit, total energy should be negative
    assert!(
        total_energy < 0.0,
        "Total energy should be negative for bound orbit"
    );

    // Total energy = KE + PE
    let ke = orbital_state.kinetic_energy();
    let pe = orbital_state.potential_energy(constants.gravitational_parameter);
    assert_relative_eq!(total_energy, ke + pe, epsilon = 1e6);
}

#[test]
fn test_euler_integration() {
    // Test that Euler integration advances position and velocity correctly
    let constants = Constants::default();
    let mut state = create_test_orbital_state(400.0);

    let initial_position = state.position;
    let initial_velocity = state.velocity;
    let initial_energy = state.total_energy(constants.gravitational_parameter);

    // Run one physics step
    let dt = 0.1; // 0.1 seconds
    run_physics_step(&mut state, &constants, dt);

    // Position should have changed
    assert_ne!(
        state.position, initial_position,
        "Position should change after physics step"
    );

    // Velocity should have changed (due to gravitational acceleration)
    assert_ne!(
        state.velocity, initial_velocity,
        "Velocity should change after physics step"
    );

    // Energy should be approximately conserved (within numerical precision)
    let final_energy = state.total_energy(constants.gravitational_parameter);
    assert_energy_conserved(initial_energy, final_energy, 0.01); // 1% tolerance for Euler integration
}

#[test]
fn test_energy_conservation_over_time() {
    // Test that energy is conserved over multiple physics steps
    let constants = Constants::default();
    let mut state = create_test_orbital_state(400.0);

    let initial_energy = state.total_energy(constants.gravitational_parameter);

    // Run 100 physics steps
    let dt = 0.1;
    for _ in 0..100 {
        run_physics_step(&mut state, &constants, dt);
    }

    let final_energy = state.total_energy(constants.gravitational_parameter);

    // Energy should be conserved within reasonable tolerance for Euler integration
    // Euler integration has numerical errors, so we allow 5% tolerance
    assert_energy_conserved(initial_energy, final_energy, 0.05);
}

#[test]
fn test_orbital_period_preservation() {
    // Test that orbital period is approximately preserved
    let constants = Constants::default();
    let mut state = create_test_orbital_state(400.0);

    let initial_position = state.position;
    let initial_radius = initial_position.length() as f64;

    // Calculate expected orbital period: T = 2π * sqrt(r³/GM)
    let mu = constants.gravitational_parameter / 1e9; // Convert to km³/s²
    let expected_period =
        2.0 * std::f64::consts::PI * (initial_radius * initial_radius * initial_radius / mu).sqrt();

    // Run physics for approximately one orbital period
    let dt = 1.0; // 1 second timesteps
    let steps = (expected_period / dt) as usize;

    for _ in 0..steps {
        run_physics_step(&mut state, &constants, dt);
    }

    // After one period, position should be approximately back to initial
    // (allowing for numerical errors)
    let final_radius = state.position.length() as f64;
    assert_relative_eq!(final_radius, initial_radius, epsilon = 100.0);
}

#[test]
fn test_angular_momentum_conservation() {
    // Test that angular momentum is approximately conserved
    let constants = Constants::default();
    let mut state = create_test_orbital_state(400.0);

    // Angular momentum L = r × v
    let initial_r = state.position;
    let initial_v = state.velocity;
    let initial_l = initial_r.cross(initial_v).length();

    // Run physics for several steps
    let dt = 0.1;
    for _ in 0..50 {
        run_physics_step(&mut state, &constants, dt);
    }

    let final_r = state.position;
    let final_v = state.velocity;
    let final_l = final_r.cross(final_v).length();

    // Angular momentum should be approximately conserved
    // (Euler integration may introduce small errors)
    assert_relative_eq!(final_l, initial_l, epsilon = 0.1);
}

#[test]
fn test_physics_system_with_multiple_objects() {
    // Test that physics system handles multiple objects correctly
    let constants = Constants::default();

    let mut state1 = create_test_orbital_state(400.0);
    let mut state2 = create_test_orbital_state(500.0);
    let mut state3 = create_test_orbital_state(600.0);

    let initial_energy1 = state1.total_energy(constants.gravitational_parameter);
    let initial_energy2 = state2.total_energy(constants.gravitational_parameter);
    let initial_energy3 = state3.total_energy(constants.gravitational_parameter);

    // Run physics for all objects
    let dt = 0.1;
    for _ in 0..10 {
        run_physics_step(&mut state1, &constants, dt);
        run_physics_step(&mut state2, &constants, dt);
        run_physics_step(&mut state3, &constants, dt);
    }

    // Each object's energy should be conserved independently
    let final_energy1 = state1.total_energy(constants.gravitational_parameter);
    let final_energy2 = state2.total_energy(constants.gravitational_parameter);
    let final_energy3 = state3.total_energy(constants.gravitational_parameter);

    assert_energy_conserved(initial_energy1, final_energy1, 0.01);
    assert_energy_conserved(initial_energy2, final_energy2, 0.01);
    assert_energy_conserved(initial_energy3, final_energy3, 0.01);
}

#[test]
fn test_zero_velocity_case() {
    // Test edge case: object with zero velocity (should fall)
    let constants = Constants::default();
    let position = Vec3::new(6771.0, 0.0, 0.0); // 400 km altitude
    let velocity = Vec3::ZERO; // Zero velocity
    let mut state = OrbitalState::new(position, velocity, 1000.0);

    let initial_radius = state.position.length() as f64;

    // Run physics step
    run_physics_step(&mut state, &constants, 1.0);

    // Object should fall (radius should decrease)
    let final_radius = state.position.length() as f64;
    assert!(
        final_radius < initial_radius,
        "Object with zero velocity should fall: {:.2} -> {:.2} km",
        initial_radius,
        final_radius
    );
}

#[test]
fn test_very_high_altitude() {
    // Test physics at very high altitude (GEO)
    let constants = Constants::default();
    let mut state = create_test_orbital_state(35786.0); // GEO altitude

    let initial_energy = state.total_energy(constants.gravitational_parameter);

    // Run physics for a long time
    let dt = 1.0;
    for _ in 0..100 {
        run_physics_step(&mut state, &constants, dt);
    }

    let final_energy = state.total_energy(constants.gravitational_parameter);

    // Energy should still be conserved
    assert_energy_conserved(initial_energy, final_energy, 0.01);

    // Velocity should be lower at GEO
    let speed = state.speed();
    assert!(
        speed < 4.0,
        "Velocity at GEO should be less than 4 km/s: {:.2} km/s",
        speed
    );
}
