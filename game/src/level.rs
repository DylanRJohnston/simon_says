use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};
use std::{sync::LazyLock, time::Duration};

use crate::player::RespawnPlayer;

#[derive(Debug, Clone, Resource)]
pub struct Level {
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub position: Vec3,
    pub typ: TileType,
}

impl Tile {
    pub fn basic(x: i32, z: i32) -> Self {
        Tile {
            position: Vec3::new(x as f32, 0.0, z as f32),
            typ: TileType::Basic,
        }
    }

    pub fn finish(x: i32, z: i32) -> Self {
        Tile {
            position: Vec3::new(x as f32, 0.0, z as f32),
            typ: TileType::Basic,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TileType {
    Basic,
}

pub static LEVELS: LazyLock<[Level; 1]> = LazyLock::new(|| {
    [Level {
        tiles: vec![
            Tile::basic(0, 0),
            Tile::basic(1, 0),
            Tile::basic(2, 0),
            Tile::basic(3, 0),
            Tile::finish(4, 0),
        ],
    }]
});

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, spawn_level);
    }
}

#[derive(Debug, Resource, Deref)]
pub struct TileMesh(Handle<Mesh>);

#[derive(Debug, Resource, Deref)]
pub struct BasicTileMaterial(Handle<StandardMaterial>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tile_handle = meshes.add(Cuboid::new(0.95, 0.95, 0.95));
    commands.insert_resource(TileMesh(tile_handle));

    let tile_material = materials.add(Color::srgb_u8(0x3b, 0x5d, 0xc9));
    commands.insert_resource(BasicTileMaterial(tile_material));

    commands.insert_resource(LEVELS[0].clone());
}

fn spawn_level(
    mut commands: Commands,
    level: Res<Level>,
    tile_mesh: Res<TileMesh>,
    tile_material: Res<BasicTileMaterial>,
) {
    if !level.is_changed() {
        return;
    }

    for tile in &level.tiles {
        commands.spawn((
            PbrBundle {
                mesh: tile_mesh.clone(),
                material: tile_material.clone(),
                transform: Transform::from_translation(tile.position),
                ..default()
            },
            Animator::new(Tween::new(
                EaseFunction::CubicOut,
                Duration::from_secs_f32(1. + rand::random::<f32>()),
                TransformPositionLens {
                    start: tile.position - Vec3::Y * 10.0,
                    end: tile.position,
                },
            )),
        ));
    }

    commands.trigger(RespawnPlayer);
}
