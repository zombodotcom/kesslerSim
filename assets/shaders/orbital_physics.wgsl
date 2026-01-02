// GPU Compute Shader for Orbital Physics
// Computes gravitational acceleration and integrates position/velocity

// Orbital state structure (matches GpuOrbitalState)
struct OrbitalState {
    position: vec4<f32>,  // x, y, z, mass
    velocity: vec4<f32>, // vx, vy, vz, padding
}

// Physics parameters (matches GpuPhysicsParams)
struct PhysicsParams {
    gm: f32,           // Gravitational parameter (m³/s²)
    dt: f32,           // Time step (s)
    object_count: u32,  // Number of objects
    _padding: u32,
}

// Storage buffer for orbital states (read/write)
@group(0) @binding(0) var<storage, read_write> orbital_states: array<OrbitalState>;

// Uniform buffer for physics parameters (read-only)
@group(0) @binding(1) var<uniform> params: PhysicsParams;

// Main compute shader entry point
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    // Bounds check
    if index >= params.object_count {
        return;
    }
    
    // Load current state
    var state = orbital_states[index];
    
    // Skip invalid objects (zero mass)
    if state.position.w <= 0.0 {
        return;
    }
    
    // Extract position and velocity
    let pos = state.position.xyz;
    let vel = state.velocity.xyz;
    
    // Calculate distance from Earth center (in km)
    let r_mag_km = length(pos);
    
    if r_mag_km <= 0.0 {
        return;
    }
    
    // Convert to meters for gravitational calculation
    let r_mag_m = r_mag_km * 1000.0;
    
    // Calculate gravitational acceleration: a = -GM/r² * r̂
    // Note: Using r² to match original physics implementation
    let acc_magnitude = -params.gm / (r_mag_m * r_mag_m);
    
    // Unit vector in direction of position
    let r_unit = pos / r_mag_km;
    
    // Acceleration in km/s²
    let acc_km_s2 = acc_magnitude / 1000.0;
    let acc = r_unit * acc_km_s2;
    
    // Euler integration
    let new_vel = vel + acc * params.dt;
    let new_pos = pos + new_vel * params.dt;
    
    // Store updated state
    state.position = vec4<f32>(new_pos, state.position.w); // Preserve mass
    state.velocity = vec4<f32>(new_vel, 0.0);
    
    orbital_states[index] = state;
}

