use bevy::prelude::*;

/// Component for orbital trails - stores position history for visualization
#[derive(Component)]
pub struct Trail {
    /// Circular buffer of positions (in render units, km/1000)
    pub positions: Vec<Vec3>,
    /// Maximum number of trail points
    pub max_length: usize,
    /// Color of the trail
    pub color: Color,
    /// Altitude band for color coding
    pub altitude_band: AltitudeBand,
}

/// Altitude bands for color coding trails
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AltitudeBand {
    LEO,  // Low Earth Orbit: 200-2000 km (Blue)
    MEO,  // Medium Earth Orbit: 2000-35786 km (Green)
    GEO,  // Geostationary Orbit: 35786+ km (Red)
}

impl AltitudeBand {
    pub fn from_altitude(altitude_km: f64) -> Self {
        if altitude_km < 2000.0 {
            AltitudeBand::LEO
        } else if altitude_km < 35786.0 {
            AltitudeBand::MEO
        } else {
            AltitudeBand::GEO
        }
    }

    pub fn color(&self) -> Color {
        match self {
            AltitudeBand::LEO => Color::srgb(0.2, 0.5, 1.0),  // Blue
            AltitudeBand::MEO => Color::srgb(0.2, 1.0, 0.3),  // Green
            AltitudeBand::GEO => Color::srgb(1.0, 0.2, 0.2), // Red
        }
    }
}

impl Trail {
    pub fn new(max_length: usize, altitude_km: f64) -> Self {
        let altitude_band = AltitudeBand::from_altitude(altitude_km);
        Self {
            positions: Vec::with_capacity(max_length),
            max_length,
            color: altitude_band.color(),
            altitude_band,
        }
    }

    /// Add a new position to the trail
    pub fn add_position(&mut self, position: Vec3) {
        self.positions.push(position);
        
        // Maintain circular buffer
        if self.positions.len() > self.max_length {
            self.positions.remove(0);
        }
    }

    /// Update altitude band and color if altitude changed significantly
    pub fn update_altitude_band(&mut self, altitude_km: f64) {
        let new_band = AltitudeBand::from_altitude(altitude_km);
        if new_band != self.altitude_band {
            self.altitude_band = new_band;
            self.color = new_band.color();
        }
    }

    /// Get trail points with fade-out alpha based on age
    pub fn get_points_with_alpha(&self) -> Vec<(Vec3, f32)> {
        let total_points = self.positions.len();
        if total_points == 0 {
            return Vec::new();
        }

        self.positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                // Fade from 1.0 (newest) to 0.1 (oldest)
                let age_factor = i as f32 / total_points.max(1) as f32;
                let alpha = 1.0 - (age_factor * 0.9); // Fade from 1.0 to 0.1
                (pos, alpha)
            })
            .collect()
    }
}

/// Resource to control trail rendering
#[derive(Resource)]
pub struct TrailConfig {
    pub enabled: bool,
    pub default_length: usize,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for performance
            default_length: 100, // Reduced length for performance
        }
    }
}

