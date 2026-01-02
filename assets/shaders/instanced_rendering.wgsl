#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::{Vertex, VertexOutput},
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
    out.position = view.view_proj * vec4<f32>(world_position, 1.0);
    out.uv = uv;
    out.normal = normal;
    out.color = instance_color;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
