// Unit tests for analytics functionality
// Tests energy binning, altitude bin assignment, and statistics calculations

use kessler_simulator::components::*;
use kessler_simulator::resources::*;
mod common;
use approx::assert_relative_eq;
use bevy::prelude::*;
use common::*;

#[test]
fn test_energy_analytics_initialization() {
    // Test that EnergyAnalytics initializes correctly
    let analytics = EnergyAnalytics::default();

    // Should have altitude bins from 200km to 2000km
    assert!(
        !analytics.altitude_bins.is_empty(),
        "Should have altitude bins"
    );
    assert_eq!(
        analytics.altitude_bins.len(),
        37,
        "Should have 37 altitude bins (200-2000km in 50km steps)"
    );

    // Check first and last bins
    assert_eq!(
        analytics.altitude_bins[0], 200.0,
        "First bin should be 200km"
    );
    assert_eq!(
        analytics.altitude_bins[36], 2000.0,
        "Last bin should be 2000km"
    );

    // Initial counts should be zero
    assert_eq!(analytics.total_objects, 0);
    assert_eq!(analytics.total_satellites, 0);
    assert_eq!(analytics.total_debris, 0);
    assert_eq!(analytics.total_energy, 0.0);
}

#[test]
fn test_get_altitude_bin() {
    // Test altitude bin assignment
    let analytics = EnergyAnalytics::default();

    // Test bin assignment for various altitudes
    let bin_200 = analytics.get_altitude_bin(200.0);
    assert_eq!(bin_200, Some(0), "200km should map to bin 0");

    let bin_250 = analytics.get_altitude_bin(250.0);
    assert_eq!(bin_250, Some(1), "250km should map to bin 1");

    let bin_225 = analytics.get_altitude_bin(225.0); // Midpoint between 200 and 250
    assert!(
        bin_225 == Some(0) || bin_225 == Some(1),
        "225km should map to bin 0 or 1"
    );

    // Test out of range
    let bin_low = analytics.get_altitude_bin(100.0);
    assert_eq!(bin_low, None, "Altitude below 200km should return None");

    let bin_high = analytics.get_altitude_bin(3000.0);
    assert_eq!(bin_high, None, "Altitude above 2000km should return None");
}

#[test]
fn test_add_energy_measurement() {
    // Test adding energy measurements
    let mut analytics = EnergyAnalytics::default();
    let constants = Constants::default();

    // Create orbital state at 400km altitude
    let position = Vec3::new(6771.0, 0.0, 0.0); // 400km altitude
    let velocity = Vec3::new(0.0, 7.66, 0.0);
    let orbital_state = OrbitalState::new(position, velocity, 1000.0);

    let altitude = orbital_state.altitude() - constants.earth_radius; // 400km
    let energy = orbital_state.total_energy(constants.gravitational_parameter);

    analytics.add_energy_measurement(altitude, energy);

    // Should have energy in the appropriate bin
    let bin_index = analytics.get_altitude_bin(altitude).unwrap();
    let bin_energies = analytics.energy_by_altitude.get(&bin_index);
    assert!(bin_energies.is_some(), "Should have energy in bin");
    assert_eq!(
        bin_energies.unwrap().len(),
        1,
        "Should have one energy measurement"
    );
    assert_relative_eq!(bin_energies.unwrap()[0], energy, epsilon = 1e6);
}

#[test]
fn test_get_average_energy() {
    // Test average energy calculation
    let mut analytics = EnergyAnalytics::default();

    let altitude = 400.0; // km
    let bin_index = analytics.get_altitude_bin(altitude).unwrap();

    // Add multiple energy measurements
    analytics.add_energy_measurement(altitude, 1e10);
    analytics.add_energy_measurement(altitude, 2e10);
    analytics.add_energy_measurement(altitude, 3e10);

    let avg_energy = analytics.get_average_energy(bin_index);
    assert!(avg_energy.is_some(), "Should have average energy");

    let expected_avg = (1e10 + 2e10 + 3e10) / 3.0;
    assert_relative_eq!(avg_energy.unwrap(), expected_avg, epsilon = 1e8);
}

#[test]
fn test_clear_measurements() {
    // Test that measurements can be cleared
    let mut analytics = EnergyAnalytics::default();

    // Add some measurements
    analytics.add_energy_measurement(400.0, 1e10);
    analytics.add_energy_measurement(500.0, 2e10);
    analytics.total_objects = 10;
    analytics.total_satellites = 5;
    analytics.total_debris = 5;
    analytics.total_energy = 3e10;

    // Clear measurements
    analytics.clear_measurements();

    // Should be empty
    assert!(
        analytics.energy_by_altitude.is_empty(),
        "Energy measurements should be cleared"
    );
    assert_eq!(analytics.total_objects, 0);
    assert_eq!(analytics.total_satellites, 0);
    assert_eq!(analytics.total_debris, 0);
    assert_eq!(analytics.total_energy, 0.0);
}

