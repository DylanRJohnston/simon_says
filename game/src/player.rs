use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens},
    Animator, EaseFunction, Sequence, Tracks, Tween, Tweenable,
};

use crate::{
    actions::Action,
    delayed_command::DelayedCommand,
    game_state::ModelAssets,
    level::{Level, Tile},
    simulation::SimulationStop,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.observe(respawn_player)
            .observe(attempt_action)
            .observe(player_death)
            .observe(level_completed)
            .observe(animate_player_movement)
            .add_systems(Update, debug_keyboard_move_forward);
    }
}

#[derive(Debug, Component, Default, Clone, Copy)]
pub struct Player {
    pub position: (i32, i32),
}

impl From<Player> for Vec3 {
    fn from(value: Player) -> Self {
        Vec3::new(
            value.position.0 as f32,
            PLAYER_Y_OFFSET,
            value.position.1 as f32,
        )
    }
}

impl<'a> From<&'a Player> for Vec3 {
    fn from(value: &'a Player) -> Self {
        Vec3::new(
            value.position.0 as f32,
            PLAYER_Y_OFFSET,
            value.position.1 as f32,
        )
    }
}

#[derive(Debug, Event)]
pub struct RespawnPlayer;

const PLAYER_Y_OFFSET: f32 = 0.5;

pub fn respawn_player(
    _event: Trigger<RespawnPlayer>,
    mesh: Res<ModelAssets>,
    // material: Res<PlayerMaterial>,
    players: Query<Entity, With<Player>>,
    level: Res<Level>,
    mut commands: Commands,
) {
    for player in &players {
        commands.entity(player).despawn_recursive();
    }

    let (start, _) = level
        .tiles
        .iter()
        .find(|(_, tile)| **tile == Tile::Start)
        .unwrap();

    let position = Vec3::new(start.0 as f32, PLAYER_Y_OFFSET, start.1 as f32);

    commands.spawn((
        Player { position: *start },
        SceneBundle {
            scene: mesh.player.clone(),
            transform: Transform {
                translation: position + Vec3::Y * 10.0,
                rotation: Quat::from_rotation_y(PI / 2.),
                scale: Vec3::ONE * 0.25,
            },
            ..default()
        },
        Animator::new(Tween::new(
            EaseFunction::CubicOut,
            Duration::from_secs_f32(2.0),
            TransformPositionLens {
                start: position + Vec3::Y * 10.0,
                end: position,
            },
        )),
    ));
}

pub fn debug_keyboard_move_forward(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        tracing::info!("space bar pressed");
        commands.trigger(Action::Forward);
    }
}

#[derive(Debug, Event)]
pub struct PlayerMove {
    position: (i32, i32),
    action: Action,
}

fn animate_player_movement(
    trigger: Trigger<PlayerMove>,
    players: Query<&Transform>,
    mut commands: Commands,
) {
    tracing::info!(player_move = ?trigger.event(), "moving player");

    let entity = trigger.entity();
    let player = players.get(entity).unwrap();

    let transform: Box<dyn Tweenable<Transform>> = Box::new(Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_secs_f32(0.2),
        TransformPositionLens {
            start: player.translation,
            end: Vec3::new(
                trigger.event().position.0 as f32,
                PLAYER_Y_OFFSET,
                trigger.event().position.1 as f32,
            ),
        },
    ));

    let rotation = match trigger.event().action {
        Action::Forward => Quat::from_rotation_z(-0.2) * player.rotation,
        Action::Backward => Quat::from_rotation_z(0.2) * player.rotation,
        Action::Left => Quat::from_rotation_x(-0.2) * player.rotation,
        Action::Right => Quat::from_rotation_x(0.2) * player.rotation,
        Action::Nothing => player.rotation,
    };

    let rotation: Box<dyn Tweenable<Transform>> = Box::new(Sequence::new([
        Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_secs_f32(0.1),
            TransformRotationLens {
                start: player.rotation,
                end: rotation,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.1),
            TransformRotationLens {
                start: rotation,
                end: player.rotation,
            },
        ),
    ]));

    commands
        .entity(entity)
        .insert(Animator::new(Tracks::new([transform, rotation])));
}

fn attempt_action(
    trigger: Trigger<Action>,
    mut commands: Commands,
    level: Res<Level>,
    mut players: Query<(Entity, &mut Player)>,
) {
    for (entity, mut player) in &mut players {
        let mut future_position = match trigger.event() {
            Action::Forward => (player.position.0 + 1, player.position.1),
            Action::Backward => (player.position.0 - 1, player.position.1),
            Action::Left => (player.position.0, player.position.1 - 1),
            Action::Right => (player.position.0, player.position.1 + 1),
            Action::Nothing => (player.position.0, player.position.1),
        };

        match level.get(future_position) {
            Some(Tile::Finish) => {
                commands.trigger(LevelCompleted);
            }
            Some(Tile::Wall) => {
                future_position = player.position;
            }
            Some(_) => {}
            None => {
                commands.trigger_targets(Death::Fell, entity);
            }
        }
        commands.trigger_targets(
            PlayerMove {
                position: future_position,
                action: *trigger.event(),
            },
            entity,
        );
        player.position = future_position;
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub enum Death {
    Fell,
}

fn player_death(trigger: Trigger<Death>, mut commands: Commands, query: Query<&Player>) {
    tracing::info!(reason = ?trigger.event(), "player died");

    let entity = trigger.entity();
    let player = *query.get(entity).unwrap();

    commands.trigger(SimulationStop);

    commands.spawn(DelayedCommand::new(0.5, move |commands| {
        commands.entity(entity).insert(Animator::new(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_secs_f32(1.0),
            TransformPositionLens {
                start: Vec3::from(player),
                end: Vec3::from(player) + Vec3::Y * -20.,
            },
        )));
    }));

    commands.spawn(DelayedCommand::new(2., |commands| {
        commands.trigger(RespawnPlayer);
    }));
}

fn level_completed(
    _trigger: Trigger<LevelCompleted>,
    mut commands: Commands,
    players: Query<(Entity, &Player)>,
) {
    for (entity, player) in &players {
        let player = *player;

        commands.spawn(DelayedCommand::new(0.3, move |commands| {
            commands.entity(entity).insert(Animator::new(Tween::new(
                EaseFunction::QuadraticIn,
                Duration::from_secs_f32(1.0),
                TransformPositionLens {
                    start: Vec3::from(player),
                    end: Vec3::from(player) + Vec3::Y * 10.,
                },
            )));
        }));
    }

    commands.trigger(SimulationStop);
    commands.spawn(DelayedCommand::new(2.1, move |commands| {
        commands.trigger(RespawnPlayer);
    }));
}

#[derive(Debug, Clone, Copy, Event)]
pub struct LevelCompleted;
