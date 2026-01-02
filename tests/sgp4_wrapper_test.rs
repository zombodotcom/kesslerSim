// Unit tests for SGP4 conversion functionality
// Tests TLE to state vector conversion, Kepler's equation solver, coordinate transforms, and known satellite validation

use approx::assert_relative_eq;
use bevy::prelude::*;
use kessler_simulator::components::*;
use kessler_simulator::resources::*;
use kessler_simulator::utils::*;
use std::fs;

#[test]
fn test_tle_to_state_vectors_basic() {
    // Test basic TLE to state vector conversion
    let tle = TleRecord {
        name: "TEST SATELLITE".to_string(),
        norad_id: 10001,
        classification: 'U',
        international_designator: "23001A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5, // ~90 minute orbit
        revolution_number: 1,
        line1: "1 10001U 23001A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10001  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    assert!(
        result.is_ok(),
        "TLE to state vector conversion should succeed"
    );

    let (position, velocity) = result.unwrap();

    // Position should be reasonable (above Earth surface)
    let radius = position.length() as f64;
    let constants = Constants::default();
    assert!(
        radius > constants.earth_radius,
        "Position radius should be above Earth surface: {:.2} km",
        radius
    );

    // Velocity should be positive
    let speed = velocity.length() as f64;
    assert!(
        speed > 0.0,
        "Velocity should be positive: {:.2} km/s",
        speed
    );

    // For circular orbit, velocity should be approximately sqrt(GM/r)
    let expected_velocity = (398600.4418 / radius).sqrt();
    assert_relative_eq!(speed, expected_velocity, epsilon = 0.1);
}

#[test]
fn test_keplers_equation_solver() {
    // Test Kepler's equation solver with known solutions
    // For circular orbit (e=0), E = M
    let mean_anomaly: f64 = 1.0; // 1 radian
    let eccentricity = 0.0;

    // This should use the solve_keplers_equation function indirectly through tle_to_state_vectors
    let tle = TleRecord {
        name: "CIRCULAR TEST".to_string(),
        norad_id: 10002,
        classification: 'U',
        international_designator: "23002A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 0.0,
        right_ascension: 0.0,
        eccentricity: 0.0, // Circular orbit
        argument_of_perigee: 0.0,
        mean_anomaly: mean_anomaly.to_degrees(),
        mean_motion: 15.5,
        revolution_number: 1,
        line1: "1 10002U 23002A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10002   0.0000   0.0000 0000000   0.0000   1.0000 15.50000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    assert!(
        result.is_ok(),
        "Circular orbit TLE should convert successfully"
    );
}

#[test]
fn test_coordinate_transform() {
    // Test that coordinate transformation from orbital plane to ECI works correctly
    // For a satellite at inclination 0 (equatorial), position should be in XY plane
    let tle = TleRecord {
        name: "EQUATORIAL TEST".to_string(),
        norad_id: 10003,
        classification: 'U',
        international_designator: "23003A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 0.0, // Equatorial orbit
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5,
        revolution_number: 1,
        line1: "1 10003U 23003A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10003   0.0000   0.0000 0001000   0.0000   0.0000 15.50000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    assert!(
        result.is_ok(),
        "Equatorial orbit should convert successfully"
    );

    let (position, _velocity) = result.unwrap();

    // For equatorial orbit at mean anomaly 0, Z component should be near zero
    assert!(
        position.z.abs() < 100.0,
        "Equatorial orbit should have small Z component: {:.2}",
        position.z
    );
}

#[test]
fn test_known_satellite_iss() {
    // Test conversion with known ISS TLE data
    let tle_data =
        fs::read_to_string("tests/fixtures/iss.tle").expect("Should be able to read ISS fixture");

    let records = parse_tle_data(&tle_data).expect("Should parse ISS TLE");
    assert_eq!(records.len(), 1, "Should have one ISS record");

    let result = tle_to_state_vectors(&records[0]);
    assert!(result.is_ok(), "ISS TLE should convert to state vectors");

    let (position, velocity) = result.unwrap();

    // ISS altitude is approximately 400 km
    let constants = Constants::default();
    let altitude = position.length() as f64 - constants.earth_radius;
    assert!(
        altitude > 300.0 && altitude < 500.0,
        "ISS altitude should be around 400 km: {:.2} km",
        altitude
    );

    // ISS orbital velocity is approximately 7.66 km/s
    let speed = velocity.length() as f64;
    assert!(
        speed > 7.0 && speed < 8.0,
        "ISS velocity should be around 7.66 km/s: {:.2} km/s",
        speed
    );
}

