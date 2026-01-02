// Integration tests for data loading
// Tests embedded TLE data loading, TLE parsing → satellite spawning, and test dataset fallback

use kessler_simulator::utils::*;
use std::fs;

mod common;
use common::*;

#[test]
fn test_embedded_tle_data_loading() {
    // Test loading TLE data from fixture files (simulating embedded data)
    let tle_data = fs::read_to_string("tests/fixtures/iss.tle")
        .expect("Should be able to read ISS fixture");
    
    let records = parse_tle_data(&tle_data).expect("Should parse embedded TLE data");
    assert_eq!(records.len(), 1, "Should parse one ISS record");
    assert_eq!(records[0].name, "ISS (ZARYA)");
    assert_eq!(records[0].norad_id, 25544);
}

#[test]
fn test_tle_parsing_to_satellite_creation() {
    // Test full pipeline: TLE parsing → SGP4 conversion → satellite creation
    let tle_data = fs::read_to_string("tests/fixtures/test_satellites.tle")
        .expect("Should be able to read test satellites fixture");
    
    let records = parse_tle_data(&tle_data).expect("Should parse TLE data");
    assert_eq!(records.len(), 3, "Should parse 3 test satellites");
    
    // Convert each TLE to state vectors
    for record in &records {
        let result = tle_to_state_vectors(record);
        assert!(result.is_ok(), "Should convert TLE {} to state vectors", record.name);
        
        let (position, velocity) = result.unwrap();
        
        // Verify state vectors are reasonable
        assert!(position.length() > 0.0, "Position should be non-zero");
        assert!(velocity.length() > 0.0, "Velocity should be non-zero");
    }
}

#[test]
fn test_multiple_tle_records_processing() {
    // Test that multiple TLE records are processed correctly
    let tle_data = fs::read_to_string("tests/fixtures/test_satellites.tle")
        .expect("Should be able to read test satellites fixture");
    
    let records = parse_tle_data(&tle_data).expect("Should parse multiple TLE records");
    
    // Verify all records are unique
    let mut norad_ids = std::collections::HashSet::new();
    for record in &records {
        assert!(!norad_ids.contains(&record.norad_id),
                "NORAD IDs should be unique");
        norad_ids.insert(record.norad_id);
    }
    
    assert_eq!(norad_ids.len(), records.len(),
               "Should have unique NORAD IDs for each record");
}

#[test]
fn test_test_dataset_fallback() {
    // Test that system can fall back to test dataset when real data fails
    // This simulates the fallback mechanism
    
    // Try to parse invalid data
    let invalid_data = "NOT VALID TLE DATA\nMORE INVALID DATA";
    let result = parse_tle_data(invalid_data);
    
    // Should handle gracefully (return empty vec or error)
    match result {
        Ok(records) => {
            // If it returns Ok, should be empty
            assert_eq!(records.len(), 0,
                      "Invalid data should result in empty records or error");
        }
        Err(_) => {
            // Error is also acceptable
        }
    }
    
    // Valid data should still work
    let valid_data = fs::read_to_string("tests/fixtures/iss.tle")
        .expect("Should be able to read ISS fixture");
    let valid_result = parse_tle_data(&valid_data);
    assert!(valid_result.is_ok(), "Valid data should parse successfully");
    assert!(valid_result.unwrap().len() > 0, "Valid data should produce records");
}

#[test]
fn test_tle_data_validation() {
    // Test that TLE data is validated during parsing
    let tle_data = fs::read_to_string("tests/fixtures/malformed.tle")
        .expect("Should be able to read malformed fixture");
    
    let result = parse_tle_data(&tle_data);
    
    // Should handle malformed data gracefully
    match result {
        Ok(records) => {
            // Should parse at least one valid record (the last one)
            assert!(records.len() >= 1,
                   "Should parse at least one valid record from malformed file");
        }
        Err(_) => {
            // Error is acceptable for malformed data
        }
    }
}

#[test]
fn test_tle_epoch_handling() {
    // Test that TLE epoch is handled correctly
    let tle_data = fs::read_to_string("tests/fixtures/iss.tle")
        .expect("Should be able to read ISS fixture");
    
    let records = parse_tle_data(&tle_data).expect("Should parse ISS TLE");
    let record = &records[0];
    
    // Epoch should be reasonable (year 23 = 2023, day should be in range)
    assert!(record.epoch_year >= 0 && record.epoch_year <= 99,
            "Epoch year should be in valid range");
    assert!(record.epoch_day > 0.0 && record.epoch_day <= 366.0,
            "Epoch day should be in valid range");
}

#[test]
fn test_tle_to_orbital_state_conversion() {
    // Test complete conversion: TLE → state vectors → orbital state
    let tle_data = fs::read_to_string("tests/fixtures/hubble.tle")
        .expect("Should be able to read Hubble fixture");
    
    let records = parse_tle_data(&tle_data).expect("Should parse Hubble TLE");
    let record = &records[0];
    
    // Convert to state vectors
    let (position, velocity) = tle_to_state_vectors(record)
        .expect("Should convert Hubble TLE to state vectors");
    
    // Create orbital state
    let orbital_state = kessler_simulator::components::OrbitalState::new(
        position, velocity, 11110.0 // Hubble mass
    );
    
    // Verify orbital state is valid
    let constants = kessler_simulator::resources::Constants::default();
    assert_orbital_valid(&orbital_state, &constants);
    
    // Verify altitude is reasonable for Hubble (~540 km)
    let altitude = orbital_state.altitude() - constants.earth_radius;
    assert!(altitude > 500.0 && altitude < 600.0,
            "Hubble altitude should be around 540 km: {:.2} km", altitude);
}

