use bevy::{
    app::{App, Plugin},
    asset::Handle,
    prelude::{AppExtStates, Image, Resource},
    scene::Scene,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use bevy_kira_audio::AudioSource;

use crate::game_state::GameState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
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

#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(path = "music/level_completed.ogg")]
    pub level_completed: Handle<AudioSource>,

    #[asset(path = "music/where_am_i.ogg")]
    pub where_am_i: Handle<AudioSource>,

    #[asset(path = "music/anachronism.ogg")]
    pub anachronism: Handle<AudioSource>,

    #[asset(path = "music/change_level.ogg")]
    pub change_level: Handle<AudioSource>,

    #[asset(path = "music/pause_music.ogg")]
    pub pause_music: Handle<AudioSource>,
    // #[asset(path = "music/didact.ogg")]
    // pub didact: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct SoundAssets {
    #[asset(path = "sounds/move.ogg")]
    pub player_move: Handle<AudioSource>,

    #[asset(
        paths(
            "sounds/simon_dialogue_long_1.ogg",
            "sounds/simon_dialogue_long_2.ogg",
            // "sounds/simon_dialogue_long_3.ogg",
            "sounds/simon_dialogue_long_4.ogg",
            "sounds/simon_dialogue_long_5.ogg",
            "sounds/simon_dialogue_long_6.ogg",
            "sounds/simon_dialogue_long_7.ogg",
            "sounds/simon_dialogue_long_8.ogg",
            "sounds/simon_dialogue_long_9.ogg",
        ),
        collection(typed)
    )]
    pub dialogue: Vec<Handle<AudioSource>>,

    #[asset(path = "sounds/death_glitch.ogg")]
    pub death_glitch: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/eye.png")]
    pub eye: Handle<Image>,

    #[asset(path = "textures/iris.png")]
    pub iris: Handle<Image>,

    #[asset(path = "textures/cw_rot.png")]
    pub cw_rot: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/Animated Human.glb#Scene0")]
    pub player: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct IconAssets {
    #[asset(path = "icons/remove.png")]
    pub remove: Handle<Image>,

    #[asset(path = "icons/up.png")]
    pub up: Handle<Image>,

    #[asset(path = "icons/down.png")]
    pub down: Handle<Image>,

    #[asset(path = "icons/bars.png")]
    pub bars: Handle<Image>,

    #[asset(path = "icons/mute.png")]
    pub mute: Handle<Image>,

    #[asset(path = "icons/unmute.png")]
    pub unmute: Handle<Image>,
}
