use bevy::prelude::*;
use crate::components::*;
use std::fs::File;
use std::io::Write;

/// Snapshot of an object's state at a point in time
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectSnapshot {
    pub entity_id: u64, // Simplified entity ID
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub is_satellite: bool,
    pub name: String,
}

/// Snapshot of a collision event
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CollisionEventSnapshot {
    pub timestamp: f64,
    pub entity1_id: u64,
    pub entity2_id: u64,
    pub position: [f64; 3],
    pub energy: f64,
}

/// Snapshot of debris creation
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DebrisEventSnapshot {
    pub timestamp: f64,
    pub parent_collision_id: u32,
    pub debris_count: u32,
    pub position: [f64; 3],
}

/// A single frame of simulation state
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationFrame {
    pub timestamp: f64,
    pub objects: Vec<ObjectSnapshot>,
    pub collisions: Vec<CollisionEventSnapshot>,
    pub debris_created: Vec<DebrisEventSnapshot>,
}

/// Resource to manage simulation recording
#[derive(Resource)]
pub struct SimulationRecorder {
    pub frames: Vec<SimulationFrame>,
    pub is_recording: bool,
    pub frame_interval: f64, // Record every N seconds
    pub last_recorded_time: f64,
    pub file_path: Option<String>,
}

impl Default for SimulationRecorder {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            is_recording: false,
            frame_interval: 1.0, // Record once per second
            last_recorded_time: 0.0,
            file_path: None,
        }
    }
}

impl SimulationRecorder {
    pub fn start(&mut self, file_path: String) {
        self.is_recording = true;
        self.frames.clear();
        self.last_recorded_time = 0.0;
        self.file_path = Some(file_path);
        info!("Started recording simulation");
    }

    pub fn stop(&mut self) {
        self.is_recording = false;
        info!("Stopped recording - {} frames captured", self.frames.len());
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.file_path.as_ref()
            .ok_or("No file path set")?;
        
        let json = serde_json::to_string_pretty(&self.frames)?;
        
        let mut file = File::create(file_path)?;
        file.write_all(json.as_bytes())?;
        
        info!("Saved {} frames to {}", self.frames.len(), file_path);
        Ok(())
    }

    pub fn add_frame(&mut self, frame: SimulationFrame) {
        self.frames.push(frame);
    }
}

