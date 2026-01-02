// Property-based tests for orbital mechanics
// Tests period conservation, semi-major axis, eccentricity, and angular momentum

use approx::assert_relative_eq;
use bevy::prelude::*;
use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use proptest::prelude::*;

mod common;
use common::*;

proptest! {
    #[test]
    fn test_orbital_period_preservation(
        altitude in 200.0..2000.0f64,
    ) {
        // Test that orbital period is approximately preserved for circular orbits
        let constants = Constants::default();
        let mut state = create_test_orbital_state(altitude);

        let initial_radius = state.position.length() as f64;

        // Calculate expected orbital period: T = 2π * sqrt(r³/GM)
        let mu = constants.gravitational_parameter / 1e9; // Convert to km³/s²
        let expected_period = 2.0 * std::f64::consts::PI *
            (initial_radius * initial_radius * initial_radius / mu).sqrt();

        // Run physics for approximately one orbital period
        let dt = 1.0;
        let steps = (expected_period / dt) as usize;
        let max_steps = steps.min(10000); // Limit to prevent very long tests

        for _ in 0..max_steps {
            run_physics_step(&mut state, &constants, dt);
        }

        // After one period, radius should be approximately preserved
        let final_radius = state.position.length() as f64;
        assert_relative_eq!(final_radius, initial_radius, epsilon = 200.0);
    }

    #[test]
    fn test_semi_major_axis_preservation(
        altitude in 200.0..2000.0f64,
    ) {
        // Test that semi-major axis is approximately preserved
        let constants = Constants::default();
        let mut state = create_test_orbital_state(altitude);

        let initial_radius = state.position.length() as f64;
        let initial_energy = state.total_energy(constants.gravitational_parameter);

        // For circular orbit, semi-major axis = radius
        // a = -GMm / (2E) for bound orbits
        let mu = constants.gravitational_parameter / 1e9; // km³/s²
        let initial_semi_major = -mu * state.mass / (2.0 * initial_energy) / 1e9; // Convert back

        // Run physics for many steps
        let dt = 0.1;
        for _ in 0..100 {
            run_physics_step(&mut state, &constants, dt);
        }

        let final_energy = state.total_energy(constants.gravitational_parameter);
        let final_semi_major = -mu * state.mass / (2.0 * final_energy) / 1e9;

        // Semi-major axis should be approximately preserved
        assert_relative_eq!(final_semi_major, initial_semi_major, epsilon = 100.0);
    }

    #[test]
    fn test_angular_momentum_conservation(
        altitude in 200.0..2000.0f64,
    ) {
        // Test that angular momentum is approximately conserved
        let constants = Constants::default();
        let mut state = create_test_orbital_state(altitude);

        // Angular momentum L = r × v
        let initial_r = state.position;
        let initial_v = state.velocity;
        let initial_l = initial_r.cross(initial_v).length();

        // Run physics for many steps
        let dt = 0.1;
        for _ in 0..100 {
            run_physics_step(&mut state, &constants, dt);
        }

        let final_r = state.position;
        let final_v = state.velocity;
        let final_l = final_r.cross(final_v).length();

        // Angular momentum should be approximately conserved
        assert_relative_eq!(final_l, initial_l, epsilon = 0.5);
    }

    #[test]
    fn test_energy_conservation_over_long_time(
        altitude in 200.0..2000.0f64,
    ) {
        // Test that energy is conserved over long simulation time
        let constants = Constants::default();
        let mut state = create_test_orbital_state(altitude);

        let initial_energy = state.total_energy(constants.gravitational_parameter);

        // Run physics for many steps (simulating long time)
        let dt = 0.1;
        for _ in 0..1000 {
            run_physics_step(&mut state, &constants, dt);
        }

        let final_energy = state.total_energy(constants.gravitational_parameter);

        // Energy should be conserved (within numerical precision)
        assert_energy_conserved(initial_energy, final_energy, 0.1); // 10% tolerance for long simulation
    }

    #[test]
    fn test_orbital_velocity_scaling(
        altitude1 in 200.0..1000.0f64,
        altitude2 in 1000.0..2000.0f64,
    ) {
        prop_assume!(altitude2 > altitude1);

        // Test that orbital velocity decreases with altitude
        let constants = Constants::default();

        let vel1 = constants.circular_velocity(altitude1);
        let vel2 = constants.circular_velocity(altitude2);

        // Higher altitude should have lower velocity
        assert!(vel2 < vel1,
               "Higher altitude should have lower velocity: {} < {}", vel2, vel1);

        // Velocity should scale as sqrt(1/r)
        let r1 = constants.earth_radius + altitude1;
        let r2 = constants.earth_radius + altitude2;
        // v = sqrt(GM/r), so v1/v2 = sqrt(r2/r1)
        let expected_ratio = (r2 / r1).sqrt();
        let actual_ratio = vel1 / vel2;

        assert_relative_eq!(actual_ratio, expected_ratio, epsilon = 0.1);
    }
}
