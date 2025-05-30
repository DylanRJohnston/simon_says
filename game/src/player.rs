use std::time::Duration;

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_kira_audio::{AudioChannel, AudioControl};
use bevy_tweening::{
    Animator, Sequence, Tracks, Tween, Tweenable,
    lens::{TransformPositionLens, TransformRotationLens},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    actions::{Action, CWRotation},
    assets::{ModelAssets, SoundAssets},
    delayed_command::{DelayedCommand, DelayedCommandExt},
    game_state::GameState,
    level::{Level, Tile},
    music::EffectChannel,
    simulation::{SimulationEvent, SimulationPause, SimulationStop, run_simulation_step},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_player)
            .add_observer(attempt_action)
            .add_observer(player_death)
            .add_observer(level_completed)
            .add_observer(animate_player_movement)
            .add_observer(despawn_player)
            .add_observer(play_player_death_sound)
            .add_systems(Update, uncullable_mesh)
            .add_systems(OnExit(GameState::Loading), pre_instance_player_mesh);
    }
}

#[derive(Debug, Component)]
pub struct Uncullable;

fn pre_instance_player_mesh(mut commands: Commands, player_mesh: Res<ModelAssets>) {
    commands.spawn((
        Uncullable,
        SceneRoot(player_mesh.player.clone()),
        Transform {
            translation: Vec3::new(-100., -100., -100.),
            scale: Vec3::ONE,
            ..default()
        },
    ));
}

fn has_uncullable_parent(
    entity: Entity,
    root_query: &Query<Entity, With<Uncullable>>,
    child_of: &Query<&ChildOf>,
) -> bool {
    if root_query.get(entity).is_ok() {
        return true;
    }

    if let Ok(parent) = child_of.get(entity) {
        if has_uncullable_parent(parent.parent(), root_query, child_of) {
            return true;
        }
    }

    false
}

fn uncullable_mesh(
    mut commands: Commands,
    mesh: Query<Entity, Added<Mesh3d>>,
    parents: Query<&ChildOf>,
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
    pub rotation: CWRotation,
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
        commands.entity(player).despawn();
    }

    level
        .tiles
        .iter()
        .filter_map(|(pos, tile)| match tile {
            Tile::Start(rot) => Some((pos, rot)),
            _ => None,
        })
        .for_each(|(start, rot)| {
            let position = Vec3::new(start.0 as f32, PLAYER_Y_OFFSET, start.1 as f32);

            let player = Player {
                position: *start,
                rotation: *rot,
            };

            commands.spawn((
                player,
                SceneRoot(mesh.player.clone()),
                Transform {
                    translation: position + Vec3::Y * 10.0,
                    rotation: player.rotation.to_quat(),
                    scale: Vec3::ONE * 0.25,
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
        });
}

#[derive(Debug, Event)]
pub struct PlayerMove {
    player: Player,
    action: Action,
}

fn animate_player_movement(
    trigger: Trigger<PlayerMove>,
    players: Query<&Transform>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let player = trigger.player;
    let action = trigger.action;
    let model = players.get(entity).unwrap();

    let transform: Box<dyn Tweenable<Transform>> = Box::new(Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_secs_f32(0.2),
        TransformPositionLens {
            start: model.translation,
            end: Vec3::new(
                trigger.event().player.position.0 as f32,
                PLAYER_Y_OFFSET,
                trigger.event().player.position.1 as f32,
            ),
        },
    ));

    let desired_rotation = player.rotation.to_quat();

    let tilt = match action {
        Action::Forward => Quat::from_rotation_z(-0.2) * desired_rotation,
        Action::Backward => Quat::from_rotation_z(0.2) * desired_rotation,
        Action::Left => Quat::from_rotation_x(-0.2) * desired_rotation,
        Action::Right => Quat::from_rotation_x(0.2) * desired_rotation,
    };

    let rotation: Box<dyn Tweenable<Transform>> = Box::new(Sequence::new([
        Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_secs_f32(0.1),
            TransformRotationLens {
                start: model.rotation,
                end: tilt,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.1),
            TransformRotationLens {
                start: tilt,
                end: desired_rotation,
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
    let (entities, mut_players) = players.iter_mut().collect::<(Vec<_>, Vec<_>)>();

    let (new_players, events) = run_simulation_step(
        &level,
        &mut_players
            .iter()
            .map(|player| **player)
            .collect::<Vec<_>>(),
        *trigger.event(),
    )
    .into_iter()
    .collect::<(Vec<_>, Vec<_>)>();

    mut_players
        .into_iter()
        .zip(entities.iter())
        .zip(new_players)
        .for_each(|((mut player, entity), new_player)| {
            *player = new_player;

            commands.trigger_targets(
                PlayerMove {
                    player: *player,
                    action: player.rotation.to_combinator()(trigger.event()),
                },
                *entity,
            );
        });

    if events
        .iter()
        .all(|event| matches!(event, Some(SimulationEvent::Finished)))
    {
        commands.trigger(LevelCompleted);
    }

    for index in events.iter().filter_map(|event| match event {
        Some(SimulationEvent::Died(index)) => Some(*index),
        _ => None,
    }) {
        commands.trigger_targets(Death::Fell, entities[index]);
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
    mut rand: Local<Option<SmallRng>>,
) {
    let rand = rand.get_or_insert_with(|| SmallRng::seed_from_u64(0));

    effect_channel
        .play(sounds.death_glitch.clone())
        .with_volume(0.2)
        .with_playback_rate(0.8 + rand.random::<f64>() * 0.4);
}

fn player_death(trigger: Trigger<Death>, mut commands: Commands, query: Query<&Player>) {
    let entity = trigger.target();
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
            commands.get_entity(entity).map(|mut commands| {
                commands.insert(Animator::new(Tween::new(
                    EaseFunction::QuadraticIn,
                    Duration::from_secs_f32(1.0),
                    TransformPositionLens {
                        start: Vec3::from(player),
                        end: Vec3::from(player) + Vec3::Y * 10.,
                    },
                )));
            });
        }));
    }
}

fn level_completed(_trigger: Trigger<LevelCompleted>, mut commands: Commands) {
    commands.trigger(DespawnPlayer);
    commands.trigger(SimulationPause);
    commands.spawn(DelayedCommand::new(2.1, move |commands| {
        commands.trigger(SimulationStop);
        commands.trigger(SpawnPlayer);
    }));
}

#[derive(Debug, Clone, Copy, Event)]
pub struct LevelCompleted;
