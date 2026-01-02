use bevy::prelude::*;
use rand::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Resource to control random debris injection
#[derive(Resource)]
pub struct DebrisInjectionConfig {
    pub enabled: bool,
    pub frequency_seconds: f64,
    pub percentage: f64, // Percentage of existing debris to add
    pub last_injection: f64,
}

impl Default for DebrisInjectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency_seconds: 3600.0, // Inject every hour (simulation time)
            percentage: 1.0, // Add 1% of existing debris count
            last_injection: 0.0,
        }
    }
}

/// Resource to control orbital decay (debris falldown)
#[derive(Resource)]
pub struct OrbitalDecayConfig {
    pub enabled: bool,
    pub decay_rate: f64, // Percentage of debris to remove per decay event
    pub frequency_seconds: f64,
    pub last_decay: f64,
    pub min_altitude_km: f64, // Minimum altitude for decay (below this, debris falls)
}

impl Default for OrbitalDecayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decay_rate: 0.1, // Remove 10% of debris per decay event
            frequency_seconds: 3600.0, // Check every hour
            last_decay: 0.0,
            min_altitude_km: 200.0, // Debris below 200km decays
        }
    }
}

/// System to inject random debris periodically (simulating ongoing launches/background noise)
pub fn random_debris_injection_system(
    mut commands: Commands,
    mut config: ResMut<DebrisInjectionConfig>,
    sim_time: Res<SimulationTime>,
    debris_query: Query<&Debris>,
    orbital_query: Query<&OrbitalState>,
    constants: Res<Constants>,
) {
    if !config.enabled || sim_time.paused {
        return;
    }

    let elapsed = sim_time.current - config.last_injection;
    if elapsed < config.frequency_seconds {
        return;
    }

    config.last_injection = sim_time.current;

    // Calculate number of new debris to inject
    let existing_debris_count = debris_query.iter().count();
    let new_debris_count = ((existing_debris_count as f64 * config.percentage / 100.0).ceil() as usize).max(1);

    if new_debris_count == 0 {
        return;
    }

    // Get random orbital parameters from existing objects
    let mut rng = thread_rng();
    let orbital_states: Vec<_> = orbital_query.iter().collect();
    
    if orbital_states.is_empty() {
        return;
    }

    // Spawn new random debris
    for _ in 0..new_debris_count {
        // Pick a random existing object to base parameters on
        let base_state = orbital_states[rng.gen_range(0..orbital_states.len())];
        
        // Randomize parameters slightly
        let altitude_variation = rng.gen_range(-100.0..100.0);
        let new_altitude = base_state.altitude() + altitude_variation;
        
        // Ensure altitude is reasonable
        if new_altitude < constants.earth_radius + 160.0 {
            continue; // Skip if too low
        }

        let new_radius = new_altitude;
        let orbital_speed = constants.circular_velocity(new_altitude - constants.earth_radius);
        
        // Random position on orbit
        let theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let phi = rng.gen_range(0.0..std::f64::consts::PI);
        
        let position = Vec3::new(
            (new_radius * phi.sin() * theta.cos()) as f32,
            (new_radius * phi.sin() * theta.sin()) as f32,
            (new_radius * phi.cos()) as f32,
        );
        
        // Velocity perpendicular to position
        let velocity = Vec3::new(
            (-orbital_speed * theta.sin()) as f32,
            (orbital_speed * theta.cos()) as f32,
            (rng.gen_range(-0.1..0.1) * orbital_speed) as f32,
        );

        // Small debris mass
        let debris_mass = rng.gen_range(10.0..100.0);

        commands.spawn((
            OrbitalState::new(position, velocity, debris_mass),
            PhysicsObject::debris(debris_mass),
            Debris::new(None, 0, sim_time.current),
            EnhancedDebris {
                debris: Debris::new(None, 0, sim_time.current),
                size_multiplier: rng.gen_range(0.3..1.0),
                glow_intensity: 0.3,
                color_tint: Color::srgb(1.0, 0.5, 0.0),
                age: 0.0,
            },
            RenderAsDebris,
        ));
    }

    info!("Injected {} random debris pieces", new_debris_count);
}

/// System to simulate orbital decay (debris falldown)
pub fn orbital_decay_system(
    mut commands: Commands,
    mut config: ResMut<OrbitalDecayConfig>,
    sim_time: Res<SimulationTime>,
    mut debris_query: Query<(Entity, &mut OrbitalState, &Debris)>,
    constants: Res<Constants>,
) {
    if !config.enabled || sim_time.paused {
        return;
    }

    let elapsed = sim_time.current - config.last_decay;
    if elapsed < config.frequency_seconds {
        return;
    }

    config.last_decay = sim_time.current;

    // Find debris below minimum altitude
    let mut decayed_count = 0;
    let mut to_remove = Vec::new();

    for (entity, orbital_state, _debris) in debris_query.iter() {
        let altitude = orbital_state.altitude() - constants.earth_radius;
        
            if altitude < config.min_altitude_km {
            // Random chance for decay based on altitude (lower = higher chance)
            let decay_probability = 1.0 - (altitude / config.min_altitude_km).clamp(0.0, 1.0);
            
            let mut rng = thread_rng();
            if rng.gen::<f64>() < decay_probability {
                to_remove.push(entity);
                decayed_count += 1;
            }
        }
    }

    // Remove decayed debris
    for entity in to_remove {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    if decayed_count > 0 {
        info!("Orbital decay: {} debris pieces fell to Earth", decayed_count);
    }
}

