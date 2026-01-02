use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    renderer::RenderDevice,
    Render, RenderApp, RenderSet,
};
use crate::components::*;
use crate::systems::render_mode::RenderMode;

/// Instance data for GPU instanced rendering
/// Packed into 16 bytes for efficient GPU memory usage
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Position (x, y, z) and scale
    pub position_scale: [f32; 4],
    /// Color (r, g, b, a)
    pub color: [f32; 4],
}

impl InstanceData {
    pub fn new(position: Vec3, scale: f32, color: Color) -> Self {
        Self {
            position_scale: [position.x, position.y, position.z, scale],
            color: {
                let srgba = color.to_srgba();
                [srgba.red, srgba.green, srgba.blue, srgba.alpha]
            },
        }
    }
}

/// Resource to sync instance data from main app to render app
#[derive(Resource, Default)]
pub struct InstanceDataSync {
    pub satellite_instances: Vec<InstanceData>,
    pub debris_instances: Vec<InstanceData>,
    pub needs_update: bool,
}

/// System to extract instance data from main app
/// This runs in ExtractSchedule to copy data from main app to render app
/// NOTE: Runs every frame to capture all position updates
pub fn extract_instance_data(
    mut commands: Commands,
    instance_data: Option<ResMut<InstanceDataSync>>,
    satellite_query: Query<(&OrbitalState, &RenderAsSatellite), With<Satellite>>,
    debris_query: Query<(&OrbitalState, &RenderAsDebris), With<Debris>>,
    render_mode: Option<Res<RenderMode>>,
) {
    // Initialize resource if it doesn't exist
    let mut instance_data = if let Some(data) = instance_data {
        data
    } else {
        commands.init_resource::<InstanceDataSync>();
        return; // Skip this frame, will work next frame
    };
    
    // Use default render mode if not available
    let render_fraction = render_mode.map(|r| r.render_fraction).unwrap_or(1.0);
    // Always update every frame - positions change every frame
    instance_data.needs_update = true;

    // Collect satellite instances (apply render fraction)
    instance_data.satellite_instances.clear();
    let total_satellites = satellite_query.iter().count();
    let max_render = (total_satellites as f32 * render_fraction) as usize;
    
    let mut count = 0;
    for (orbital_state, _) in satellite_query.iter() {
        if count >= max_render {
            break;
        }
        let scaled_pos = orbital_state.position / 1000.0;
        instance_data.satellite_instances.push(InstanceData::new(
            scaled_pos,
            0.05,
            Color::srgb(0.0, 1.0, 0.0), // Green for satellites
        ));
        count += 1;
    }

    // Collect debris instances (apply render fraction)
    instance_data.debris_instances.clear();
    let total_debris = debris_query.iter().count();
    let max_render_debris = (total_debris as f32 * render_fraction) as usize;
    
    let mut count = 0;
    for (orbital_state, _) in debris_query.iter() {
        if count >= max_render_debris {
            break;
        }
        let scaled_pos = orbital_state.position / 1000.0;
        instance_data.debris_instances.push(InstanceData::new(
            scaled_pos,
            0.03,
            Color::srgb(1.0, 0.0, 0.0), // Red for debris
        ));
        count += 1;
    }
}

