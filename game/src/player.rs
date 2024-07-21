use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .observe(respawn_player)
            .observe(move_forward)
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

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Event)]
pub struct RespawnPlayer;

pub fn respawn_player(
    _event: Trigger<RespawnPlayer>,
    mesh: Res<PlayerMesh>,
    material: Res<PlayerMaterial>,
    mut commands: Commands,
) {
    let position = Vec3::new(0.0, 1.0, 0.0);

    commands.spawn((
        Player,
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
        commands.trigger(MoveForward);
    }
}

#[derive(Debug, Event)]
pub struct MoveForward;

pub fn move_forward(
    _event: Trigger<MoveForward>,
    players: Query<(Entity, &Transform), With<Player>>,
    mut commands: Commands,
) {
    for (entity, transform) in &players {
        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.2),
            TransformPositionLens {
                start: transform.translation,
                end: transform.translation + Vec3::X,
            },
        );

        commands.entity(entity).insert(Animator::new(tween));
    }
}
