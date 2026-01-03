// Simple point rendering shader for millions of objects
// Reads positions from storage buffer, renders as points

// Camera uniform (simple - just view-projection matrix)
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Position storage buffer (from compute shader)
@group(0) @binding(1) var<storage, read> positions: array<vec4<f32>>; // x, y, z, scale

// Color storage buffer (optional - can use uniform color)
@group(0) @binding(2) var<storage, read> colors: array<vec4<f32>>; // r, g, b, a

// Simple vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Get position from storage buffer
    let pos_data = positions[vertex_index];
    let world_pos = pos_data.xyz;
    let scale = pos_data.w;
    
    // Transform to clip space
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    
    // Get color from buffer (or use default)
    out.color = colors[vertex_index];
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

