use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;
pub mod utils;

use resources::*;
use systems::*;
use systems::materials::MaterialsCache;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "info,kessler_simulator::systems::collision=warn,bevy_render=warn,bevy_ecs=warn".to_string(),
            ..default()
        }))
        .init_resource::<Constants>()
        .init_resource::<SimulationTime>()
        .init_resource::<EnergyAnalytics>()
        .init_resource::<TleDataCache>()
        .init_resource::<SpatialOctree>()
        .init_resource::<CollisionPairs>()
        .init_resource::<OptimizedPhysicsData>()
        .init_resource::<StressTestConfig>()
        .init_resource::<DebrisInjectionConfig>()
        .init_resource::<OrbitalDecayConfig>()
        .init_resource::<SatelliteSelection>()
        .init_resource::<systems::render_mode::RenderMode>()
        .init_resource::<systems::gpu_physics::GpuPhysicsState>()
        .init_resource::<components::trails::TrailConfig>()
        .init_resource::<systems::ui::FpsHistory>()
        .init_resource::<SimulationRecorder>()
        .init_resource::<utils::integrators::IntegratorConfig>()
        .insert_resource(systems::profiling::SystemProfiler::new(100))
        .add_plugins(systems::gpu_instancing::GpuInstancingPlugin)
        .add_plugins(systems::gpu_instancing_render::InstancedRenderPlugin)
        .add_plugins(systems::gpu_physics::GpuPhysicsPlugin)
        .insert_resource(AmbientLight {
            color: Color::srgb(0.8, 0.9, 1.0),
            brightness: 0.15,
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, (
            setup_scene,
            initialize_tle_data_system,
            systems::materials::setup_materials_cache,
            // systems::ui::setup_ui_system, // UI temporarily disabled - Bevy 0.16.1 API issues
            systems::recording::setup_recording_directory,
        ))
        .add_systems(Update, (
            camera_control_system,
            time_control_system,
            physics_system,
            systems::render_mode::update_render_mode,
        ))
        .add_systems(Update, (
            systems::gpu_physics::gpu_physics_toggle_system,
            prepare_optimized_physics_system,
            optimized_physics_system,
            apply_optimized_physics_system,
            optimized_physics_monitor_system,
            systems::gpu_physics::gpu_physics_readback_system,
        ))
        .add_systems(Update, (
            update_spatial_octree_system,
            collision_detection_system,
            debris_generation_system,
        ))
        .add_systems(Update, (
            satellite_rendering_system,
            debris_rendering_system,
            collision_flash_rendering_system,
            cleanup_expired_flash_system,
            update_debris_effects_system,
            update_positions_system,
            energy_analytics_system,
        ))
        .add_systems(Update, (
            debug_orbital_system,
            debug_analytics_system,
            process_tle_fetch_system,
        ))
        .add_systems(Update, (
            stress_test_spawn_system,
            stress_test_cleanup_system,
            systems::stress_test::auto_stop_on_low_fps_system,
            performance_comparison_system,
        ))
        .add_systems(Update, (
            random_debris_injection_system,
            orbital_decay_system,
        ))
        .add_systems(Update, (
            satellite_selection_system,
            satellite_info_display_system,
            systems::hud::hud_log_system,
        ))
        .add_systems(Update, (
            systems::trail_rendering::update_trail_system,
            systems::trail_rendering::render_trails_system,
            systems::trail_rendering::toggle_trails_system,
            systems::trail_rendering::adjust_trail_length_system,
        ))
        .add_systems(Update, (
            systems::ui::update_fps_history_system,
            systems::ui::update_ui_stats_system,
        ))
        .add_systems(Update, (
            systems::recording::record_simulation_system,
        ))
        .add_systems(Update, (
            systems::particles::spawn_collision_particles_system,
            systems::particles::update_particles_system,
        ))
        // Collision prediction temporarily disabled - causes crashes
        // .add_systems(Update, (
        //     systems::collision_prediction::predict_collisions_system,
        //     systems::collision_prediction::visualize_collision_warnings_system,
        // ))
        .add_systems(Update, (
            systems::profiling::profile_frame_system,
            systems::profiling::log_performance_stats_system,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let earth_texture = asset_server.load("textures/gebco_08_rev_bath_3600x1800_color.jpg");
    
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(earth_texture.clone()),
            base_color: Color::srgb(1.0, 1.0, 1.0),
            unlit: false,
            ..default()
        })),
        Mesh3d(meshes.add(Sphere::new(6.371).mesh().uv(32, 18))),
        Transform::default(),
    ));
    
    commands.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.5, 1.0),
            unlit: true,
            ..default()
        })),
        Mesh3d(meshes.add(Sphere::new(5.0).mesh().uv(16, 8))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, 0.8, 0.0)),
    ));

    commands.spawn((
        PointLight {
            intensity: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