/// System to prepare GPU instance buffers (runs in render app)
/// Updates buffers every frame with latest instance data
pub fn prepare_instance_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    instance_data: Option<Res<InstanceDataSync>>,
) {
    let Some(instance_data) = instance_data else { return };

    // Update buffers for satellite instances - recreate every frame with fresh data
    if !instance_data.satellite_instances.is_empty() {
        let count = instance_data.satellite_instances.len();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("satellite_instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data.satellite_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });
        commands.insert_resource(SatelliteInstanceBuffer(buffer));
        commands.insert_resource(SatelliteInstanceCount(count as u32));
    } else {
        commands.remove_resource::<SatelliteInstanceBuffer>();
        commands.remove_resource::<SatelliteInstanceCount>();
    }

    // Update buffers for debris instances - recreate every frame with fresh data
    if !instance_data.debris_instances.is_empty() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("debris_instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data.debris_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });
        commands.insert_resource(DebrisInstanceBuffer(buffer));
        commands.insert_resource(DebrisInstanceCount(instance_data.debris_instances.len() as u32));
    } else {
        commands.remove_resource::<DebrisInstanceBuffer>();
        commands.remove_resource::<DebrisInstanceCount>();
    }
}

/// Resource to store satellite instance buffer
#[derive(Resource)]
pub struct SatelliteInstanceBuffer(pub Buffer);

/// Resource to store debris instance buffer
#[derive(Resource)]
pub struct DebrisInstanceBuffer(pub Buffer);

/// Resource to store satellite instance count
#[derive(Resource, Default)]
pub struct SatelliteInstanceCount(pub u32);

/// Resource to store debris instance count
#[derive(Resource, Default)]
pub struct DebrisInstanceCount(pub u32);

/// Resource to store base mesh for satellite instancing
#[derive(Resource)]
pub struct SatelliteBaseMesh(pub Handle<Mesh>);

/// Resource to store base mesh for debris instancing
#[derive(Resource)]
pub struct DebrisBaseMesh(pub Handle<Mesh>);

/// System to queue instanced rendering
/// This prepares the render pipeline for drawing instances
pub fn queue_instanced_rendering(
    satellite_buffer: Option<Res<SatelliteInstanceBuffer>>,
    debris_buffer: Option<Res<DebrisInstanceBuffer>>,
    satellite_count: Option<Res<SatelliteInstanceCount>>,
    debris_count: Option<Res<DebrisInstanceCount>>,
) {
    // GPU buffers are ready - custom render pipeline would use these
    // For now, this is a placeholder until the full render pipeline is implemented
    if let Some(count) = satellite_count {
        if count.0 > 0 {
            // Buffers ready for rendering
        }
    }
    
    if let Some(count) = debris_count {
        if count.0 > 0 {
            // Buffers ready for rendering
        }
    }
}

/// Plugin for GPU instancing
pub struct GpuInstancingPlugin;

/// System to create base meshes for instancing
pub fn create_base_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    use bevy::math::primitives::Sphere;
    // Create base mesh for satellites (small sphere, ico(5))
    let satellite_mesh = meshes.add(Sphere::new(0.05).mesh().ico(5).unwrap());
    commands.insert_resource(SatelliteBaseMesh(satellite_mesh));
    
    // Create base mesh for debris (medium sphere, ico(4))
    // Note: Size will be scaled per-instance, so we use a base size
    let debris_mesh = meshes.add(Sphere::new(0.3).mesh().ico(4).unwrap());
    commands.insert_resource(DebrisBaseMesh(debris_mesh));
}

impl Plugin for GpuInstancingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize in main app
        app.init_resource::<InstanceDataSync>();
        // Initialize count resources in main app so rendering systems can see them
        app.init_resource::<SatelliteInstanceCount>();
        app.init_resource::<DebrisInstanceCount>();
        
        // Create base meshes on startup
        app.add_systems(Startup, create_base_meshes);
        
        // Also initialize in render app
        app.sub_app_mut(RenderApp)
            .init_resource::<InstanceDataSync>()
            .init_resource::<RenderMode>();
        
        // Extract instance data from main app to render app
        app.sub_app_mut(RenderApp)
            .add_systems(bevy::render::ExtractSchedule, extract_instance_data);
        
        // Prepare buffers in render app
        app.sub_app_mut(RenderApp)
            .add_systems(Render, prepare_instance_buffers.in_set(bevy::render::RenderSet::Prepare));
        
        // Queue instanced rendering
        app.sub_app_mut(RenderApp)
            .add_systems(Render, queue_instanced_rendering.in_set(bevy::render::RenderSet::Queue));
        
        // System to sync instance counts from render app to main app
        // This allows rendering systems in main app to know GPU instancing is active
        app.add_systems(Update, sync_instance_counts_to_main_app);
    }
}

/// System to sync instance counts from render app to main app
/// This allows rendering systems in the main app to detect GPU instancing
fn sync_instance_counts_to_main_app(
    mut commands: Commands,
    instance_data: Option<Res<InstanceDataSync>>,
) {
    if let Some(data) = instance_data {
        // Update counts in main app based on instance data
        commands.insert_resource(SatelliteInstanceCount(data.satellite_instances.len() as u32));
        commands.insert_resource(DebrisInstanceCount(data.debris_instances.len() as u32));
    }
}
