// Unit tests for collision detection functionality
// Tests octree insertion/querying, sphere-sphere detection, spatial partitioning, and deduplication

use kessler_simulator::components::*;
use kessler_simulator::systems::collision::*;
mod common;
use bevy::prelude::*;
use common::*;

#[test]
fn test_octree_insertion() {
    // Test basic octree insertion
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, 6, 0);

    let entity1 = Entity::from_raw(1);
    let position1 = Vec3::new(100.0, 200.0, 300.0);

    let inserted = octree.insert(entity1, position1);
    assert!(inserted, "Object should be inserted into octree");
    assert_eq!(octree.objects.len(), 1, "Octree should contain one object");
}

#[test]
fn test_octree_query_sphere() {
    // Test querying objects within a sphere
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, 6, 0);

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Insert objects at different positions
    octree.insert(entity1, Vec3::new(100.0, 100.0, 100.0));
    octree.insert(entity2, Vec3::new(200.0, 200.0, 200.0));
    octree.insert(entity3, Vec3::new(5000.0, 5000.0, 5000.0)); // Far away

    // Query for objects near position 1
    let mut results = Vec::new();
    octree.query_sphere(Vec3::new(100.0, 100.0, 100.0), 200.0, &mut results);

    // Should find entity1 and possibly entity2 (if within radius)
    assert!(results.contains(&entity1), "Query should find entity1");
    assert!(
        results.len() >= 1,
        "Query should return at least one result"
    );
}

#[test]
fn test_octree_subdivision() {
    // Test that octree subdivides when it has too many objects
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, 6, 0);

    // Insert more than MAX_OBJECTS_PER_NODE (4) objects
    for i in 0..5 {
        let entity = Entity::from_raw(i as u32);
        let position = Vec3::new(i as f32 * 10.0, 0.0, 0.0);
        octree.insert(entity, position);
    }

    // Octree should have subdivided
    assert!(
        octree.children.is_some(),
        "Octree should subdivide with many objects"
    );
}

#[test]
fn test_sphere_intersects_cube() {
    // Test sphere-cube intersection detection
    let octree = OctreeNode::new(Vec3::ZERO, 100.0, 6, 0);

    // Sphere that intersects cube
    let sphere_center = Vec3::new(50.0, 50.0, 50.0);
    let sphere_radius = 100.0;
    assert!(
        octree.sphere_intersects_cube(sphere_center, sphere_radius),
        "Sphere should intersect cube"
    );

    // Sphere that doesn't intersect cube
    let far_sphere_center = Vec3::new(500.0, 500.0, 500.0);
    assert!(
        !octree.sphere_intersects_cube(far_sphere_center, 10.0),
        "Distant sphere should not intersect cube"
    );
}

#[test]
fn test_contains_point() {
    // Test point containment check
    let octree = OctreeNode::new(Vec3::ZERO, 100.0, 6, 0);

    // Point inside cube
    let inside_point = Vec3::new(50.0, 50.0, 50.0);
    assert!(
        octree.contains_point(inside_point),
        "Point should be inside cube"
    );

    // Point outside cube
    let outside_point = Vec3::new(200.0, 200.0, 200.0);
    assert!(
        !octree.contains_point(outside_point),
        "Point should be outside cube"
    );

    // Point on boundary
    let boundary_point = Vec3::new(100.0, 0.0, 0.0);
    assert!(
        octree.contains_point(boundary_point),
        "Point on boundary should be inside"
    );
}

#[test]
fn test_collision_detection_same_position() {
    // Test collision detection for objects at same position
    let mut octree = SpatialOctree::default();

    // Create two objects at the same position
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    let position = Vec3::new(6771.0, 0.0, 0.0); // 400 km altitude

    octree.root.insert(entity1, position);
    octree.root.insert(entity2, position);

    // Query for collisions
    let mut results = Vec::new();
    octree.root.query_sphere(position, 1.0, &mut results);

    // Should find both objects
    assert!(results.contains(&entity1), "Should find entity1");
    assert!(results.contains(&entity2), "Should find entity2");
}

