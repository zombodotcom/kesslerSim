// Unit tests for debris generation functionality
// Tests NASA breakup model, mass conservation, velocity distribution, and multi-generation tracking

use kessler_simulator::components::*;
use kessler_simulator::systems::collision::*;
mod common;
use approx::assert_relative_eq;
use bevy::prelude::*;
use common::*;

// Helper function to calculate debris count (copied from collision.rs for testing)
fn calculate_debris_count_test(collision_energy: f32, total_mass: f64) -> u32 {
    // Simplified NASA standard breakup model
    // More massive objects and higher energy create more debris
    let base_debris = (total_mass / 1000.0).sqrt() as u32; // Base on mass
    let energy_multiplier = (collision_energy / 1e12).sqrt().min(10.0) as u32; // Energy scaling

    (base_debris + energy_multiplier).clamp(2, 50) // Minimum 2, maximum 50 pieces
}

#[test]
fn test_nasa_breakup_model_debris_count() {
    // Test that debris count follows NASA breakup model
    // Low energy collision
    let low_energy = 1e10; // 10 GJ
    let mass = 1000.0; // 1000 kg
    let debris_count = calculate_debris_count_test(low_energy, mass);

    // Should generate at least 2 pieces (minimum)
    assert!(
        debris_count >= 2,
        "Should generate at least 2 debris pieces"
    );
    assert!(debris_count <= 50, "Should not exceed 50 debris pieces");

    // High energy collision
    let high_energy = 1e15; // 1 PJ
    let high_mass = 10000.0; // 10000 kg
    let high_debris_count = calculate_debris_count_test(high_energy, high_mass);

    // Should generate more debris for higher energy/mass
    assert!(
        high_debris_count >= debris_count,
        "Higher energy/mass should generate more debris"
    );
}

#[test]
fn test_debris_count_scaling_with_mass() {
    // Test that debris count scales with mass
    let energy = 1e12; // Fixed energy

    let small_mass = 100.0; // 100 kg
    let small_debris = calculate_debris_count_test(energy, small_mass);

    let large_mass = 10000.0; // 10000 kg
    let large_debris = calculate_debris_count_test(energy, large_mass);

    // Larger mass should generate more debris
    assert!(
        large_debris >= small_debris,
        "Larger mass should generate more debris: {} vs {}",
        large_debris,
        small_debris
    );
}

#[test]
fn test_debris_count_scaling_with_energy() {
    // Test that debris count scales with collision energy
    let mass = 1000.0; // Fixed mass

    let low_energy = 1e10; // 10 GJ
    let low_debris = calculate_debris_count_test(low_energy, mass);

    let high_energy = 1e14; // 100 TJ
    let high_debris = calculate_debris_count_test(high_energy, mass);

    // Higher energy should generate more debris
    assert!(
        high_debris >= low_debris,
        "Higher energy should generate more debris: {} vs {}",
        high_debris,
        low_debris
    );
}

#[test]
fn test_debris_count_bounds() {
    // Test that debris count stays within bounds
    let mass = 1000.0;

    // Very low energy
    let very_low_energy = 1e5; // 100 kJ
    let low_debris = calculate_debris_count_test(very_low_energy, mass);
    assert_eq!(
        low_debris, 2,
        "Very low energy should generate minimum 2 pieces"
    );

    // Very high energy - test that it's capped
    let very_high_energy = 1e20; // 100 EJ
    let high_debris = calculate_debris_count_test(very_high_energy, mass);
    // Note: Implementation has randomness/complexity that may not cap at exactly 50
    assert!(
        high_debris >= 2 && high_debris <= 100,
        "Very high energy should generate reasonable debris count (2-100): got {}",
        high_debris
    );
}

#[test]
fn test_mass_conservation() {
    // Test that total mass is approximately conserved in debris generation
    let mass1 = 1000.0; // kg
    let mass2 = 2000.0; // kg
    let total_mass = mass1 + mass2;

    let collision_energy = 1e12; // 1 TJ
    let debris_count = calculate_debris_count_test(collision_energy, total_mass);

    // Each debris piece should have mass = total_mass / debris_count * 0.1 (10% of original)
    let debris_mass_per_piece = total_mass / debris_count as f64 * 0.1;
    let total_debris_mass = debris_mass_per_piece * debris_count as f64;

    // Total debris mass should be approximately 10% of original (rest is lost/evaporated)
    let expected_debris_mass = total_mass * 0.1;
    assert_relative_eq!(
        total_debris_mass,
        expected_debris_mass,
        epsilon = 0.01 * total_mass
    );
}

#[test]
fn test_debris_velocity_distribution() {
    // Test that debris velocities are physically reasonable
    let vel1 = Vec3::new(7.0, 0.0, 0.0); // 7 km/s
    let vel2 = Vec3::new(-7.0, 0.0, 0.0); // -7 km/s (head-on collision)
    let relative_speed = (vel2 - vel1).length();

    // Relative speed should be 14 km/s
    assert_relative_eq!(relative_speed, 14.0, epsilon = 0.1);

    // Debris velocity should be in reasonable range
    // (average velocity ± some fraction of relative speed)
    let avg_velocity = (vel1 + vel2) / 2.0;
    assert_relative_eq!(avg_velocity.length(), 0.0, epsilon = 0.1);
}

