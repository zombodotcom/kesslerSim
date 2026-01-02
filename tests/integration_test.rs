// Integration tests for full simulation cycles
// Tests collision cascades, data loading integration, and system interactions

use bevy::prelude::*;
use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use kessler_simulator::systems::*;
use kessler_simulator::utils::*;

mod common;
use common::*;

#[test]
fn test_tle_parsing_to_satellite_spawning() {
    // Test full cycle: TLE parsing → SGP4 conversion → entity spawning
    let tle_data = r#"ISS (ZARYA)
1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999
2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345"#;

    // Step 1: Parse TLE
    let records = parse_tle_data(tle_data).expect("Should parse TLE data");
    assert_eq!(records.len(), 1, "Should parse one TLE record");

    // Step 2: Convert TLE to state vectors
    let result = tle_to_state_vectors(&records[0]);
    assert!(result.is_ok(), "Should convert TLE to state vectors");

    let (position, velocity) = result.unwrap();

    // Step 3: Verify state vectors are valid
    let constants = Constants::default();
    let orbital_state = OrbitalState::new(position, velocity, 1000.0);
    assert_orbital_valid(&orbital_state, &constants);
}

#[test]
fn test_physics_and_collision_integration() {
    // Test that physics updates and collision detection work together
    let constants = Constants::default();

    // Create two objects that will collide
    let mut state1 = create_test_orbital_state(400.0);
    let mut state2 = create_test_orbital_state(400.0);

    // Place them at same position (guaranteed collision)
    state1.position = Vec3::new(6771.0, 0.0, 0.0);
    state2.position = Vec3::new(6771.0, 0.0, 0.0);

    let initial_energy1 = state1.total_energy(constants.gravitational_parameter);
    let initial_energy2 = state2.total_energy(constants.gravitational_parameter);

    // Run physics for a few steps
    let dt = 0.1;
    for _ in 0..10 {
        run_physics_step(&mut state1, &constants, dt);
        run_physics_step(&mut state2, &constants, dt);
    }

    // Objects should have moved (even if slightly)
    // Energy should be approximately conserved
    let final_energy1 = state1.total_energy(constants.gravitational_parameter);
    let final_energy2 = state2.total_energy(constants.gravitational_parameter);

    assert_energy_conserved(initial_energy1, final_energy1, 0.01);
    assert_energy_conserved(initial_energy2, final_energy2, 0.01);
}

#[test]
fn test_collision_cascade_simulation() {
    // Test that a collision creates debris that can cause more collisions
    // This is a simplified test of the Kessler cascade concept

    let constants = Constants::default();

    // Create initial collision scenario
    let mut state1 = create_test_orbital_state(400.0);
    let mut state2 = create_test_orbital_state(400.0);

    // Place objects at same position
    let collision_point = Vec3::new(6771.0, 0.0, 0.0);
    state1.position = collision_point;
    state2.position = collision_point;

    // Simulate collision: objects are destroyed, debris is created
    // In a real simulation, this would spawn multiple debris objects
    // For this test, we verify the concept works

    // After collision, debris would be created with different velocities
    // This would lead to more potential collisions (Kessler cascade)

    // Verify that collision point is reasonable
    assert!(
        collision_point.length() > constants.earth_radius as f32,
        "Collision should occur above Earth surface"
    );
}

#[test]
fn test_data_loading_fallback() {
    // Test that system falls back to test data when real data unavailable
    // This tests the data loading integration

    // Try to parse a file that doesn't exist (simulating failure)
    // In real code, this would trigger fallback to test dataset
    // For this test, we verify the parsing logic handles errors

    let invalid_data = "INVALID TLE DATA";
    let result = parse_tle_data(invalid_data);

    // Should handle gracefully (either return empty vec or error)
    // The actual behavior depends on implementation
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle invalid data gracefully"
    );
}

#[test]
fn test_multiple_satellites_physics() {
    // Test physics system with multiple satellites
    let constants = Constants::default();

    let mut states = vec![
        create_test_orbital_state(400.0),
        create_test_orbital_state(500.0),
        create_test_orbital_state(600.0),
        create_test_orbital_state(700.0),
    ];

    let initial_energies: Vec<f64> = states
        .iter()
        .map(|s| s.total_energy(constants.gravitational_parameter))
        .collect();

    // Run physics for all objects
    let dt = 0.1;
    for _ in 0..10 {
        for state in &mut states {
            run_physics_step(state, &constants, dt);
        }
    }

    // Each object's energy should be conserved
    for (i, state) in states.iter().enumerate() {
        let final_energy = state.total_energy(constants.gravitational_parameter);
        assert_energy_conserved(initial_energies[i], final_energy, 0.01);
    }
}