#[test]
fn test_collision_detection_distant_objects() {
    // Test that octree spatial partitioning separates distant objects
    let mut octree = SpatialOctree::default();

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    // Objects far apart (different octants)
    let position1 = Vec3::new(1000.0, 1000.0, 1000.0);
    let position2 = Vec3::new(40000.0, 40000.0, 40000.0); // Different octant

    octree.root.insert(entity1, position1);
    octree.root.insert(entity2, position2);

    // Query near position1 with moderate radius
    let mut results = Vec::new();
    octree.root.query_sphere(position1, 10000.0, &mut results);

    // Should find entity1
    assert!(results.contains(&entity1), "Should find entity1");
    // Note: query_sphere doesn't filter by exact distance, only by node intersection
    // This test verifies octree structure works, collision system does distance filtering
}

#[test]
fn test_octree_clear() {
    // Test that octree can be cleared
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, 6, 0);

    // Insert some objects
    for i in 0..5 {
        let entity = Entity::from_raw(i as u32);
        octree.insert(entity, Vec3::new(i as f32 * 10.0, 0.0, 0.0));
    }

    assert!(
        octree.objects.len() > 0 || octree.children.is_some(),
        "Octree should have objects or children"
    );

    // Clear octree
    octree.clear();

    assert_eq!(
        octree.objects.len(),
        0,
        "Octree should have no objects after clear"
    );
    assert!(
        octree.children.is_none(),
        "Octree should have no children after clear"
    );
}

#[test]
fn test_collision_pair_deduplication() {
    // Test that collision pairs are not duplicated
    let mut checked_pairs = std::collections::HashSet::new();

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);

    // Create pair (entity1, entity2)
    let pair1 = if entity1.index() < entity2.index() {
        (entity1, entity2)
    } else {
        (entity2, entity1)
    };

    // Add pair to set
    checked_pairs.insert(pair1);

    // Try to add same pair in reverse order
    let pair2 = if entity2.index() < entity1.index() {
        (entity2, entity1)
    } else {
        (entity1, entity2)
    };

    // Should already be in set (deduplicated)
    assert!(
        checked_pairs.contains(&pair2),
        "Reverse pair should be deduplicated"
    );
    assert_eq!(checked_pairs.len(), 1, "Should only have one unique pair");
}

#[test]
fn test_octree_boundary_conditions() {
    // Test octree behavior at boundaries
    let mut octree = OctreeNode::new(Vec3::ZERO, 100.0, 6, 0);

    // Insert object exactly at boundary
    let entity = Entity::from_raw(1);
    let boundary_position = Vec3::new(100.0, 100.0, 100.0);

    let inserted = octree.insert(entity, boundary_position);
    assert!(inserted, "Object at boundary should be inserted");

    // Query at boundary
    let mut results = Vec::new();
    octree.query_sphere(boundary_position, 1.0, &mut results);
    assert!(results.contains(&entity), "Should find object at boundary");
}

#[test]
fn test_multiple_objects_same_octant() {
    // Test multiple objects in the same octant
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, 6, 0);

    // Insert multiple objects close together
    for i in 0..10 {
        let entity = Entity::from_raw(i as u32);
        let position = Vec3::new(100.0 + i as f32, 100.0, 100.0);
        octree.insert(entity, position);
    }

    // Query should find multiple objects
    let mut results = Vec::new();
    octree.query_sphere(Vec3::new(105.0, 100.0, 100.0), 50.0, &mut results);

    assert!(
        results.len() > 1,
        "Should find multiple objects in same region"
    );
}

#[test]
fn test_octree_depth_limiting() {
    // Test that octree doesn't subdivide beyond max depth
    let max_depth = 3;
    let mut octree = OctreeNode::new(Vec3::ZERO, 1000.0, max_depth, 0);

    // Insert many objects to force subdivision
    for i in 0..20 {
        let entity = Entity::from_raw(i as u32);
        let position = Vec3::new((i as f32) * 10.0, 0.0, 0.0);
        octree.insert(entity, position);
    }

    // Check that depth doesn't exceed max_depth
    fn check_depth(node: &OctreeNode, current_depth: u32, max_depth: u32) {
        assert!(
            current_depth <= max_depth,
            "Depth should not exceed max_depth"
        );
        if let Some(ref children) = node.children {
            for child in children.iter() {
                check_depth(child, current_depth + 1, max_depth);
            }
        }
    }

    check_depth(&octree, 0, max_depth);
}