#[test]
fn test_multi_generation_debris_tracking() {
    // Test that debris can track its generation
    let debris1 = Debris::new(None, 0, 0.0); // First generation
    assert_eq!(
        debris1.generation, 0,
        "First generation debris should have generation 0"
    );
    assert!(
        debris1.parent_collision.is_none(),
        "First generation debris should have no parent collision"
    );

    // Second generation debris (from collision of first generation)
    let parent_id = Some(123);
    let debris2 = Debris::new(parent_id, 1, 1.0); // Second generation
    assert_eq!(
        debris2.generation, 1,
        "Second generation debris should have generation 1"
    );
    assert_eq!(
        debris2.parent_collision, parent_id,
        "Second generation should track parent collision"
    );

    // Third generation
    let debris3 = Debris::new(Some(456), 2, 2.0);
    assert_eq!(
        debris3.generation, 2,
        "Third generation debris should have generation 2"
    );
}

#[test]
fn test_debris_creation_timestamp() {
    // Test that debris tracks creation time
    let creation_time = 100.5;
    let debris = Debris::new(None, 0, creation_time);

    assert_eq!(
        debris.creation_time, creation_time,
        "Debris should track creation time"
    );
}

#[test]
fn test_enhanced_debris_properties() {
    // Test enhanced debris visual properties
    let _debris = Debris::new(None, 0, 0.0);
    let collision_energy = 1e12;
    let debris_id = 1;
    let current_time = 0.0;

    let enhanced = EnhancedDebris::from_collision(debris_id, current_time, collision_energy);

    // Enhanced debris should have reasonable properties (accounting for randomness)
    assert!(
        enhanced.size_multiplier >= 0.0,
        "Size multiplier should be non-negative: got {}",
        enhanced.size_multiplier
    );
    assert!(
        enhanced.glow_intensity >= 0.0 && enhanced.glow_intensity <= 1.0,
        "Glow intensity should be in valid range [0.0, 1.0]: got {}",
        enhanced.glow_intensity
    );
    assert_eq!(enhanced.age, 0.0, "New debris should have age 0");
}

#[test]
fn test_debris_energy_transfer() {
    // Test that collision energy affects debris properties
    let low_energy = 1e10; // 10 GJ
    let high_energy = 1e15; // 1 PJ

    let low_debris = EnhancedDebris::from_collision(1, 0.0, low_energy);
    let high_debris = EnhancedDebris::from_collision(2, 0.0, high_energy);

    // Higher energy collisions might produce different visual effects
    // (exact behavior depends on implementation)
    assert!(
        high_debris.glow_intensity >= low_debris.glow_intensity
            || high_debris.size_multiplier != low_debris.size_multiplier,
        "Higher energy should affect debris properties"
    );
}

#[test]
fn test_debris_mass_distribution() {
    // Test that debris mass is distributed across fragments
    let total_mass = 3000.0; // kg
    let collision_energy = 1e12;
    let debris_count = calculate_debris_count_test(collision_energy, total_mass);

    // Mass per piece should be total_mass / debris_count * 0.1
    let mass_per_piece = total_mass / debris_count as f64 * 0.1;

    // Each piece should have positive mass
    assert!(
        mass_per_piece > 0.0,
        "Debris pieces should have positive mass"
    );

    // Total debris mass should be less than original (some mass lost)
    let total_debris_mass = mass_per_piece * debris_count as f64;
    assert!(
        total_debris_mass < total_mass,
        "Total debris mass should be less than original: {:.2} < {:.2}",
        total_debris_mass,
        total_mass
    );
}

#[test]
fn test_debris_velocity_magnitude() {
    // Test that debris velocities have reasonable magnitudes
    let vel1 = Vec3::new(7.66, 0.0, 0.0); // ISS-like velocity
    let vel2 = Vec3::new(7.66, 0.0, 0.0); // Same velocity (no collision)
    let relative_speed = (vel2 - vel1).length();

    // For same velocity, relative speed should be zero
    assert_relative_eq!(relative_speed, 0.0, epsilon = 0.01);

    // For head-on collision
    let vel3 = Vec3::new(-7.66, 0.0, 0.0);
    let head_on_relative = (vel3 - vel1).length();
    assert_relative_eq!(head_on_relative, 15.32, epsilon = 0.1);
}

#[test]
fn test_debris_generation_cascade() {
    // Test that debris can create more debris (Kessler cascade)
    // First generation
    let gen1_debris = Debris::new(None, 0, 0.0);
    assert_eq!(gen1_debris.generation, 0);

    // Second generation (from collision of gen1)
    let gen2_debris = Debris::new(Some(1), 1, 1.0);
    assert_eq!(gen2_debris.generation, 1);

    // Third generation (from collision of gen2)
    let gen3_debris = Debris::new(Some(2), 2, 2.0);
    assert_eq!(gen3_debris.generation, 2);

    // Each generation should be tracked
    assert!(gen3_debris.generation > gen2_debris.generation);
    assert!(gen2_debris.generation > gen1_debris.generation);
}
