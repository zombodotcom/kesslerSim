// Unit tests for TLE parser functionality
// Tests valid parsing, edge cases, error handling, exponential notation, and multiple records

use kessler_simulator::utils::*;
use std::fs;

#[test]
fn test_parse_valid_tle() {
    // Test parsing a valid TLE record
    let name = "ISS (ZARYA)";
    let line1 = "1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999";
    let line2 = "2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_ok(), "Valid TLE should parse successfully");
    
    let tle = result.unwrap();
    assert_eq!(tle.name, "ISS (ZARYA)");
    assert_eq!(tle.norad_id, 25544);
    assert_eq!(tle.classification, 'U');
    assert_eq!(tle.international_designator, "98067A");
    assert_eq!(tle.epoch_year, 23);
    assert!((tle.epoch_day - 200.12345678).abs() < 0.0001);
    assert!((tle.inclination - 51.6442).abs() < 0.0001);
    assert!((tle.right_ascension - 123.4567).abs() < 0.0001);
    assert!((tle.mean_motion - 15.49).abs() < 0.01);
}

#[test]
fn test_parse_exponential_notation() {
    // Test parsing exponential notation like "12345-3"
    let name = "TEST SATELLITE";
    let line1 = "1 10001U 23001A   23200.00000000  .00000000  12345-3  00000-0 0  9999";
    let line2 = "2 10001  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_ok(), "TLE with exponential notation should parse");
    
    let tle = result.unwrap();
    // mean_motion_ddot should be approximately 0.12345e-3
    assert!(tle.mean_motion_ddot.abs() > 0.0, "Exponential notation should be parsed");
}

#[test]
fn test_parse_decimal_fraction() {
    // Test parsing decimal fraction for eccentricity
    let name = "TEST SATELLITE";
    let line1 = "1 10001U 23001A   23200.00000000  .00000000  00000+0  00000-0 0  9999";
    let line2 = "2 10001  51.6000   0.0000 0001234   0.0000   0.0000 15.50000000    01";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_ok(), "TLE with decimal fraction should parse");
    
    let tle = result.unwrap();
    // Eccentricity should be 0.0001234 (7 digits, so divided by 10^7)
    assert!((tle.eccentricity - 0.0001234).abs() < 0.0000001, 
            "Decimal fraction should be parsed correctly");
}

#[test]
fn test_parse_multiple_records() {
    // Test parsing multiple TLE records from a file
    let tle_data = r#"ISS (ZARYA)
1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999
2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345
HUBBLE SPACE TELESCOPE
1 20580U 90037A   23200.12345678  .00001234  00000+0  12345-4 0  9999
2 20580  28.4692 234.5678 0002345  56.7890 234.5678 15.09000000 67890"#;
    
    let result = parse_tle_data(tle_data);
    assert!(result.is_ok(), "Multiple TLE records should parse");
    
    let records = result.unwrap();
    assert_eq!(records.len(), 2, "Should parse 2 TLE records");
    assert_eq!(records[0].name, "ISS (ZARYA)");
    assert_eq!(records[1].name, "HUBBLE SPACE TELESCOPE");
}

#[test]
fn test_parse_from_fixture_file() {
    // Test parsing from actual fixture file
    let tle_data = fs::read_to_string("tests/fixtures/iss.tle")
        .expect("Should be able to read ISS fixture file");
    
    let result = parse_tle_data(&tle_data);
    assert!(result.is_ok(), "Should parse ISS fixture file");
    
    let records = result.unwrap();
    assert_eq!(records.len(), 1, "ISS fixture should contain 1 record");
    assert_eq!(records[0].norad_id, 25544);
}

#[test]
fn test_parse_invalid_length() {
    // Test error handling for lines that are too short
    let name = "TEST SATELLITE";
    let line1 = "1 10001U 23001A   23200.0"; // Too short
    let line2 = "2 10001  51.6000"; // Too short
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_err(), "TLE with invalid length should return error");
    
    match result.unwrap_err() {
        TleParseError::InvalidLength => {}, // Expected
        _ => panic!("Should return InvalidLength error"),
    }
}

