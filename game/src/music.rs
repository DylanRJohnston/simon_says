use std::{cell::RefCell, time::Duration};

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    delayed_command::DelayedCommand,
    game_state::{GameState, MusicAssets, SoundAssets},
    player::{LevelCompleted, PlayerMove},
};

pub struct MusicPlugin;

#[derive(Debug, Resource)]
pub struct MusicChannel;

#[derive(Debug, Resource)]
pub struct EffectChannel;

#[derive(Debug, Resource)]
pub struct DialogueChannel;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_macro::wasm_bindgen]
extern "C" {
    fn stop_menu_music_js();
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .insert_resource(MusicHandles::default())
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectChannel>()
            .add_audio_channel::<DialogueChannel>()
            .add_systems(
                Update,
                loop_menu_music.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(Update, loop_game_music.run_if(in_state(GameState::InGame)))
            .add_systems(Update, loop_pause_music.run_if(in_state(GameState::Paused)))
            .add_systems(OnEnter(GameState::MainMenu), change_music)
            .add_systems(OnEnter(GameState::InGame), change_music)
            .add_systems(OnEnter(GameState::Paused), change_music)
            .observe(level_completed)
            .observe(suppress_music)
            .observe(set_music_volume)
            .observe(player_move)
            .observe(change_level);

        // We move the menu music into the web page for wasm
        #[cfg(target_arch = "wasm32")]
        app.add_systems(OnEnter(GameState::MainMenu), || unsafe {
            stop_menu_music_js();
        });
    }
}

const DEFAULT_MUSIC_VOLUME: f64 = 0.8;

#[derive(Debug, Default, Resource)]
struct MusicHandles {
    menu: Option<Handle<AudioInstance>>,
    game: Option<Handle<AudioInstance>>,
    pause: Option<Handle<AudioInstance>>,
}

fn change_music(
    state: Res<State<GameState>>,
    handles: Res<MusicHandles>,
    audio_instances: ResMut<Assets<AudioInstance>>,
    dialogue_channel: ResMut<AudioChannel<DialogueChannel>>,
) {
    let audio_instances = RefCell::new(audio_instances);

    let pause_music = |handle: &Option<Handle<AudioInstance>>| {
        handle.as_ref().and_then(|handle| {
            audio_instances
                .borrow_mut()
                .get_mut(handle)?
                .pause(AudioTween::new(
                    Duration::from_secs_f32(2.),
                    AudioEasing::InPowi(2),
                ))
        });
    };

    let play_music = |handle: &Option<Handle<AudioInstance>>| {
        handle.as_ref().and_then(|handle| {
            audio_instances
                .borrow_mut()
                .get_mut(handle)?
                .resume(AudioTween::new(
                    Duration::from_secs_f32(2.),
                    AudioEasing::OutPowi(2),
                ))
        });
    };

    let set_dialogue_volume = |volume: f64| {
        dialogue_channel.set_volume(volume).fade_in(AudioTween::new(
            Duration::from_secs_f32(0.2),
            AudioEasing::OutPowi(2),
        ));
    };

    match state.get() {
        GameState::MainMenu => {
            play_music(&handles.menu);
            pause_music(&handles.pause);
            pause_music(&handles.game);
        }
        GameState::InGame => {
            play_music(&handles.game);
            pause_music(&handles.menu);
            pause_music(&handles.pause);
            set_dialogue_volume(1.)
        }
        GameState::Paused => {
            play_music(&handles.pause);
            pause_music(&handles.menu);
            pause_music(&handles.game);
            set_dialogue_volume(0.);
        }
        GameState::Loading => {}
    }
}

fn loop_game_music(
    music: Res<MusicAssets>,
    audio: ResMut<AudioChannel<MusicChannel>>,
    mut handles: ResMut<MusicHandles>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() {
        return;
    }

    *timer = Timer::from_seconds(120., TimerMode::Once);

    handles.game.as_ref().and_then(|handle| {
        audio_instances.get_mut(handle)?.stop(AudioTween::new(
            Duration::from_secs_f32(10.),
            AudioEasing::InPowi(2),
        ))
    });

    handles.game = Some(
        audio
            .play(music.where_am_i.clone())
            .with_volume(DEFAULT_MUSIC_VOLUME)
            .fade_in(AudioTween::new(
                Duration::from_secs_f32(10.),
                AudioEasing::OutPowi(2),
            ))
            .handle(),
    );
}

fn loop_menu_music(
    music: Res<MusicAssets>,
    audio: ResMut<AudioChannel<MusicChannel>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut handles: ResMut<MusicHandles>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() {
        return;
    }

    *timer = Timer::from_seconds(91., TimerMode::Once);

    handles.menu.as_ref().and_then(|handle| {
        audio_instances.get_mut(handle)?.stop(AudioTween::new(
            Duration::from_secs_f32(5.),
            AudioEasing::InPowi(2),
        ))
    });

    handles.menu = Some(
        audio
            .play(music.anachronism.clone())
            .with_volume(DEFAULT_MUSIC_VOLUME)
            .fade_in(AudioTween::new(
                Duration::from_secs_f32(5.),
                AudioEasing::OutPowi(2),
            ))
            .handle(),
    );
}

fn loop_pause_music(
    music: Res<MusicAssets>,
    audio: ResMut<AudioChannel<MusicChannel>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut handles: ResMut<MusicHandles>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() {
        return;
    }

    *timer = Timer::from_seconds(55., TimerMode::Once);

    handles.pause.as_ref().and_then(|handle| {
        audio_instances.get_mut(handle)?.stop(AudioTween::new(
            Duration::from_secs_f32(5.),
            AudioEasing::InPowi(2),
        ))
    });

    handles.pause = Some(
        audio
            .play(music.pause_music.clone())
            .with_volume(1.5)
            .fade_in(AudioTween::new(
                Duration::from_secs_f32(5.),
                AudioEasing::OutPowi(2),
            ))
            .handle(),
    );
}

#[derive(Debug, Event)]
pub struct StartPauseMusic;

#[derive(Debug, Event)]
pub struct StopPauseMusic;

fn level_completed(
    _trigger: Trigger<LevelCompleted>,
    mut commands: Commands,
    music: Res<MusicAssets>,
    audio: Res<AudioChannel<EffectChannel>>,
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
pub struct PlayChangeLevelMusic;

fn change_level(
    _trigger: Trigger<PlayChangeLevelMusic>,
    mut commands: Commands,
    music: Res<MusicAssets>,
    audio: Res<AudioChannel<EffectChannel>>,
) {
    let handle = music.change_level.clone();
    audio.play(handle).with_volume(DEFAULT_MUSIC_VOLUME);

    commands.trigger(SuppressMusicVolume {
        volume: 0.0,
        leading_edge: Duration::from_secs_f32(0.6),
        middle: Duration::from_secs_f32(1.),
        falling_edge: Duration::from_secs_f32(2.),
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

fn set_music_volume(trigger: Trigger<SetMusicVolume>, music: ResMut<AudioChannel<MusicChannel>>) {
    let event = trigger.event();

    music
        .set_volume(event.volume)
        .fade_in(AudioTween::new(event.duration, event.easing));
}

fn player_move(
    _trigger: Trigger<PlayerMove>,
    sounds: Res<SoundAssets>,
    audio: Res<AudioChannel<EffectChannel>>,
) {
    let handle = sounds.player_move.clone();
    audio
        .play(handle)
        .with_volume(0.4)
        .start_from(0.2)
        .with_playback_rate(1.0 + 0.02 * rand::random::<f64>());
}
