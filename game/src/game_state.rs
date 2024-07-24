use bevy::{
    app::{App, Plugin},
    asset::Handle,
    prelude::{AppExtStates, Image, Resource, States},
    scene::Scene,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use bevy_kira_audio::AudioSource;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::MainMenu)
                .load_collection::<MusicAssets>()
                .load_collection::<SoundAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<ModelAssets>()
                .load_collection::<IconAssets>(),
        );
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    InGame,
}

#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(path = "music/level_completed.ogg")]
    pub level_completed: Handle<AudioSource>,

    #[asset(path = "music/where_am_i.ogg")]
    pub where_am_i: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct SoundAssets {
    #[asset(path = "sounds/move.ogg")]
    pub player_move: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/eye.png")]
    pub eye: Handle<Image>,

    #[asset(path = "textures/iris.png")]
    pub iris: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/Animated Human.glb#Scene0")]
    pub player: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct IconAssets {
    #[asset(path = "icons/drag.png")]
    pub drag: Handle<Image>,

    #[asset(path = "icons/remove.png")]
    pub remove: Handle<Image>,
}
