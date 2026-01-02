// Property-based tests for energy conservation
// Uses proptest to generate random orbital states and verify energy conservation

use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use bevy::prelude::*;
use proptest::prelude::*;
use approx::assert_relative_eq;

mod common;
use common::*;

proptest! {
    #[test]
    fn test_energy_conservation_random_states(
        altitude in 200.0..2000.0f64,
        mass in 100.0..10000.0f64,
    ) {
        // Generate random orbital state
        let mut state = create_test_orbital_state(altitude);
        state.mass = mass;
        
        let constants = Constants::default();
        let initial_energy = state.total_energy(constants.gravitational_parameter);
        
        // Run physics for multiple steps
        let dt = 0.1;
        for _ in 0..50 {
            run_physics_step(&mut state, &constants, dt);
        }
        
        let final_energy = state.total_energy(constants.gravitational_parameter);
        
        // Energy should be conserved within 5% (Euler integration has numerical errors)
        assert_energy_conserved(initial_energy, final_energy, 0.05);
    }
    
    #[test]
    fn test_energy_scaling_with_mass(
        altitude in 200.0..2000.0f64,
        mass1 in 100.0..5000.0f64,
        mass2 in 100.0..5000.0f64,
    ) {
        // Test that energy scales correctly with mass
        let constants = Constants::default();
        
        let mut state1 = create_test_orbital_state(altitude);
        state1.mass = mass1;
        
        let mut state2 = create_test_orbital_state(altitude);
        state2.mass = mass2;
        
        let energy1 = state1.total_energy(constants.gravitational_parameter);
        let energy2 = state2.total_energy(constants.gravitational_parameter);
        
        // Energy should scale linearly with mass (for same position/velocity)
        let energy_ratio = energy1 / energy2;
        let mass_ratio = mass1 / mass2;
        
        assert_relative_eq!(energy_ratio, mass_ratio, epsilon = 0.01);
    }
    
    #[test]
    fn test_potential_energy_decreases_with_altitude(
        altitude1 in 200.0..1000.0f64,
        altitude2 in 1000.0..2000.0f64,
        mass in 1000.0..5000.0f64,
    ) {
        prop_assume!(altitude2 > altitude1);
        
        // Test that potential energy becomes less negative (closer to zero) at higher altitude
        let constants = Constants::default();
        
        let position1 = Vec3::new((6371.0 + altitude1) as f32, 0.0, 0.0);
        let velocity1 = Vec3::new(0.0, 7.66, 0.0);
        let state1 = OrbitalState::new(position1, velocity1, mass);
        
        let position2 = Vec3::new((6371.0 + altitude2) as f32, 0.0, 0.0);
        let velocity2 = Vec3::new(0.0, 7.66, 0.0);
        let state2 = OrbitalState::new(position2, velocity2, mass);
        
        let pe1 = state1.potential_energy(constants.gravitational_parameter);
        let pe2 = state2.potential_energy(constants.gravitational_parameter);
        
        // Higher altitude should have less negative potential energy
        assert!(pe2 > pe1, "Higher altitude should have less negative PE: {} > {}", pe2, pe1);
    }
    
    #[test]
    fn test_kinetic_energy_increases_with_velocity(
        altitude in 200.0..2000.0f64,
        speed1 in 5.0..8.0f64,
        speed2 in 8.0..12.0f64,
        mass in 1000.0..5000.0f64,
    ) {
        prop_assume!(speed2 > speed1);
        
        // Test that kinetic energy increases with velocity magnitude
        let radius = 6371.0 + altitude;
        let position = Vec3::new(radius as f32, 0.0, 0.0);
        
        let velocity1 = Vec3::new(0.0, speed1 as f32, 0.0);
        let state1 = OrbitalState::new(position, velocity1, mass);
        
        let velocity2 = Vec3::new(0.0, speed2 as f32, 0.0);
        let state2 = OrbitalState::new(position, velocity2, mass);
        
        let ke1 = state1.kinetic_energy();
        let ke2 = state2.kinetic_energy();
        
        // KE = 0.5 * m * v², so KE2/KE1 = (v2/v1)²
        let expected_ratio = (speed2 / speed1).powi(2);
        let actual_ratio = ke2 / ke1;
        
        assert_relative_eq!(actual_ratio, expected_ratio, epsilon = 0.01);
    }
    
    #[test]
    fn test_total_energy_negative_for_bound_orbits(
        altitude in 200.0..2000.0f64,
        mass in 100.0..10000.0f64,
    ) {
        // Test that bound orbits have negative total energy
        let constants = Constants::default();
        let state = create_test_orbital_state(altitude);
        
        let total_energy = state.total_energy(constants.gravitational_parameter);
        
        // Bound orbits should have negative total energy
        assert!(total_energy < 0.0,
               "Bound orbit should have negative total energy: {:.6e} J", total_energy);
    }
}

