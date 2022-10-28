use crate::AppState;
use bevy::{prelude::*, gltf::Gltf};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [AssetLoader] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at https://bevy-cheatbook.github.io/features/assets.html
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        LoadingState::new(AppState::Loading)
            .with_collection::<FontAssets>()
            .with_collection::<AudioAssets>()
            .with_collection::<TextureAssets>()
            .with_collection::<ModelAssets>()
            .continue_to_state(AppState::Menu)
            .build(app);
    }
}

// the following asset collections will be loaded during the State `AppState::Loading`
// when done loading, they will be inserted as resources (see https://github.com/NiklasEi/bevy_asset_loader)

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub texture_bevy: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct ModelAssets {
//#[asset(path = "map/ground_4/scene.gltf")]   // nice! smoll
    #[asset(path = "map/terrain/scene.gltf")]
//    #[asset(path = "map/island/scene.gltf")]
    pub terrain: Handle<Gltf>,
//    #[asset(path = "map/mountain.glb#Scene0")]
//    pub mountain: Handle<Scene>,
    #[asset(path = "Tank_1/PARTS/tank_green.gltf#Scene0")]
    pub tank_body: Handle<Scene>,
    #[asset(path = "Tank_1/PARTS/turret_green.gltf#Scene0")]
    pub tank_turret: Handle<Scene>,
    #[asset(path = "Tank_1/PARTS/cannon_green.gltf#Scene0")]
    pub tank_cannon: Handle<Scene>,
    #[asset(path = "Tank_1/GREEN/tank_1_green.glb#Scene0")]
    pub tank_green: Handle<Scene>,
/*
    #[asset(path = "Tank_1/PARTS/tank_green.gltf#Mesh0/Primitive0")]
    pub tank_body_mesh: Handle<Mesh>,
    #[asset(path = "Tank_1/PARTS/turret_green.gltf#Mesh0/Primitive0")]
    pub tank_turret_mesh: Handle<Mesh>,
    #[asset(path = "Tank_1/PARTS/cannon_green.gltf#Mesh0/Primitive0")]
    pub tank_cannon_mesh: Handle<Mesh>,
*/    
}
