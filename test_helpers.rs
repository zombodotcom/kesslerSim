// Quick test helpers for faster development
// Run with: cargo test --test test_helpers

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Quick test to verify instance data structure
    #[test]
    fn test_instance_data_size() {
        use crate::systems::gpu_instancing::InstanceData;
        assert_eq!(std::mem::size_of::<InstanceData>(), 32); // 16 bytes * 2
    }
    
    /// Quick test to verify instance data alignment
    #[test]
    fn test_instance_data_alignment() {
        use crate::systems::gpu_instancing::InstanceData;
        assert_eq!(std::mem::align_of::<InstanceData>(), 16);
    }
}

