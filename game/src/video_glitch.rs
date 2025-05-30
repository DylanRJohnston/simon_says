use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{Animator, Lens, Tween, component_animator_system};
use bevy_video_glitch::VideoGlitchSettings;

use crate::{delayed_command::DelayedCommandExt, game_state::GameState, player::Death};

pub struct VideoGlitchPlugin;

// TODO: https://www.shadertoy.com/view/MtXBDs
impl Plugin for VideoGlitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, component_animator_system::<VideoGlitchSettings>)
            .add_plugins(bevy_video_glitch::VideoGlitchPlugin)
            .add_systems(Update, video_glitch)
            .add_observer(player_death);
    }
}

#[derive(Debug, Component)]
pub struct GlitchLens {
    start: f32,
    end: f32,
}

impl Lens<VideoGlitchSettings> for GlitchLens {
    fn lerp(
        &mut self,
        target: &mut dyn bevy_tweening::Targetable<VideoGlitchSettings>,
        ratio: f32,
    ) {
        target.intensity = self.start + (self.end - self.start) * ratio;
    }
}

fn video_glitch(
    state: Res<State<GameState>>,
    mut commands: Commands,
    video_glitch: Query<Entity, With<VideoGlitchSettings>>,
    mut prev_state: Local<GameState>,
) -> Result {
    if !state.is_changed() {
        return Ok(());
    }

    match (*prev_state, state.get()) {
        (GameState::Paused, GameState::MainMenu | GameState::InGame) => {
            commands
                .entity(video_glitch.single()?)
                .insert(Animator::new(Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(1.),
                    GlitchLens {
                        start: 0.4,
                        end: 0.,
                    },
                )));
        }
        (GameState::MainMenu | GameState::InGame, GameState::Paused) => {
            commands
                .entity(video_glitch.single()?)
                .insert(Animator::new(Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(1.),
                    GlitchLens {
                        start: 0.0,
                        end: 0.4,
                    },
                )));
        }
        _ => {}
    }

    *prev_state = *state.get();

    Ok(())
}

fn player_death(
    _trigger: Trigger<Death>,
    mut commands: Commands,
    camera: Query<Entity, With<Camera>>,
) -> Result {
    let entity = camera.single()?;

    commands.delayed(1.0, move |commands| {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(1.0),
            GlitchLens {
                start: 0.0,
                end: 1.0,
            },
        )
        .then(Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(1.0),
            GlitchLens {
                start: 1.0,
                end: 0.0,
            },
        ));

        commands.entity(entity).insert(Animator::new(tween));
    });

    Ok(())
}