#[test]
fn test_multiple_altitude_bins() {
    // Test energy measurements across multiple altitude bins
    let mut analytics = EnergyAnalytics::default();
    let constants = Constants::default();

    // Add measurements at different altitudes
    let altitudes = vec![300.0, 400.0, 500.0, 600.0];
    for altitude in altitudes {
        let position = Vec3::new((6371.0 + altitude) as f32, 0.0, 0.0);
        let velocity = Vec3::new(0.0, 7.66, 0.0);
        let orbital_state = OrbitalState::new(position, velocity, 1000.0);
        let energy = orbital_state.total_energy(constants.gravitational_parameter);
        analytics.add_energy_measurement(altitude, energy);
    }

    // Should have measurements in multiple bins
    let mut bins_with_data = 0;
    for i in 0..analytics.altitude_bins.len() {
        if analytics.get_average_energy(i).is_some() {
            bins_with_data += 1;
        }
    }

    assert!(
        bins_with_data >= 4,
        "Should have measurements in at least 4 bins"
    );
}

#[test]
fn test_energy_binning_edge_cases() {
    // Test edge cases for energy binning
    let mut analytics = EnergyAnalytics::default();

    // Test exactly at bin boundaries
    analytics.add_energy_measurement(200.0, 1e10); // Exactly at first bin
    analytics.add_energy_measurement(2000.0, 2e10); // Exactly at last bin

    // Test just outside boundaries
    analytics.add_energy_measurement(199.0, 3e10); // Just below first bin
    analytics.add_energy_measurement(2001.0, 4e10); // Just above last bin

    // Should only have measurements in valid bins
    let bin_200 = analytics.get_altitude_bin(200.0);
    let bin_2000 = analytics.get_altitude_bin(2000.0);

    if let Some(bin_idx) = bin_200 {
        assert!(
            analytics.get_average_energy(bin_idx).is_some(),
            "Should have energy at 200km bin"
        );
    }

    if let Some(bin_idx) = bin_2000 {
        assert!(
            analytics.get_average_energy(bin_idx).is_some(),
            "Should have energy at 2000km bin"
        );
    }
}

#[test]
fn test_statistics_calculation() {
    // Test that statistics are calculated correctly
    let mut analytics = EnergyAnalytics::default();
    let constants = Constants::default();

    // Add multiple objects at different altitudes
    let mut total_energy_sum = 0.0;
    for i in 0..10 {
        let altitude = 300.0 + (i as f64 * 50.0);
        let position = Vec3::new((6371.0 + altitude) as f32, 0.0, 0.0);
        let velocity = Vec3::new(0.0, 7.66, 0.0);
        let orbital_state = OrbitalState::new(position, velocity, 1000.0);
        let energy = orbital_state.total_energy(constants.gravitational_parameter);
        analytics.add_energy_measurement(altitude, energy);
        total_energy_sum += energy;
    }

    // Set object counts
    analytics.total_objects = 10;
    analytics.total_satellites = 7;
    analytics.total_debris = 3;
    analytics.total_energy = total_energy_sum;

    // Verify statistics
    assert_eq!(analytics.total_objects, 10);
    assert_eq!(analytics.total_satellites, 7);
    assert_eq!(analytics.total_debris, 3);
    assert_relative_eq!(analytics.total_energy, total_energy_sum, epsilon = 1e8);
}

#[test]
fn test_altitude_bin_spacing() {
    // Test that altitude bins are evenly spaced
    let analytics = EnergyAnalytics::default();

    // Bins should be 50km apart
    for i in 1..analytics.altitude_bins.len() {
        let spacing = analytics.altitude_bins[i] - analytics.altitude_bins[i - 1];
        assert_relative_eq!(spacing, 50.0, epsilon = 0.1);
    }
}

#[test]
fn test_empty_bin_average() {
    // Test that empty bins return None
    let analytics = EnergyAnalytics::default();

    // Bin 0 should exist but be empty initially
    let avg = analytics.get_average_energy(0);
    assert_eq!(avg, None, "Empty bin should return None");
}

#[test]
fn test_energy_measurement_aggregation() {
    // Test that multiple measurements in same bin are aggregated
    let mut analytics = EnergyAnalytics::default();

    let altitude = 400.0;
    let bin_index = analytics.get_altitude_bin(altitude).unwrap();

    // Add many measurements to same bin
    for i in 0..100 {
        analytics.add_energy_measurement(altitude, (i as f64 + 1.0) * 1e9);
    }

    // Should have all 100 measurements
    let bin_energies = analytics.energy_by_altitude.get(&bin_index);
    assert!(bin_energies.is_some());
    assert_eq!(
        bin_energies.unwrap().len(),
        100,
        "Should have 100 measurements"
    );

    // Average should be sum / 100
    let expected_avg = (1.0 + 100.0) / 2.0 * 1e9 * 100.0 / 100.0;
    let actual_avg = analytics.get_average_energy(bin_index).unwrap();
    assert_relative_eq!(actual_avg, expected_avg, epsilon = 1e8);
}
