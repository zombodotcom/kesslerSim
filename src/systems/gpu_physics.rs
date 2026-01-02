use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    renderer::RenderDevice,
    Render, RenderApp, RenderSet,
};
use crate::components::*;
use crate::resources::*;

/// GPU-compatible orbital state data structure
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuOrbitalState {
    /// Position vector in kilometers (x, y, z, mass)
    pub position: [f32; 4],
    /// Velocity vector in km/s (x, y, z, padding)
    pub velocity: [f32; 4],
}

impl GpuOrbitalState {
    pub fn from_orbital_state(state: &OrbitalState) -> Self {
        Self {
            position: [state.position.x, state.position.y, state.position.z, state.mass as f32],
            velocity: [state.velocity.x, state.velocity.y, state.velocity.z, 0.0],
        }
    }

    pub fn to_orbital_state(&self) -> OrbitalState {
        OrbitalState::new(
            Vec3::new(self.position[0], self.position[1], self.position[2]),
            Vec3::new(self.velocity[0], self.velocity[1], self.velocity[2]),
            self.position[3] as f64,
        )
    }
}

/// Physics parameters for GPU compute shader
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPhysicsParams {
    /// Gravitational parameter (GM) in m³/s²
    pub gm: f32,
    /// Time step in seconds
    pub dt: f32,
    /// Number of objects to process
    pub object_count: u32,
    /// Padding for alignment
    pub _padding: u32,
}

/// Resource to sync orbital state data from main app to render app
#[derive(Resource, Default)]
pub struct GpuPhysicsData {
    pub orbital_states: Vec<GpuOrbitalState>,
    pub physics_params: Option<GpuPhysicsParams>,
    pub needs_update: bool,
}

/// System to extract orbital data from main app (runs in main app, extracts to render app)
pub fn extract_gpu_physics_data(
    mut commands: Commands,
    gpu_data: Option<ResMut<GpuPhysicsData>>,
    orbital_query: Query<&OrbitalState, (With<PhysicsObject>, Changed<OrbitalState>)>,
    constants: Option<Res<Constants>>,
    sim_time: Option<Res<SimulationTime>>,
    time: Res<Time>,
    gpu_state: Option<Res<GpuPhysicsState>>,
) {
    // Initialize resource if it doesn't exist
    let mut gpu_data = if let Some(data) = gpu_data {
        data
    } else {
        commands.init_resource::<GpuPhysicsData>();
        return; // Skip this frame, will work next frame
    };
    
    // Skip if required resources don't exist
    let Some(constants) = constants else { return; };
    let Some(sim_time) = sim_time else { return; };
    let Some(gpu_state) = gpu_state else { return; };
    
    if !gpu_state.enabled || !gpu_data.needs_update {
        return;
    }

    // Collect orbital states
    gpu_data.orbital_states.clear();
    for orbital_state in orbital_query.iter() {
        gpu_data.orbital_states.push(GpuOrbitalState::from_orbital_state(orbital_state));
    }

    // Set physics parameters
    gpu_data.physics_params = Some(GpuPhysicsParams {
        gm: constants.gravitational_parameter as f32,
        dt: (time.delta_secs() as f64 * sim_time.speed_multiplier) as f32,
        object_count: gpu_data.orbital_states.len() as u32,
        _padding: 0,
    });

    gpu_data.needs_update = false;
}

/// System to prepare GPU buffers with current orbital data (runs in render app)
pub fn prepare_gpu_physics_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    gpu_data: Option<Res<GpuPhysicsData>>,
) {
    let Some(gpu_data) = gpu_data else { return };
    
    if gpu_data.orbital_states.is_empty() {
        return;
    }

    // Create buffer for orbital states
    let states_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("gpu_physics_states_buffer"),
        contents: bytemuck::cast_slice(&gpu_data.orbital_states),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    // Create buffer for physics parameters
    if let Some(params) = gpu_data.physics_params {
        let params_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("gpu_physics_params_buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        commands.insert_resource(GpuPhysicsBuffers {
            orbital_states_buffer: states_buffer,
            physics_params_buffer: params_buffer,
            object_count: gpu_data.orbital_states.len(),
        });
    }
}

/// Resource to manage GPU physics buffers
#[derive(Resource)]
pub struct GpuPhysicsBuffers {
    pub orbital_states_buffer: Buffer,
    pub physics_params_buffer: Buffer,
    pub object_count: usize,
}

/// Resource to track GPU physics state
#[derive(Resource, Default)]
pub struct GpuPhysicsState {
    pub enabled: bool,
    pub last_update_frame: u64,
}

/// System to copy GPU results back to CPU
/// This runs after GPU compute completes
pub fn gpu_physics_readback_system(
    _orbital_query: Query<(Entity, &mut OrbitalState), With<PhysicsObject>>,
    _buffers: Option<Res<GpuPhysicsBuffers>>,
    _render_device: Option<Res<RenderDevice>>,
    mut gpu_state: ResMut<GpuPhysicsState>,
) {
    // GPU readback would go here - for now, CPU physics is used
    // This is a placeholder for future GPU compute shader integration
    gpu_state.enabled = false; // Disable until compute shader is implemented
}

/// System to enable/disable GPU physics based on object count
pub fn gpu_physics_toggle_system(
    mut gpu_state: ResMut<GpuPhysicsState>,
    mut gpu_data: ResMut<GpuPhysicsData>,
    orbital_query: Query<&OrbitalState, With<PhysicsObject>>,
) {
    let object_count = orbital_query.iter().count();
    
    // Enable GPU physics for large object counts (>5000)
    // Below that, CPU physics is faster due to overhead
    let should_enable = object_count > 5000;
    
    if should_enable != gpu_state.enabled {
        gpu_state.enabled = should_enable;
        gpu_data.needs_update = true;
        
        if gpu_state.enabled {
            info!("GPU Physics: ENABLED ({} objects)", object_count);
        } else {
            info!("GPU Physics: DISABLED ({} objects - using CPU)", object_count);
        }
    }
}

/// Plugin to set up GPU physics
pub struct GpuPhysicsPlugin;

impl Plugin for GpuPhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Initialize in main app
        app.init_resource::<GpuPhysicsData>();
        app.init_resource::<GpuPhysicsState>();
        
        // Also initialize in render app (resources needed by extract system)
        app.sub_app_mut(RenderApp)
            .init_resource::<GpuPhysicsData>()
            .init_resource::<GpuPhysicsState>()
            .init_resource::<Constants>()
            .init_resource::<SimulationTime>();
        
        // Extract GPU physics data from main app to render app
        app.sub_app_mut(RenderApp)
            .add_systems(bevy::render::ExtractSchedule, extract_gpu_physics_data);
        
        // Prepare buffers in render app
        app.sub_app_mut(RenderApp)
            .add_systems(Render, prepare_gpu_physics_buffers.in_set(bevy::render::RenderSet::Prepare));
    }
}
