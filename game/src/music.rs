use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::prelude::*;

use crate::{
    delayed_command::DelayedCommand,
    game_state::{GameState, MusicAssets, SoundAssets},
    player::{LevelCompleted, PlayerMove},
};

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Update, play_game_music.run_if(in_state(GameState::InGame)))
            .add_systems(OnEnter(GameState::InGame), stop_menu_music)
            .observe(level_completed)
            .observe(suppress_music)
            .observe(set_music_volume)
            .observe(player_move);

        // We move the menu music into the web page for wasm
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            play_menu_music.run_if(in_state(GameState::MainMenu)),
        );
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct GameMusicHandle(Handle<AudioInstance>);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct MenuMusicHandle(Handle<AudioInstance>);

const DEFAULT_MUSIC_VOLUME: f64 = 0.8;

fn play_game_music(
    mut commands: Commands,
    music: Res<MusicAssets>,
    audio: Res<Audio>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() {
        return;
    }

    *timer = Timer::from_seconds(120., TimerMode::Once);

    let handle = audio
        .play(music.where_am_i.clone())
        .with_volume(DEFAULT_MUSIC_VOLUME)
        .fade_in(AudioTween::new(
            Duration::from_secs_f32(2.),
            AudioEasing::OutPowi(2),
        ))
        .handle();

    commands.insert_resource(GameMusicHandle(handle));
}

fn play_menu_music(
    mut commands: Commands,
    music: Res<MusicAssets>,
    audio: Res<Audio>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() {
        return;
    }

    *timer = Timer::from_seconds(91., TimerMode::Once);

    let handle = audio
        .play(music.anachronism.clone())
        .with_volume(DEFAULT_MUSIC_VOLUME)
        .fade_in(AudioTween::new(
            Duration::from_secs_f32(2.),
            AudioEasing::OutPowi(2),
        ))
        .handle();

    commands.insert_resource(MenuMusicHandle(handle));
}

#[cfg(not(target_arch = "wasm32"))]
fn stop_menu_music(mut music: Music) {
    if let Some(handle) = music.menu_music() {
        handle.stop(AudioTween::linear(Duration::from_secs_f32(5.)));
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_macro::wasm_bindgen]
extern "C" {
    fn stop_menu_music_js();
}

#[cfg(target_arch = "wasm32")]
fn stop_menu_music() {
    unsafe {
        stop_menu_music_js();
    }
}

#[derive(SystemParam)]
pub struct Music<'w> {
    game_music_handle: Option<Res<'w, GameMusicHandle>>,
    menu_music_handle: Option<Res<'w, MenuMusicHandle>>,
    audio_instances: ResMut<'w, Assets<AudioInstance>>,
}

impl Music<'_> {
    fn game_music(&mut self) -> Option<&mut AudioInstance> {
        self.audio_instances
            .get_mut(&self.game_music_handle.as_mut()?.0)
    }

    fn menu_music(&mut self) -> Option<&mut AudioInstance> {
        self.audio_instances
            .get_mut(&self.menu_music_handle.as_mut()?.0)
    }
}

fn level_completed(
    _trigger: Trigger<LevelCompleted>,
    mut commands: Commands,
    music: Res<MusicAssets>,
    audio: Res<Audio>,
) {
    let handle = music.level_completed.clone();
    audio.play(handle).with_volume(DEFAULT_MUSIC_VOLUME);

    commands.trigger(SuppressMusicVolume {
        volume: 0.0,
        leading_edge: Duration::from_secs_f32(0.5),
        middle: Duration::from_secs_f32(0.),
        falling_edge: Duration::from_secs(5),
    });
}

#[derive(Debug, Clone, Event)]
pub struct SuppressMusicVolume {
    pub volume: f64,
    pub leading_edge: Duration,
    pub middle: Duration,
    pub falling_edge: Duration,
}

fn suppress_music(trigger: Trigger<SuppressMusicVolume>, mut commands: Commands) {
    let event = trigger.event();

    commands.trigger(SetMusicVolume {
        volume: event.volume,
        duration: event.leading_edge,
        easing: AudioEasing::InPowi(2),
    });

    let falling_edge = event.falling_edge;
    commands.spawn(DelayedCommand::new(
        event.leading_edge.as_secs_f32() + event.middle.as_secs_f32(),
        move |commands| {
            commands.trigger(SetMusicVolume {
                volume: DEFAULT_MUSIC_VOLUME,
                duration: falling_edge,
                easing: AudioEasing::OutPowi(2),
            });
        },
    ));
}

#[derive(Debug, Clone, Copy, Event)]
pub struct SetMusicVolume {
    pub volume: f64,
    pub duration: Duration,
    pub easing: AudioEasing,
}

fn set_music_volume(trigger: Trigger<SetMusicVolume>, mut music: Music) {
    tracing::info!(event = ?trigger.event(), "setting music volume");

    let event = trigger.event();

    if let Some(handle) = music.game_music() {
        handle.set_volume(event.volume, AudioTween::new(event.duration, event.easing));
    }
}

fn player_move(_trigger: Trigger<PlayerMove>, sounds: Res<SoundAssets>, audio: Res<Audio>) {
    let handle = sounds.player_move.clone();
    audio
        .play(handle)
        .with_volume(0.4)
        .start_from(0.2)
        .with_playback_rate(1.0 + 0.02 * rand::random::<f64>());
}
