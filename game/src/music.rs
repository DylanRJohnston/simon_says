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
            .add_systems(
                Update,
                play_music.run_if(|state: Option<Res<State<GameState>>>| {
                    !matches!(
                        state.as_ref().map(|state| state.get()),
                        Some(GameState::Loading)
                    )
                }),
            )
            .observe(level_completed)
            .observe(suppress_music)
            .observe(set_music_volume)
            .observe(player_move);
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct MusicHandle(Handle<AudioInstance>);

const DEFAULT_MUSIC_VOLUME: f64 = 0.8;

fn play_music(
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
            Duration::from_secs_f32(10.),
            AudioEasing::OutPowi(2),
        ))
        .handle();

    commands.insert_resource(MusicHandle(handle));
}

#[derive(SystemParam)]
pub struct Music<'w> {
    handle: Res<'w, MusicHandle>,
    audio_instances: ResMut<'w, Assets<AudioInstance>>,
}

impl Deref for Music<'_> {
    type Target = AudioInstance;
    fn deref(&self) -> &Self::Target {
        self.audio_instances.get(&self.handle.0).unwrap()
    }
}

impl DerefMut for Music<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.audio_instances.get_mut(&self.handle.0).unwrap()
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
        falling_edge: Duration::from_secs(5),
    });
}

#[derive(Debug, Clone, Event)]
pub struct SuppressMusicVolume {
    pub volume: f64,
    pub leading_edge: Duration,
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
        event.leading_edge.as_secs_f32(),
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

    music.set_volume(event.volume, AudioTween::new(event.duration, event.easing));
}

fn player_move(_trigger: Trigger<PlayerMove>, sounds: Res<SoundAssets>, audio: Res<Audio>) {
    let handle = sounds.player_move.clone();
    audio
        .play(handle)
        .with_volume(0.4)
        .start_from(0.2)
        .with_playback_rate(1.0 + 0.02 * rand::random::<f64>());
}
