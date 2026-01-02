use bevy::asset::Handle;
use bevy::prelude::*;

#[derive(Resource)]
pub struct MaterialsCache {
    pub satellite_material: Handle<StandardMaterial>,
    pub debris_material: Handle<StandardMaterial>,
    pub flash_material: Handle<StandardMaterial>,
}

pub fn setup_materials_cache(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let satellite_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: false,
        ..default()
    });

    let debris_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::srgb(0.3, 0.0, 0.0).into(),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        unlit: false,
        ..default()
    });

    let flash_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.8, 0.3),
        emissive: Color::srgb(10.0, 8.0, 5.0).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(MaterialsCache {
        satellite_material,
        debris_material,
        flash_material,
    });
}
