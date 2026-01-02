use bevy::prelude::*;

/// Component for active satellites
#[derive(Component)]
pub struct Satellite {
    pub name: String,
    pub norad_id: u32,
    pub active: bool,
}

impl Satellite {
    pub fn new(name: String, norad_id: u32, active: bool) -> Self {
        Self {
            name,
            norad_id,
            active,
        }
    }
}

/// Component for debris objects
#[derive(Component, Clone)]
pub struct Debris {
    /// ID of the collision event that created this debris
    pub parent_collision: Option<u32>,
    /// Generation number (0 = original object, 1 = first-gen debris, etc.)
    pub generation: u32,
    /// When this debris was created (simulation time)
    pub creation_time: f64,
}

impl Debris {
    pub fn new(parent_collision: Option<u32>, generation: u32, creation_time: f64) -> Self {
        Self {
            parent_collision,
            generation,
            creation_time,
        }
    }

    /// Create debris from an original satellite collision
    pub fn from_collision(collision_id: u32, creation_time: f64) -> Self {
        Self::new(Some(collision_id), 1, creation_time)
    }

    /// Create higher generation debris from existing debris
    pub fn from_debris(parent: &Debris, collision_id: u32, creation_time: f64) -> Self {
        Self::new(Some(collision_id), parent.generation + 1, creation_time)
    }
}

/// Marker component for objects that should be rendered as satellites
#[derive(Component)]
pub struct RenderAsSatellite;

/// Marker component for objects that should be rendered as debris
#[derive(Component)]
pub struct RenderAsDebris;

/// Component for collision flash effects
#[derive(Component)]
pub struct CollisionFlash {
    /// Time when the collision occurred
    pub start_time: f64,
    /// Duration of the flash effect in seconds
    pub duration: f64,
    /// Position of the collision
    pub position: Vec3,
    /// Maximum intensity of the flash
    pub max_intensity: f32,
}

impl CollisionFlash {
    pub fn new(position: Vec3, current_time: f64) -> Self {
        Self {
            start_time: current_time,
            duration: 0.5, // Flash lasts only 0.5 seconds (much faster)
            position,
            max_intensity: 5.0, // Very bright flash
        }
    }

    /// Get current intensity based on elapsed time (0.0 to max_intensity)
    pub fn current_intensity(&self, current_time: f64) -> f32 {
        let elapsed = current_time - self.start_time;
        if elapsed >= self.duration {
            return 0.0;
        }

        // Flash intensity: quick bright flash that fades exponentially
        let progress = (elapsed / self.duration) as f32;
        let intensity = if progress < 0.1 {
            // Initial bright flash
            self.max_intensity * (1.0 - progress * 5.0).max(0.0)
        } else {
            // Exponential fade
            self.max_intensity * 0.5 * (-5.0 * (progress - 0.1)).exp()
        };

        intensity.max(0.0)
    }

    /// Check if the flash effect is still active
    pub fn is_active(&self, current_time: f64) -> bool {
        current_time - self.start_time < self.duration
    }
}

/// Enhanced debris component with visual properties
#[derive(Component)]
pub struct EnhancedDebris {
    /// Base debris data
    pub debris: Debris,
    /// Size multiplier for rendering (1.0 = normal size)
    pub size_multiplier: f32,
    /// Glow intensity (0.0 = no glow, 1.0 = maximum glow)
    pub glow_intensity: f32,
    /// Color tint for the debris
    pub color_tint: Color,
    /// Time since creation for animation effects
    pub age: f64,
}

impl EnhancedDebris {
    pub fn from_collision(collision_id: u32, creation_time: f64, collision_energy: f32) -> Self {
        use rand::prelude::*;
        let mut rng = thread_rng();

        // Size varies based on collision energy and randomness
        let energy_factor = (collision_energy / 1e12).sqrt().clamp(0.5, 3.0);
        let size_multiplier = rng.gen_range(0.5..2.0) * energy_factor;

        // Higher energy collisions create more glowing debris
        let glow_intensity = (collision_energy / 1e13).sqrt().clamp(0.3, 1.0);

        // Color varies from orange to red based on energy
        let red_intensity = 1.0;
        let green_intensity = (1.0 - collision_energy / 1e14).clamp(0.2, 0.8);
        let color_tint = Color::srgb(red_intensity, green_intensity, 0.0);

        Self {
            debris: Debris::from_collision(collision_id, creation_time),
            size_multiplier,
            glow_intensity,
            color_tint,
            age: 0.0,
        }
    }

    /// Update age and get current visual properties
    pub fn update_age(&mut self, current_time: f64) {
        self.age = current_time - self.debris.creation_time;

        // Glow fades over time
        let fade_factor = (-self.age as f32 * 0.1).exp(); // Exponential fade over ~10 seconds
        self.glow_intensity *= fade_factor.max(0.1); // Minimum 10% glow
    }
}

/// Marker component for entities scheduled for deletion
#[derive(Component)]
pub struct ScheduledForDeletion;

/// Previous position for incremental octree updates
#[derive(Component)]
pub struct PreviousPosition(pub Vec3);
