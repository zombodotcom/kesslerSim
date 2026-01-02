use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::systems::tracking_ui::SatelliteSelection;

/// Resource to track FPS history for graph
#[derive(Resource, Default)]
pub struct FpsHistory {
    pub values: Vec<f32>,
    pub max_length: usize,
}

impl FpsHistory {
    pub fn new(max_length: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_length),
            max_length,
        }
    }

    pub fn add(&mut self, fps: f32) {
        self.values.push(fps);
        if self.values.len() > self.max_length {
            self.values.remove(0);
        }
    }

    pub fn average(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f32>() / self.values.len() as f32
    }
}

/// System to update FPS history
pub fn update_fps_history_system(
    mut fps_history: ResMut<FpsHistory>,
    time: Res<Time>,
) {
    let fps = 1.0 / time.delta_secs();
    fps_history.add(fps);
}

/// System to create on-screen UI
/// NOTE: Bevy 0.16.1 UI may have different module structure
pub fn setup_ui_system(
    mut _commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // UI temporarily disabled - NodeBundle/Style not found in Bevy 0.16.1 prelude
    // TODO: Find correct import path for Bevy 0.16.1 UI types
    // For now, stats are shown in console output
}

/// Marker components for UI elements
#[derive(Component)]
pub struct UiStatsPanel;

#[derive(Component)]
pub struct UiStatsText;

#[derive(Component)]
pub struct UiFpsGraph;

#[derive(Component)]
pub struct UiControlsPanel;

#[derive(Component)]
pub struct UiSatellitePanel;

#[derive(Component)]
pub struct UiSatelliteInfo;

/// System to update UI stats text
pub fn update_ui_stats_system(
    mut _text_query: Query<&mut Text, With<UiStatsText>>,
    orbital_query: Query<&OrbitalState, (With<Satellite>, With<RenderAsSatellite>)>,
    debris_query: Query<&OrbitalState, (With<Debris>, With<RenderAsDebris>)>,
    sim_time: Res<SimulationTime>,
    _fps_history: Res<FpsHistory>,
    time: Res<Time>,
) {
    // UI temporarily disabled - output stats to console instead
    let satellite_count = orbital_query.iter().count();
    let debris_count = debris_query.iter().count();
    let total = satellite_count + debris_count;
    let fps = 1.0 / time.delta_secs();
    let status = if sim_time.paused { "PAUSED" } else { "RUNNING" };
    let speed_text = match sim_time.speed_multiplier {
        1.0 => "1x",
        60.0 => "60x",
        3600.0 => "3600x",
        86400.0 => "86400x",
        _ => "?x",
    };
    
    // Stats are logged to console - UI will be fixed when we find correct Bevy 0.16.1 UI API
    info!(
        "FPS: {:.1} | Objects: {} total ({} satellites, {} debris) | Speed: {} | Time: {:.1}s | Status: {}",
        fps, total, satellite_count, debris_count, speed_text, sim_time.current, status
    );
}

/// System to update selected satellite info
#[allow(dead_code, unused_variables)]
pub fn update_satellite_info_system(
    mut _text_query: Query<&mut Text, With<UiSatelliteInfo>>,
    _satellite_selection: Res<SatelliteSelection>,
    _satellite_query: Query<(&Satellite, &OrbitalState, &TleData), (With<Satellite>, With<RenderAsSatellite>)>,
) {
    // TODO: Implement
}
