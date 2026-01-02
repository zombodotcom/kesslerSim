use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct RenderMode {
    pub object_threshold: usize,
    pub render_fraction: f32,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self {
            object_threshold: 5000, // Lower threshold for better performance
            render_fraction: 1.0,
        }
    }
}

impl RenderMode {
    // Removed auto-adjust - user doesn't want render culling
}

pub fn update_render_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut render_mode: ResMut<RenderMode>,
) {
    // Manual toggle with 'M' key (disabled - user wants full rendering)
    if keyboard.just_pressed(KeyCode::KeyM) {
        render_mode.render_fraction = 1.0;
        info!("Render mode: FULL (render all objects)");
    }
}
