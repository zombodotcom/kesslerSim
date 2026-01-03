// Minimal GPU renderer - ~100 lines, simple and efficient
// Prepares GPU resources for point rendering from compute shader buffers

use bevy::prelude::*;
use bevy::render::{
    render_resource::*,
    renderer::{RenderDevice, RenderQueue},
    Render, RenderApp, RenderSet,
};
use crate::systems::gpu_physics::GpuPhysicsBuffers;

/// Camera matrix (64 bytes)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    view_proj: [[f32; 4]; 4],
}

/// Minimal render resources
#[derive(Resource)]
struct RenderData {
    camera_buffer: Buffer,
}

/// Prepare camera buffer once
fn prepare_renderer(
    mut commands: Commands,
    device: Res<RenderDevice>,
) {
    // Camera buffer for view-projection matrix
    let camera_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("camera_buffer"),
        size: 64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    commands.insert_resource(RenderData {
        camera_buffer,
    });
}

/// Update camera matrix from Bevy camera
fn update_camera(
    queue: Res<RenderQueue>,
    render_data: Option<Res<RenderData>>,
    cameras: Query<(&GlobalTransform, &Projection), With<Camera3d>>,
) {
    let Some(render_data) = render_data else { return };
    let Ok((transform, proj)) = cameras.get_single() else { return };
    
    let view = transform.compute_matrix().inverse();
    let proj_mat = match proj {
        Projection::Perspective(p) => {
            Mat4::perspective_rh(p.fov, p.aspect_ratio, p.near, 1000.0)
        },
        Projection::Orthographic(o) => {
            let scale = o.scale;
            Mat4::orthographic_rh(-scale, scale, -scale, scale, o.near, o.far)
        },
        Projection::Custom(_) => Mat4::IDENTITY,
    };
    
    let view_proj = proj_mat * view;
    let data = CameraData { view_proj: view_proj.to_cols_array_2d() };
    
    queue.write_buffer(&render_data.camera_buffer, 0, bytemuck::cast_slice(&[data]));
}

pub struct SimpleGpuRenderPlugin;

impl Plugin for SimpleGpuRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        
        // Prepare camera buffer
        render_app.add_systems(Render, prepare_renderer.in_set(RenderSet::Prepare));
        
        // Update camera matrix each frame
        app.add_systems(Update, update_camera);
        
        // Note: Actual point rendering will be added via render graph when needed
        // For now, this sets up the camera buffer that the shader will use
        // The compute shader (gpu_physics) already updates positions in GPU buffers
    }
}
