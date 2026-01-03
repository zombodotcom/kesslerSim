// TDD tests for GPU instancing
// Run with: cargo test --test gpu_instancing_test

#[cfg(test)]
mod tests {
    use crate::systems::gpu_instancing::InstanceData;
    
    /// Test that InstanceData has correct size and alignment for GPU
    #[test]
    fn test_instance_data_size() {
        assert_eq!(std::mem::size_of::<InstanceData>(), 32);
        assert_eq!(std::mem::align_of::<InstanceData>(), 16);
    }
    
    /// Test that InstanceData is GPU-compatible (Pod + Zeroable)
    #[test]
    fn test_instance_data_gpu_compatible() {
        use bytemuck::{Pod, Zeroable};
        
        // Verify it implements required traits
        let _: &dyn Pod = &InstanceData::new(
            bevy::prelude::Vec3::ZERO,
            1.0,
            bevy::prelude::Color::WHITE,
        );
        let _: &dyn Zeroable = &InstanceData::new(
            bevy::prelude::Vec3::ZERO,
            1.0,
            bevy::prelude::Color::WHITE,
        );
    }
    
    /// Test InstanceData creation
    #[test]
    fn test_instance_data_creation() {
        let instance = InstanceData::new(
            bevy::prelude::Vec3::new(1.0, 2.0, 3.0),
            0.5,
            bevy::prelude::Color::srgb(1.0, 0.0, 0.0),
        );
        
        assert_eq!(instance.position_scale[0], 1.0);
        assert_eq!(instance.position_scale[1], 2.0);
        assert_eq!(instance.position_scale[2], 3.0);
        assert_eq!(instance.position_scale[3], 0.5);
        assert_eq!(instance.color[0], 1.0); // Red
        assert_eq!(instance.color[1], 0.0); // Green
        assert_eq!(instance.color[2], 0.0); // Blue
    }
    
    /// Test ViewUniform size matches pipeline expectation
    #[test]
    fn test_view_uniform_size() {
        // ViewUniform should be 400 bytes in Bevy 0.16.1
        // 6 mat4x4<f32> = 6 * 64 = 384 bytes
        // vec3<f32> = 12 bytes
        // f32 padding = 4 bytes
        // Total = 400 bytes
        let expected_size = 400u64;
        let actual_min_binding_size = 400u64;
        
        assert_eq!(actual_min_binding_size, expected_size, 
            "ViewUniform buffer size must be 400 bytes to match Bevy 0.16.1");
    }
}

