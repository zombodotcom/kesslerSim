use crate::components::*;
use crate::resources::*;
use crate::systems::collision::CollisionPairs;
use crate::systems::stress_test::StressTestConfig;
use bevy::prelude::*;

pub fn hud_log_system(
    time: Res<Time>,
    satellite_query: Query<(), With<Satellite>>,
    debris_query: Query<(), With<Debris>>,
    collision_pairs: Res<CollisionPairs>,
    sim_time: Res<SimulationTime>,
    stress_config: Res<StressTestConfig>,
    mut last_log: Local<f32>,
) {
    let now = time.elapsed_secs();
    if now - *last_log < 1.0 {
        return;
    }
    *last_log = now;

    let fps = if time.delta_secs() > 0.0 {
        1.0 / time.delta_secs()
    } else {
        60.0
    };
    let frame_time = time.delta_secs() * 1000.0;

    let satellite_count = satellite_query.iter().count();
    let debris_count = debris_query.iter().count();
    let total = satellite_count + debris_count;
    let collision_count = collision_pairs.pairs.len();

    println!("========================================");
    println!("FPS: {:.1} ({:.2} ms)", fps, frame_time);
    println!(
        "Objects: {} total ({} satellites, {} debris)",
        total, satellite_count, debris_count
    );
    println!("Collisions this frame: {}", collision_count);
    println!(
        "Sim Speed: {:.0}x | Sim Time: {:.2}s",
        sim_time.speed_multiplier, sim_time.current
    );
    println!(
        "Status: {}",
        if sim_time.paused { "PAUSED" } else { "RUNNING" }
    );
    if stress_config.enabled {
        println!(
            "Spawning: {}/{}",
            stress_config.current_objects, stress_config.target_objects
        );
    }
    println!("========================================");
}
