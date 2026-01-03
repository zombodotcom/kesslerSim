// Simplified GPU instancing - just ensures GPU physics buffers are updated
// The simple renderer reads directly from GPU physics buffers
// No complex instance data extraction needed

use bevy::prelude::*;
use crate::components::*;
use crate::systems::gpu_physics::GpuPhysicsData;

/// System to mark GPU physics data as needing update when objects change
/// This ensures the compute shader gets fresh data
pub fn mark_gpu_physics_dirty(
    mut gpu_data: ResMut<GpuPhysicsData>,
    orbital_query: Query<&OrbitalState, (With<PhysicsObject>, Changed<OrbitalState>)>,
) {
    // If any orbital states changed, mark for update
    if !orbital_query.is_empty() {
        gpu_data.needs_update = true;
    }
}

/// Legacy types kept for compatibility with rendering.rs
/// These are no longer used by the simple renderer but may be referenced elsewhere
#[derive(Resource, Default)]
pub struct SatelliteInstanceCount(pub u32);

#[derive(Resource, Default)]
pub struct DebrisInstanceCount(pub u32);

/// Plugin for simplified GPU instancing
/// Just ensures GPU physics data stays in sync
pub struct GpuInstancingPlugin;

impl Plugin for GpuInstancingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize legacy resources for compatibility
        app.init_resource::<SatelliteInstanceCount>();
        app.init_resource::<DebrisInstanceCount>();
        
        app.add_systems(
            Update,
            mark_gpu_physics_dirty,
        );
    }
}
