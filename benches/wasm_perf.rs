// WASM performance benchmarks
// Benchmarks physics system, collision detection, memory usage, and frame time consistency

use criterion::{black_box, criterion_group, criterion_main, Criterion};
// Note: Benchmarks need the crate to be a library
// For now, these benchmarks are disabled - convert to lib.rs + main.rs structure to enable
// use kessler_simulator::components::*;
// use kessler_simulator::resources::*;
// use kessler_simulator::systems::collision::*;
use bevy::prelude::*;

// Test utilities (inline for benchmarks)
fn create_test_orbital_state(altitude_km: f64) -> kessler_simulator::components::OrbitalState {
    let constants = kessler_simulator::resources::Constants::default();
    let radius_km = constants.earth_radius + altitude_km;
    let orbital_velocity = constants.circular_velocity(altitude_km);
    
    kessler_simulator::components::OrbitalState::new(
        Vec3::new(radius_km as f32, 0.0, 0.0),
        Vec3::new(0.0, orbital_velocity as f32, 0.0),
        1000.0,
    )
}

fn run_physics_step(state: &mut kessler_simulator::components::OrbitalState, constants: &kessler_simulator::resources::Constants, dt: f64) {
    let pos_x = state.position.x as f64;
    let pos_y = state.position.y as f64;
    let pos_z = state.position.z as f64;
    
    let vel_x = state.velocity.x as f64;
    let vel_y = state.velocity.y as f64;
    let vel_z = state.velocity.z as f64;

    let r_magnitude_km = (pos_x * pos_x + pos_y * pos_y + pos_z * pos_z).sqrt();
    let r_magnitude_m = r_magnitude_km * 1000.0;
    
    if r_magnitude_m > 0.0 {
        let gm = constants.gravitational_parameter;
        let acc_magnitude = -gm / (r_magnitude_m * r_magnitude_m);
        
        let r_unit_x = pos_x / r_magnitude_km;
        let r_unit_y = pos_y / r_magnitude_km;
        let r_unit_z = pos_z / r_magnitude_km;
        
        let acc_km_s2 = acc_magnitude / 1000.0;
        let acc_x = r_unit_x * acc_km_s2;
        let acc_y = r_unit_y * acc_km_s2;
        let acc_z = r_unit_z * acc_km_s2;

        let new_vel_x = vel_x + acc_x * dt;
        let new_vel_y = vel_y + acc_y * dt;
        let new_vel_z = vel_z + acc_z * dt;
        
        let new_pos_x = pos_x + new_vel_x * dt;
        let new_pos_y = pos_y + new_vel_y * dt;
        let new_pos_z = pos_z + new_vel_z * dt;

        state.velocity = Vec3::new(new_vel_x as f32, new_vel_y as f32, new_vel_z as f32);
        state.position = Vec3::new(new_pos_x as f32, new_pos_y as f32, new_pos_z as f32);
    }
}

fn bench_physics_step(c: &mut Criterion) {
    let constants = kessler_simulator::resources::Constants::default();
    let mut state = create_test_orbital_state(400.0);
    
    c.bench_function("physics_step", |b| {
        b.iter(|| {
            run_physics_step(black_box(&mut state), &constants, 0.1);
        });
    });
}

fn bench_physics_multiple_objects(c: &mut Criterion) {
    let constants = kessler_simulator::resources::Constants::default();
    let mut states: Vec<_> = (0..100)
        .map(|i| create_test_orbital_state(400.0 + i as f64 * 10.0))
        .collect();
    
    c.bench_function("physics_100_objects", |b| {
        b.iter(|| {
            for state in &mut states {
                run_physics_step(black_box(state), &constants, 0.1);
            }
        });
    });
}

fn bench_octree_insertion(_c: &mut Criterion) {
    // Benchmark disabled - requires library structure
    // use kessler_simulator::systems::collision::OctreeNode;
    // let mut octree = OctreeNode::new(Vec3::ZERO, 50000.0, 6, 0);
    /*
    c.bench_function("octree_insert_1000", |b| {
        b.iter(|| {
            octree.clear();
            for i in 0..1000 {
                let entity = Entity::from_raw(i);
                let position = Vec3::new(
                    (i as f32 % 1000.0) - 500.0,
                    ((i / 1000) as f32 % 1000.0) - 500.0,
                    ((i / 1000000) as f32 % 1000.0) - 500.0,
                );
                octree.insert(entity, position);
            }
        });
    });
    */
}

fn bench_octree_query(_c: &mut Criterion) {
    // Benchmark disabled - requires library structure
    /*
    use kessler_simulator::systems::collision::OctreeNode;
    let mut octree = OctreeNode::new(Vec3::ZERO, 50000.0, 6, 0);
    
    // Insert objects
    for i in 0..1000 {
        let entity = Entity::from_raw(i);
        let position = Vec3::new(
            (i as f32 % 1000.0) - 500.0,
            ((i / 1000) as f32 % 1000.0) - 500.0,
            ((i / 1000000) as f32 % 1000.0) - 500.0,
        );
        octree.insert(entity, position);
    }
    
    c.bench_function("octree_query_sphere", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            octree.query_sphere(black_box(Vec3::new(0.0, 0.0, 0.0)), black_box(100.0), &mut results);
            black_box(results);
        });
    });
    */
}

fn bench_energy_calculation(c: &mut Criterion) {
    let constants = kessler_simulator::resources::Constants::default();
    let state = create_test_orbital_state(400.0);
    
    c.bench_function("energy_calculation", |b| {
        b.iter(|| {
            black_box(state.total_energy(constants.gravitational_parameter));
        });
    });
}

fn bench_tle_parsing(c: &mut Criterion) {
    let tle_data = r#"ISS (ZARYA)
1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999
2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345"#;
    
    c.bench_function("tle_parsing", |b| {
        b.iter(|| {
            black_box(kessler_simulator::utils::tle_parser::parse_tle_data(black_box(tle_data)));
        });
    });
}

fn bench_tle_to_state_vectors(_c: &mut Criterion) {
    // Benchmark disabled - requires library structure
    /*
    use kessler_simulator::utils::tle_parser::TleRecord;
    use kessler_simulator::utils::sgp4_wrapper::tle_to_state_vectors;
    
    let tle = TleRecord {
        name: "TEST".to_string(),
        norad_id: 25544,
        classification: 'U',
        international_designator: "98067A".to_string(),
        epoch_year: 23,
        epoch_day: 200.0,
        mean_motion_dot: 0.0001,
        mean_motion_ddot: 0.0,
        bstar: 0.0,
        inclination: 51.6442,
        right_ascension: 123.4567,
        eccentricity: 0.001234,
        argument_of_perigee: 45.6789,
        mean_anomaly: 123.4567,
        mean_motion: 15.49,
        revolution_number: 12345,
        line1: "1 25544U 98067A   23200.12345678  .00001234  00000+0  12345-4 0  9999".to_string(),
        line2: "2 25544  51.6442 123.4567 0001234  45.6789 123.4567 15.49000000 12345".to_string(),
    };
    
    c.bench_function("tle_to_state_vectors", |b| {
        b.iter(|| {
            black_box(tle_to_state_vectors(black_box(&tle)));
        });
    });
    */
}

criterion_group!(
    benches,
    bench_physics_step,
    bench_physics_multiple_objects,
    bench_octree_insertion,
    bench_octree_query,
    bench_energy_calculation,
    bench_tle_parsing,
    bench_tle_to_state_vectors
);
criterion_main!(benches);

