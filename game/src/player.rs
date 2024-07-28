use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, render::view::NoFrustumCulling, tasks::futures_lite::future};
use bevy_kira_audio::{AudioChannel, AudioControl};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens},
    Animator, EaseFunction, Sequence, Tracks, Tween, Tweenable,
};

use crate::{
    actions::Action,
    delayed_command::{DelayedCommand, DelayedCommandExt},
    game_state::{GameState, ModelAssets, SoundAssets},
    level::{Level, Tile},
    music::EffectChannel,
    simulation::{run_simulation_step, SimulationEvent, SimulationPause, SimulationStop},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.observe(spawn_player)
            .observe(attempt_action)
            .observe(player_death)
            .observe(level_completed)
            .observe(animate_player_movement)
            .observe(despawn_player)
            .observe(play_player_death_sound)
            .add_systems(Update, uncullable_mesh)
            .add_systems(OnExit(GameState::Loading), pre_instance_player_mesh);
    }
}

#[derive(Debug, Component)]
pub struct Uncullable;

fn pre_instance_player_mesh(mut commands: Commands, player_mesh: Res<ModelAssets>) {
    commands.spawn((
        Uncullable,
        SceneBundle {
            scene: player_mesh.player.clone(),
            transform: Transform {
                translation: Vec3::new(-100., -100., -100.),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
    ));
}

fn has_uncullable_parent(
    entity: Entity,
    root_query: &Query<Entity, With<Uncullable>>,
    parent_query: &Query<&Parent>,
) -> bool {
    if root_query.get(entity).is_ok() {
        return true;
    }

    if let Ok(parent) = parent_query.get(entity) {
        if has_uncullable_parent(parent.get(), root_query, parent_query) {
            return true;
        }
    }

    false
}

fn uncullable_mesh(
    mut commands: Commands,
    mesh: Query<Entity, Added<Handle<Mesh>>>,
    parents: Query<&Parent>,
    root_query: Query<Entity, With<Uncullable>>,
) {
    for mesh in &mesh {
        if has_uncullable_parent(mesh, &root_query, &parents) {
            commands
                .entity(mesh)
                .insert((Name::new("unculled_mesh"), NoFrustumCulling));
        }
    }
}

#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Eq, Hash)]
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
pub struct SpawnPlayer;

const PLAYER_Y_OFFSET: f32 = 0.5;

pub fn spawn_player(
    _event: Trigger<SpawnPlayer>,
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
        let (new_player, event) = run_simulation_step(&level, *player, *trigger.event());
        *player = new_player;

        commands.trigger_targets(
            PlayerMove {
                position: player.position,
                action: *trigger.event(),
            },
            entity,
        );

        match event {
            Some(SimulationEvent::Finished) => commands.trigger(LevelCompleted),
            Some(SimulationEvent::Died) => commands.trigger_targets(Death::Fell, entity),
            None => {}
        }
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub enum Death {
    Fell,
}

#[derive(Debug, Event)]
struct PlayPlayerDeathSound;

fn play_player_death_sound(
    _trigger: Trigger<PlayPlayerDeathSound>,
    sounds: Res<SoundAssets>,
    effect_channel: Res<AudioChannel<EffectChannel>>,
) {
    effect_channel
        .play(sounds.death_glitch.clone())
        .with_volume(0.2)
        .with_playback_rate(0.8 + rand::random::<f64>() * 0.4);
}

fn player_death(trigger: Trigger<Death>, mut commands: Commands, query: Query<&Player>) {
    let entity = trigger.entity();
    let player = *query.get(entity).unwrap();

    commands.trigger(SimulationPause);

    commands.delayed(1.3, |commands| commands.trigger(PlayPlayerDeathSound));

    commands.delayed(0.5, move |commands| {
        commands.entity(entity).insert(Animator::new(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_secs_f32(1.0),
            TransformPositionLens {
                start: Vec3::from(player),
                end: Vec3::from(player) + Vec3::Y * -20.,
            },
        )));
    });

    commands.delayed(2., |commands| {
        commands.trigger(SimulationStop);
        commands.trigger(SpawnPlayer);
    });
}

#[derive(Debug, Event)]
pub struct DespawnPlayer;

fn despawn_player(
    _trigger: Trigger<DespawnPlayer>,
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
}

fn level_completed(
    _trigger: Trigger<LevelCompleted>,
    mut commands: Commands,
    players: Query<(Entity, &Player)>,
) {
    commands.trigger(DespawnPlayer);
    commands.trigger(SimulationPause);
    commands.spawn(DelayedCommand::new(2.1, move |commands| {
        commands.trigger(SimulationStop);
        commands.trigger(SpawnPlayer);
    }));
}

#[derive(Debug, Clone, Copy, Event)]
pub struct LevelCompleted;
