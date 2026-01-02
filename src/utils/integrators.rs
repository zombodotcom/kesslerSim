use bevy::prelude::*;

/// Runge-Kutta 4th order integrator for more accurate physics
pub struct RK4Integrator;

impl RK4Integrator {
    /// Integrate orbital state using RK4 method
    /// Returns new position and velocity
    pub fn integrate(
        position: Vec3,
        velocity: Vec3,
        dt: f64,
        gm: f64,
    ) -> (Vec3, Vec3) {
        // Helper function to compute acceleration
        let accel = |pos: Vec3| -> Vec3 {
            let r_mag_km = pos.length() as f64;
            if r_mag_km <= 0.0 {
                return Vec3::ZERO;
            }
            
            let r_mag_m = r_mag_km * 1000.0;
            let acc_magnitude = -gm / (r_mag_m * r_mag_m);
            let r_unit = pos / r_mag_km as f32;
            let acc_km_s2 = (acc_magnitude / 1000.0) as f32;
            r_unit * acc_km_s2
        };

        // k1: derivative at current state
        let k1_v = accel(position);
        let k1_p = velocity;

        // k2: derivative at midpoint using k1
        let k2_v = accel(position + k1_p * (dt as f32 * 0.5));
        let k2_p = velocity + k1_v * (dt as f32 * 0.5);

        // k3: derivative at midpoint using k2
        let k3_v = accel(position + k2_p * (dt as f32 * 0.5));
        let k3_p = velocity + k2_v * (dt as f32 * 0.5);

        // k4: derivative at endpoint using k3
        let k4_v = accel(position + k3_p * dt as f32);
        let k4_p = velocity + k3_v * dt as f32;

        // Combine weighted average
        let new_velocity = velocity + (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v) * (dt as f32 / 6.0);
        let new_position = position + (k1_p + 2.0 * k2_p + 2.0 * k3_p + k4_p) * (dt as f32 / 6.0);

        (new_position, new_velocity)
    }
}

/// Euler integrator (original, less accurate but faster)
pub struct EulerIntegrator;

impl EulerIntegrator {
    pub fn integrate(
        position: Vec3,
        velocity: Vec3,
        dt: f64,
        gm: f64,
    ) -> (Vec3, Vec3) {
        let r_mag_km = position.length() as f64;
        if r_mag_km <= 0.0 {
            return (position, velocity);
        }

        let r_mag_m = r_mag_km * 1000.0;
        let acc_magnitude = -gm / (r_mag_m * r_mag_m);
        let r_unit = position / r_mag_km as f32;
        let acc_km_s2 = (acc_magnitude / 1000.0) as f32;
        let acc = r_unit * acc_km_s2;

        let new_velocity = velocity + acc * dt as f32;
        let new_position = position + new_velocity * dt as f32;

        (new_position, new_velocity)
    }
}

/// Resource to configure which integrator to use
#[derive(Resource)]
pub struct IntegratorConfig {
    pub use_rk4: bool,
    pub adaptive_timestep: bool,
    pub min_timestep: f64,
    pub max_timestep: f64,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            use_rk4: false, // Default to Euler for compatibility
            adaptive_timestep: false,
            min_timestep: 0.01,
            max_timestep: 1.0,
        }
    }
}

