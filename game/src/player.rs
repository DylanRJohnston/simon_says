use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::{
    actions::Action,
    delayed_command::DelayedCommand,
    level::{Level, Tile},
    simulation::SimulationStop,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .observe(respawn_player)
            .observe(attempt_action)
            .observe(player_death)
            .observe(level_completed)
            .add_systems(Update, animate_player_movement)
            .add_systems(Update, debug_keyboard_move_forward);
    }
}

#[derive(Debug, Resource, Deref)]
pub struct PlayerMesh(Handle<Mesh>);

#[derive(Debug, Resource, Deref)]
pub struct PlayerMaterial(Handle<StandardMaterial>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Capsule3d::new(0.3, 0.95 / 2.0));
    commands.insert_resource(PlayerMesh(mesh));

    let material = materials.add(Color::srgb_u8(0xff, 0xcd, 0x75));
    commands.insert_resource(PlayerMaterial(material));
}

#[derive(Debug, Component, Default, Clone, Copy)]
pub struct Player {
    pub position: (i32, i32),
}

impl From<Player> for Vec3 {
    fn from(value: Player) -> Self {
        Vec3::new(value.position.0 as f32, 1.0, value.position.1 as f32)
    }
}

impl<'a> From<&'a Player> for Vec3 {
    fn from(value: &'a Player) -> Self {
        Vec3::new(value.position.0 as f32, 1.0, value.position.1 as f32)
    }
}

#[derive(Debug, Event)]
pub struct RespawnPlayer;

pub fn respawn_player(
    _event: Trigger<RespawnPlayer>,
    mesh: Res<PlayerMesh>,
    material: Res<PlayerMaterial>,
    players: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for player in &players {
        commands.entity(player).despawn_recursive();
    }

    let position = Vec3::new(0.0, 1.0, 0.0);

    commands.spawn((
        Player::default(),
        PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
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

fn animate_player_movement(
    mut commands: Commands,
    players: Query<(Entity, &Player, &Transform), Changed<Player>>,
) {
    for (entity, player, transform) in &players {
        // Hacky way of not animating player movement when first spawning
        if transform.translation.y > 1. {
            continue;
        }

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.2),
            TransformPositionLens {
                start: transform.translation,
                end: Vec3::new(player.position.0 as f32, 1., player.position.1 as f32),
            },
        );

        commands.entity(entity).insert(Animator::new(tween));
    }
}

fn attempt_action(
    trigger: Trigger<Action>,
    mut commands: Commands,
    level: Res<Level>,
    mut players: Query<(Entity, &mut Player)>,
) {
    for (entity, mut player) in &mut players {
        match trigger.event() {
            Action::Forward => player.position.0 += 1,
            Action::Backward => player.position.0 -= 1,
            Action::Left => player.position.1 -= 1,
            Action::Right => player.position.1 += 1,
        }

        match level.get(player.position) {
            Some(Tile::Finish) => commands.trigger(LevelCompleted),
            Some(_) => {}
            None => commands.trigger_targets(Death::Fell, entity),
        }
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

    commands.spawn(DelayedCommand::new(1.5, |commands| {
        commands.trigger(RespawnPlayer);
    }));
}

fn level_completed(_trigger: Trigger<LevelCompleted>, mut commands: Commands) {
    tracing::info!("Level Completed");

    commands.trigger(SimulationStop);
}

#[derive(Debug, Clone, Copy, Event)]
pub struct LevelCompleted;