#[test]
fn test_parse_invalid_field() {
    // Test error handling for invalid numeric fields
    let name = "TEST SATELLITE";
    let line1 = "1 INVALIDU 23001A   23200.00000000  .00000000  00000+0  00000-0 0  9999";
    let line2 = "2 10001  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_err(), "TLE with invalid NORAD ID should return error");
    
    match result.unwrap_err() {
        TleParseError::InvalidField(_) => {}, // Expected
        _ => panic!("Should return InvalidField error"),
    }
}

#[test]
fn test_parse_empty_exponential() {
    // Test parsing empty or zero exponential notation
    let name = "TEST SATELLITE";
    let line1 = "1 10001U 23001A   23200.00000000  .00000000  00000+0  00000-0 0  9999";
    let line2 = "2 10001  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    assert!(result.is_ok(), "TLE with zero exponential should parse");
    
    let tle = result.unwrap();
    assert_eq!(tle.mean_motion_ddot, 0.0, "Zero exponential should be 0.0");
    assert_eq!(tle.bstar, 0.0, "Zero bstar should be 0.0");
}

#[test]
fn test_parse_malformed_data() {
    // Test parsing malformed TLE data (should skip invalid records)
    let tle_data = fs::read_to_string("tests/fixtures/malformed.tle")
        .expect("Should be able to read malformed fixture file");
    
    let result = parse_tle_data(&tle_data);
    assert!(result.is_ok(), "Parser should handle malformed data gracefully");
    
    let records = result.unwrap();
    // Should parse at least one valid record (the last one)
    assert!(records.len() >= 1, "Should parse at least one valid record from malformed file");
}

#[test]
fn test_parse_boundary_values() {
    // Test parsing boundary values (min/max field values)
    let name = "BOUNDARY TEST";
    // Test with maximum reasonable values
    let line1 = "1 99999U 99999Z   99365.99999999  .99999999  99999+9  99999-9 0  9999";
    let line2 = "2 99999 180.0000 360.0000 9999999 360.0000 360.0000 99.99999999 99999";
    
    let result = TleRecord::from_tle_lines(name, line1, line2);
    // This might fail due to invalid values, but should handle gracefully
    if result.is_ok() {
        let tle = result.unwrap();
        assert_eq!(tle.norad_id, 99999);
        assert!((tle.inclination - 180.0).abs() < 0.0001);
    }
}

#[test]
fn test_parse_real_tle_format() {
    // Test parsing with real-world TLE format from fixtures
    let tle_data = fs::read_to_string("tests/fixtures/test_satellites.tle")
        .expect("Should be able to read test satellites fixture");
    
    let result = parse_tle_data(&tle_data);
    assert!(result.is_ok(), "Should parse real TLE format");
    
    let records = result.unwrap();
    assert_eq!(records.len(), 3, "Should parse all 3 test satellites");
    
    // Verify each record has valid data
    for record in &records {
        assert!(!record.name.is_empty(), "Record should have a name");
        assert!(record.norad_id > 0, "NORAD ID should be positive");
        assert!(record.inclination >= 0.0 && record.inclination <= 180.0, 
                "Inclination should be in valid range");
        assert!(record.eccentricity >= 0.0 && record.eccentricity < 1.0,
                "Eccentricity should be in valid range");
    }
}

#[test]
fn test_tle_record_clone() {
    // Test that TLE records can be cloned (needed for some test scenarios)
    let name = "TEST SATELLITE";
    let line1 = "1 10001U 23001A   23200.00000000  .00000000  00000+0  00000-0 0  9999";
    let line2 = "2 10001  51.6000   0.0000 0001000   0.0000   0.0000 15.50000000    01";
    
    let tle1 = TleRecord::from_tle_lines(name, line1, line2).unwrap();
    let tle2 = tle1.clone();
    
    assert_eq!(tle1.name, tle2.name);
    assert_eq!(tle1.norad_id, tle2.norad_id);
    assert_eq!(tle1.inclination, tle2.inclination);
}