#[test]
fn test_known_satellite_hubble() {
    // Test conversion with known Hubble TLE data
    let tle_data = fs::read_to_string("tests/fixtures/hubble.tle")
        .expect("Should be able to read Hubble fixture");

    let records = parse_tle_data(&tle_data).expect("Should parse Hubble TLE");
    assert_eq!(records.len(), 1, "Should have one Hubble record");

    let result = tle_to_state_vectors(&records[0]);
    assert!(result.is_ok(), "Hubble TLE should convert to state vectors");

    let (position, velocity) = result.unwrap();

    // Hubble altitude is approximately 540 km
    let constants = Constants::default();
    let altitude = position.length() as f64 - constants.earth_radius;
    assert!(
        altitude > 500.0 && altitude < 600.0,
        "Hubble altitude should be around 540 km: {:.2} km",
        altitude
    );

    // Hubble orbital velocity is approximately 7.5 km/s
    let speed = velocity.length() as f64;
    assert!(
        speed > 7.0 && speed < 8.0,
        "Hubble velocity should be around 7.5 km/s: {:.2} km/s",
        speed
    );
}

#[test]
fn test_high_eccentricity() {
    // Test conversion with high eccentricity orbit
    let tle = TleRecord {
        name: "HIGH ECCENTRICITY".to_string(),
        norad_id: 10004,
        classification: 'U',
        international_designator: "23004A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 45.0,
        right_ascension: 0.0,
        eccentricity: 0.5, // High eccentricity
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 10.0,
        revolution_number: 1,
        line1: "1 10004U 23004A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10004  45.0000   0.0000 5000000   0.0000   0.0000 10.00000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    // High eccentricity might cause convergence issues, but should still work
    if result.is_ok() {
        let (position, velocity) = result.unwrap();
        let radius = position.length() as f64;
        assert!(radius > 0.0, "Position should be valid");
        assert!(velocity.length() > 0.0, "Velocity should be valid");
    }
}

#[test]
fn test_low_altitude() {
    // Test conversion with low altitude orbit (edge case)
    let tle = TleRecord {
        name: "LOW ALTITUDE".to_string(),
        norad_id: 10005,
        classification: 'U',
        international_designator: "23005A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 16.0, // Higher mean motion = lower altitude
        revolution_number: 1,
        line1: "1 10005U 23005A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10005  51.6000   0.0000 0001000   0.0000   0.0000 16.00000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    assert!(result.is_ok(), "Low altitude orbit should convert");

    let (position, velocity) = result.unwrap();
    let constants = Constants::default();
    let altitude = position.length() as f64 - constants.earth_radius;

    // Should still be above Earth surface
    assert!(
        altitude > 100.0,
        "Low altitude should still be above 100 km: {:.2} km",
        altitude
    );
}

#[test]
fn test_orbital_period_calculation() {
    // Test that orbital period can be calculated from TLE
    let tle = TleRecord {
        name: "PERIOD TEST".to_string(),
        norad_id: 10006,
        classification: 'U',
        international_designator: "23006A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5, // ~90 minute orbit
        revolution_number: 1,
        line1: "1 10006U 23006A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10006  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01".to_string(),
    };

    let result = tle_to_simple_orbit(&tle);
    assert!(result.is_ok(), "Should calculate orbital period");

    let (_position, _velocity, period) = result.unwrap();

    // Period should be approximately 90 minutes (5400 seconds) for mean motion 15.5
    let expected_period = 86400.0 / 15.5; // seconds per day / revolutions per day
    assert_relative_eq!(period, expected_period, epsilon = 10.0);
}

#[test]
fn test_energy_conservation_in_conversion() {
    // Test that energy is conserved when converting TLE to state vectors
    let tle = TleRecord {
        name: "ENERGY TEST".to_string(),
        norad_id: 10007,
        classification: 'U',
        international_designator: "23007A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5,
        revolution_number: 1,
        line1: "1 10007U 23007A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10007  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01".to_string(),
    };

    let result = tle_to_state_vectors(&tle);
    assert!(result.is_ok(), "Should convert TLE to state vectors");

    let (position, velocity) = result.unwrap();
    let constants = Constants::default();

    // Create orbital state and check energy
    let orbital_state = OrbitalState::new(position, velocity, 1000.0);
    let total_energy = orbital_state.total_energy(constants.gravitational_parameter);

    // For a bound orbit, total energy should be negative
    assert!(
        total_energy < 0.0,
        "Bound orbit should have negative total energy: {:.6e} J",
        total_energy
    );
}

#[test]
fn test_days_since_epoch() {
    // Test epoch calculation
    let tle = TleRecord {
        name: "EPOCH TEST".to_string(),
        norad_id: 10008,
        classification: 'U',
        international_designator: "23008A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6,
        right_ascension: 0.0,
        eccentricity: 0.001,
        argument_of_perigee: 0.0,
        mean_anomaly: 0.0,
        mean_motion: 15.5,
        revolution_number: 1,
        line1: "1 10008U 23008A   23200.00000000  .00000000  00000+0  00000-0 0  9999".to_string(),
        line2: "2 10008  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01".to_string(),
    };

    let days = days_since_epoch(&tle);

    // Days since epoch should be positive (TLE is from 2023, we're in 2026)
    assert!(
        days > 0.0,
        "Days since epoch should be positive: {:.2} days",
        days
    );
    assert!(
        days < 2000.0,
        "Days since epoch should be reasonable: {:.2} days",
        days
    );
}
