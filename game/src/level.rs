use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_firework::{
    bevy_utilitarian::prelude::{Gradient, ParamCurve, RandF32, RandValue, RandVec3},
    core::{BlendMode, ParticleSpawnerBundle, ParticleSpawnerSettings},
    emission_shape::EmissionShape,
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};
use std::{f32::consts::PI, sync::LazyLock, time::Duration};

use crate::{actions::Action, player::RespawnPlayer, ui::constants::BUTTON_SUCCESS_COLOR};

#[derive(Debug, Clone, Resource)]
pub struct Level {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub actions: HashSet<Action>,
    pub action_limit: usize,
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
        actions: HashSet::from([Action::Forward]),
        action_limit: 1,
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

    commands.spawn((ParticleSpawnerBundle::from_settings(
        ParticleSpawnerSettings {
            one_shot: false,
            rate: 40.0,
            emission_shape: EmissionShape::HollowSphere {
                inner_radius: 5.0,
                outer_radius: 8.0,
            },
            lifetime: RandF32::constant(20.),
            inherit_parent_velocity: true,
            initial_velocity: RandVec3 {
                magnitude: RandF32 {
                    min: 0.08,
                    max: 0.12,
                },
                direction: Vec3::Y,
                spread: 2. * PI,
            },
            initial_scale: RandF32 {
                min: 0.05,
                max: 0.05,
            },
            scale_curve: ParamCurve::linear(vec![(0., 0.5), (0.5, 1.0), (1.0, 0.5)]),
            color: Gradient::linear(vec![
                (0.0, LinearRgba::new(0., 0., 0., 0.0)),
                (0.1, LinearRgba::new(0., 0., 0., 0.85)),
                (0.9, LinearRgba::new(0., 0., 0., 0.85)),
                (1.0, LinearRgba::new(0., 0., 0., 0.0)),
            ]),
            blend_mode: BlendMode::Multiply,
            linear_drag: 0.1,
            pbr: false,
            acceleration: Vec3::ZERO,
            fade_edge: 1.0,
            ..default()
        },
    ),));
}