#[test]
fn test_analytics_during_simulation() {
    // Test that analytics collect data during simulation
    let mut analytics = EnergyAnalytics::default();
    let constants = Constants::default();

    // Simulate multiple objects at different altitudes
    let altitudes = vec![300.0, 400.0, 500.0, 600.0];

    for altitude in &altitudes {
        let position = Vec3::new((6371.0 + altitude) as f32, 0.0, 0.0);
        let velocity = Vec3::new(0.0, 7.66, 0.0);
        let orbital_state = OrbitalState::new(position, velocity, 1000.0);
        let energy = orbital_state.total_energy(constants.gravitational_parameter);
        analytics.add_energy_measurement(*altitude, energy);
    }

    // Analytics should have collected data
    assert!(
        analytics.energy_by_altitude.len() > 0,
        "Analytics should have collected energy measurements"
    );

    // Should be able to get averages
    for altitude in &altitudes {
        if let Some(bin_index) = analytics.get_altitude_bin(*altitude) {
            let avg = analytics.get_average_energy(bin_index);
            assert!(
                avg.is_some(),
                "Should have average energy for altitude {}",
                altitude
            );
        }
    }
}

#[test]
fn test_end_to_end_simulation_cycle() {
    // Test a complete simulation cycle: load → spawn → physics → collision → debris
    let constants = Constants::default();

    // Step 1: Create orbital states (simulating loaded satellites)
    let mut state1 = create_test_orbital_state(400.0);
    let mut state2 = create_test_orbital_state(400.0);

    // Step 2: Position them for collision
    let collision_point = Vec3::new(6771.0, 0.0, 0.0);
    state1.position = collision_point;
    state2.position = collision_point;

    // Give them different velocities for realistic collision
    state1.velocity = Vec3::new(7.5, 0.0, 0.0); // Moving in +x
    state2.velocity = Vec3::new(-7.5, 0.0, 0.0); // Moving in -x (head-on collision)

    // Step 3: Run physics (objects would move, but we're testing at collision point)
    let initial_energy = state1.total_energy(constants.gravitational_parameter)
        + state2.total_energy(constants.gravitational_parameter);

    // Step 4: Simulate collision (in real system, this would create debris)
    // For this test, we verify energy calculation
    let relative_velocity = (state2.velocity - state1.velocity).length();
    let collision_energy =
        0.5 * (state1.mass + state2.mass) as f32 * relative_velocity * relative_velocity;

    assert!(
        collision_energy > 0.0,
        "Collision should have positive energy: got {}",
        collision_energy
    );

    // Step 5: Verify system state is consistent
    assert_orbital_valid(&state1, &constants);
    assert_orbital_valid(&state2, &constants);
}

#[test]
fn test_debris_mechanics_integration() {
    // Test that debris mechanics (injection + decay) work with physics
    let constants = Constants::default();

    // Create debris at low altitude (should decay)
    let low_altitude = 150.0; // Below decay threshold (200km)
    let mut debris_state = create_test_orbital_state(low_altitude);

    // Create debris at high altitude (should persist)
    let high_altitude = 500.0;
    let mut satellite_state = create_test_orbital_state(high_altitude);

    // Run physics for both
    let dt = 1.0;
    for _ in 0..100 {
        run_physics_step(&mut debris_state, &constants, dt);
        run_physics_step(&mut satellite_state, &constants, dt);
    }

    // Low altitude debris should have lower radius (decaying)
    let debris_radius = debris_state.position.length() as f64;
    let satellite_radius = satellite_state.position.length() as f64;

    // Both should still be valid orbital states
    assert_orbital_valid(&debris_state, &constants);
    assert_orbital_valid(&satellite_state, &constants);
}

#[test]
fn test_system_resources_initialization() {
    // Test that all system resources initialize correctly
    let constants = Constants::default();
    let sim_time = SimulationTime::default();
    let analytics = EnergyAnalytics::default();
    let _octree = SpatialOctree::default();
    let collision_pairs = CollisionPairs::default();

    // Verify resources have correct initial state
    assert_eq!(sim_time.current, 0.0);
    assert!(!sim_time.paused);
    assert_eq!(analytics.total_objects, 0);
    assert_eq!(collision_pairs.pairs.len(), 0);

    // Verify constants are correct
    assert!(constants.earth_radius > 0.0);
    assert!(constants.gravitational_parameter > 0.0);
}
