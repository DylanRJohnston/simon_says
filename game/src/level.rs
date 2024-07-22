use bevy::{prelude::*, utils::HashMap};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};
use std::{f32::consts::PI, sync::LazyLock, time::Duration};

use crate::{
    player::{self, Player, RespawnPlayer},
    ui::constants::BUTTON_SUCCESS_COLOR,
};

#[derive(Debug, Clone, Resource)]
pub struct Level {
    tiles: HashMap<(i32, i32), Tile>,
}

impl Level {
    pub fn get(&self, position: (i32, i32)) -> Option<&Tile> {
        self.tiles.get(&position)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Tile {
    Basic,
    Finish,
}

pub static LEVELS: LazyLock<[Level; 1]> = LazyLock::new(|| {
    [Level {
        tiles: HashMap::from([
            ((0, 0), Tile::Basic),
            ((1, 0), Tile::Basic),
            ((2, 0), Tile::Basic),
            ((3, 0), Tile::Basic),
            ((4, 0), Tile::Finish),
        ]),
    }]
});

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, spawn_level)
            .add_systems(Startup, spawn_eye)
            .add_systems(Update, eye_track_player)
            .add_systems(Update, animate_eye)
            .add_systems(Update, eye_boredom);
    }
}

#[derive(Debug, Resource, Deref)]
pub struct TileMesh(Handle<Mesh>);

#[derive(Debug, Resource)]
pub struct TileMaterials {
    pub basic: Handle<StandardMaterial>,
    pub finish: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tile_handle = meshes.add(Cuboid::new(0.95, 0.95, 0.95));
    commands.insert_resource(TileMesh(tile_handle));

    let basic = materials.add(Color::srgb_u8(0x3b, 0x5d, 0xc9));

    let finish = materials.add(*BUTTON_SUCCESS_COLOR);
    commands.insert_resource(TileMaterials { basic, finish });

    commands.insert_resource(LEVELS[0].clone());
}

fn spawn_level(
    mut commands: Commands,
    level: Res<Level>,
    tile_mesh: Res<TileMesh>,
    tile_material: Res<TileMaterials>,
) {
    if !level.is_changed() {
        return;
    }

    for ((x, y), _tile) in &level.tiles {
        let position = Vec3::new(*x as f32, 0.0, *y as f32);

        commands.spawn((
            PbrBundle {
                mesh: tile_mesh.clone(),
                material: match _tile {
                    Tile::Basic => tile_material.basic.clone(),
                    Tile::Finish => tile_material.finish.clone(),
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            Animator::new(Tween::new(
                EaseFunction::CubicOut,
                Duration::from_secs_f32(1. + rand::random::<f32>()),
                TransformPositionLens {
                    start: position - Vec3::Y * 10.0,
                    end: position,
                },
            )),
        ));
    }

    commands.trigger(RespawnPlayer);
}

#[derive(Debug, Component, Default)]
pub struct Eye {
    pub target: Vec3,
    pub boredom: Timer,
}

const IRIS_CENTER: Vec3 = Vec3::new(-0.31, -0.28, 0.0);

#[derive(Debug, Component, Default)]
pub struct Iris;

fn spawn_eye(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
) {
    let quad_handle = meshes.add(Rectangle::new(8., 8.));

    let eye_material = material.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/eye.png")),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.0,
        ..default()
    });

    let iris_material = material.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/iris.png")),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.0,
        ..default()
    });

    for (x, y, z) in [(3.0, 1.0, -6.0), (1.0, -6.0, -6.0), (-3.0, -1.0, -6.0)] {
        commands
            .spawn((
                Eye::default(),
                Name::new("Simon"),
                PbrBundle {
                    mesh: quad_handle.clone(),
                    material: eye_material.clone(),
                    transform: Transform::from_xyz(x, y, z),
                    ..default()
                },
            ))
            .with_children(|eye| {
                eye.spawn((
                    Iris,
                    PbrBundle {
                        mesh: quad_handle.clone(),
                        material: iris_material.clone(),
                        transform: Transform::from_translation(IRIS_CENTER),
                        ..default()
                    },
                ));
            });
    }
}

fn eye_track_player(
    players: Query<&Transform, (Changed<Transform>, With<Player>)>,
    mut eye: Query<&mut Eye>,
) {
    if let Some(player) = players.iter().next() {
        for mut eye in &mut eye {
            eye.target = player.translation;
            eye.boredom = Timer::from_seconds(2.5 + rand::random::<f32>() * 2.5, TimerMode::Once);
        }
    }
}

fn animate_eye(
    mut eye: Query<(&mut Transform, &Eye, &Children), Without<Iris>>,
    mut iris: Query<&mut Transform, With<Iris>>,
    time: Res<Time>,
) {
    for (mut transform, eye, children) in &mut eye {
        let mut iris = iris.get_mut(children[0]).unwrap();

        let base_rotation = Quat::from_euler(EulerRot::XYZ, 0., PI, 0.);

        let target = transform.looking_at(eye.target, Vec3::Y).rotation * base_rotation;
        let difference = (transform.rotation.conjugate() * target).xyz();

        iris.translation = iris.translation.lerp(
            IRIS_CENTER + 2. * Vec3::new(-difference.y, difference.x, 0.),
            4.0 * time.delta_seconds(),
        );

        let rotation = transform.rotation.lerp(
            transform.looking_at(eye.target, Vec3::Y).rotation * base_rotation,
            0.5 * time.delta_seconds(),
        );

        transform.rotation = rotation;
    }
}

fn eye_boredom(mut eye: Query<&mut Eye>, time: Res<Time>) {
    for mut eye in &mut eye {
        if !eye.boredom.tick(time.delta()).just_finished() {
            continue;
        }

        eye.target = Vec3::new(
            rand::random::<f32>() * 10. - 5.,
            rand::random::<f32>() * 3. - 1.5,
            rand::random::<f32>() * 10. - 5.,
        );

        eye.boredom = Timer::new(
            Duration::from_secs_f32(2. + rand::random::<f32>() * 3.0),
            TimerMode::Once,
        );
    }
}
