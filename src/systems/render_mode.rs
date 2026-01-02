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
            object_threshold: 10000,
            render_fraction: 1.0,
        }
    }
}

pub fn update_render_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut render_mode: ResMut<RenderMode>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        if render_mode.object_threshold == 10000 {
            render_mode.object_threshold = 50000;
            render_mode.render_fraction = 0.3;
            info!("Render mode: HIGH DENSITY (render 30% of >50k objects)");
        } else if render_mode.object_threshold == 50000 {
            render_mode.object_threshold = 100000;
            render_mode.render_fraction = 0.1;
            info!("Render mode: EXTREME DENSITY (render 10% of >100k objects)");
        } else {
            render_mode.object_threshold = 10000;
            render_mode.render_fraction = 1.0;
            info!("Render mode: FULL (render all objects)");
        }
    }
}
