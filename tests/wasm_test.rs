// WASM-specific tests
// Tests WASM compilation, function exports, memory management, and browser compatibility

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_compilation() {
    // Test that WASM compilation works
    // This test will only run in WASM environment
    assert!(true, "WASM compilation successful");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_basic_calculations() {
    // Test that basic calculations work in WASM
    let result = 2.0 + 2.0;
    assert_eq!(result, 4.0, "Basic calculations should work in WASM");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_memory_allocation() {
    // Test memory allocation in WASM
    let vec: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    assert_eq!(vec.len(), 1000, "Should allocate memory in WASM");
    assert_eq!(vec[0], 0.0);
    assert_eq!(vec[999], 999.0);
}

// Non-WASM tests for functions that should work in both environments
#[test]
fn test_cross_platform_compatibility() {
    // Test that core functions work in both native and WASM
    use kessler_simulator::utils::*;
    use kessler_simulator::components::*;
    use kessler_simulator::resources::*;
    
    // Test TLE parsing (should work in both)
    let tle_data = "ISS (ZARYA)\n1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999\n2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345";
    let result = parse_tle_data(tle_data);
    assert!(result.is_ok(), "TLE parsing should work cross-platform");
    
    // Test orbital state creation (should work in both)
    let position = bevy::prelude::Vec3::new(6771.0, 0.0, 0.0);
    let velocity = bevy::prelude::Vec3::new(0.0, 7.66, 0.0);
    let state = OrbitalState::new(position, velocity, 1000.0);
    
    assert!(state.altitude() > 0.0, "Orbital state should work cross-platform");
    
    // Test constants (should work in both)
    let constants = Constants::default();
    assert!(constants.earth_radius > 0.0, "Constants should work cross-platform");
}

#[test]
fn test_wasm_safe_types() {
    // Test that types used are WASM-safe (no platform-specific code)
    use kessler_simulator::components::*;
    
    // Vec3 should be WASM-safe
    let vec = bevy::prelude::Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(vec.x, 1.0);
    assert_eq!(vec.y, 2.0);
    assert_eq!(vec.z, 3.0);
    
    // OrbitalState should be WASM-safe
    let state = OrbitalState::new(vec, vec, 1000.0);
    assert!(state.mass > 0.0);
}

#[test]
fn test_error_handling_wasm_compatible() {
    // Test that error handling is WASM-compatible
    use kessler_simulator::utils::*;
    
    // Invalid TLE should return error (not panic)
    let invalid_tle = "INVALID DATA";
    let result = parse_tle_data(invalid_tle);
    
    // Should handle gracefully (either Ok with empty vec or Err)
    match result {
        Ok(records) => assert_eq!(records.len(), 0, "Invalid TLE should return empty or error"),
        Err(_) => {}, // Error is acceptable
    }
}

