// Custom instanced rendering shader for Bevy 0.16.1
// Uses manual view uniform binding since mesh_view_bindings structure may differ

// View uniform structure (matches Bevy's ViewUniform - 400 bytes total)
// Each mat4x4<f32> is 64 bytes (16 floats * 4 bytes)
// vec3<f32> is 12 bytes, f32 is 4 bytes
// Total: 64*6 + 12 + 4 = 400 bytes
struct ViewUniform {
    view_proj: mat4x4<f32>,           // 64 bytes
    inverse_view_proj: mat4x4<f32>,   // 64 bytes
    view: mat4x4<f32>,                // 64 bytes
    inverse_view: mat4x4<f32>,         // 64 bytes
    projection: mat4x4<f32>,          // 64 bytes
    inverse_projection: mat4x4<f32>,  // 64 bytes
    world_position: vec3<f32>,         // 12 bytes
    _padding: f32,                    // 4 bytes
    // Total: 400 bytes
}

// Bind view uniform at group 0, binding 0 (matches our pipeline setup)
@group(0) @binding(0) var<uniform> view_uniform: ViewUniform;

// Simple vertex output structure
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// Instance data structure matching Rust InstanceData
struct InstanceData {
    position_scale: vec4<f32>,
    color: vec4<f32>,
}

@vertex
fn vertex(
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) instance_pos_scale: vec4<f32>,
    @location(6) instance_color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Extract position and scale from instance data
    let instance_pos = instance_pos_scale.xyz;
    let instance_scale = instance_pos_scale.w;
    
    // Transform vertex position by instance transform
    let world_position = position * instance_scale + instance_pos;
    
    out.world_position = world_position;
    // Use manually bound view uniform
    out.position = view_uniform.view_proj * vec4<f32>(world_position, 1.0);
    out.color = instance_color;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
