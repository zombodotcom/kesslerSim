use bevy::prelude::*;
use crate::components::*;

/// Component for particle effects
#[derive(Component)]
pub struct ParticleEffect {
    pub effect_type: ParticleType,
    pub start_time: f64,
    pub duration: f64,
    pub position: Vec3,
    pub intensity: f32,
}

#[derive(Clone, Copy)]
pub enum ParticleType {
    CollisionExplosion,
    DebrisTrail,
    ReEntryFireball,
}

/// System to spawn particle effects for collisions
pub fn spawn_collision_particles_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    collision_query: Query<&CollisionFlash, Added<CollisionFlash>>,
    time: Res<Time>,
) {
    for flash in collision_query.iter() {
        // Spawn explosion particles
        let particle_count = 20;
        for i in 0..particle_count {
            let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
            let distance = 0.5;
            let offset = Vec3::new(
                angle.cos() * distance,
                angle.sin() * distance,
                (i as f32 / particle_count as f32 - 0.5) * distance,
            );

            let particle_mesh = meshes.add(Sphere::new(0.05).mesh().ico(3).unwrap());
            let particle_material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.2),
                emissive: Color::srgb(10.0, 8.0, 2.0).into(),
                unlit: true,
                alpha_mode: bevy::render::alpha::AlphaMode::Blend,
                ..default()
            });

            commands.spawn((
                Mesh3d(particle_mesh),
                MeshMaterial3d(particle_material),
                Transform::from_translation(flash.position / 1000.0 + offset),
                ParticleEffect {
                    effect_type: ParticleType::CollisionExplosion,
                    start_time: time.elapsed_secs_f64(),
                    duration: 2.0,
                    position: flash.position / 1000.0,
                    intensity: 1.0,
                },
            ));
        }
    }
}

/// System to update and cleanup particle effects
pub fn update_particles_system(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut ParticleEffect, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();

    for (entity, mut particle, mut transform, mut material_handle) in particle_query.iter_mut() {
        let elapsed = current_time - particle.start_time;
        
        if elapsed >= particle.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = elapsed / particle.duration;
        let fade = 1.0 - progress;

        // Update particle position and fade
        match particle.effect_type {
            ParticleType::CollisionExplosion => {
                // Particles expand outward and fade
                let expansion = progress * 2.0;
                let offset = (transform.translation - particle.position).normalize() * expansion as f32;
                transform.translation = particle.position + offset;
                
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    let base = material.base_color.to_srgba();
                    material.base_color = Color::srgba(base.red, base.green, base.blue, fade as f32);
                }
            }
            ParticleType::DebrisTrail => {
                // Trail particles fade over time
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    let base = material.base_color.to_srgba();
                    material.base_color = Color::srgba(base.red, base.green, base.blue, (fade * 0.5) as f32);
                }
            }
            ParticleType::ReEntryFireball => {
                // Fireball particles glow and fade
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = Color::srgba(10.0 * fade as f32, 2.0 * fade as f32, 0.0, 1.0).into();
                    let base = material.base_color.to_srgba();
                    material.base_color = Color::srgba(base.red, base.green, base.blue, fade as f32);
                }
            }
        }
    }
}

