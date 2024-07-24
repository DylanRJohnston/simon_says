use bevy::{
    log::tracing_subscriber::filter::targets::IntoIter,
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

use crate::{
    actions::Action,
    delayed_command::DelayedCommand,
    player::{LevelCompleted, RespawnPlayer},
    ui::constants::BUTTON_SUCCESS_COLOR,
};

#[derive(Debug, Clone, Resource)]
pub struct Level {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub actions: Vec<Action>,
    pub action_limit: usize,
    pub command_challenge: Option<usize>,
    pub cycle_challenge: Option<usize>,
}

impl Level {
    pub fn get(&self, position: (i32, i32)) -> Option<&Tile> {
        self.tiles.get(&position)
    }
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub enum Tile {
    Start,
    Basic,
    Wall,
    Finish,
}

#[derive(Debug, Component)]
pub struct Start;

pub struct LevelBuilder(Level);

impl LevelBuilder {
    pub fn new() -> Self {
        Self(Level {
            tiles: HashMap::new(),
            actions: vec![
                Action::Forward,
                Action::Right,
                Action::Backward,
                Action::Left,
            ],
            action_limit: 1,
            command_challenge: None,
            cycle_challenge: None,
        })
    }

    pub fn block(mut self, lower: (i32, i32), upper: (i32, i32), tile: Tile) -> Self {
        for x in lower.0..=upper.0 {
            for y in lower.1..=upper.1 {
                self.0.tiles.insert((x, y), tile);
            }
        }

        self
    }

    pub fn insert(mut self, tiles: impl IntoIterator<Item = ((i32, i32), Tile)>) -> Self {
        for (position, tile) in tiles {
            self.0.tiles.insert(position, tile);
        }
        self
    }

    pub fn remove(mut self, tiles: impl IntoIterator<Item = (i32, i32)>) -> Self {
        for position in tiles {
            self.0.tiles.remove(&position);
        }
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.0.actions = Vec::from_iter(actions);
        self
    }

    pub fn action_limit(mut self, limit: usize) -> Self {
        self.0.action_limit = limit;
        self
    }

    pub fn build(self) -> Level {
        self.0
    }
}

impl Default for LevelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub static LEVELS: LazyLock<[Level; 7]> = LazyLock::new(|| {
    [
        LevelBuilder::new()
            .action_limit(3)
            .actions([Action::Forward])
            .block((-2, 0), (2, 0), Tile::Basic)
            .insert([((-2, 0), Tile::Start), ((2, 0), Tile::Finish)])
            .build(),
        LevelBuilder::new()
            .action_limit(3)
            .actions([Action::Forward, Action::Right])
            .block((-1, -1), (1, 1), Tile::Basic)
            .insert([((-1, -1), Tile::Start), ((1, 1), Tile::Finish)])
            .build(),
        LevelBuilder::new()
            .action_limit(3)
            .block((-1, -1), (1, 1), Tile::Basic)
            .insert([((-1, 1), Tile::Start), ((1, -1), Tile::Finish)])
            .remove([(1, 0)])
            .build(),
        LevelBuilder::new()
            .action_limit(3)
            .block((-1, -1), (1, 1), Tile::Basic)
            .insert([
                ((-1, -1), Tile::Start),
                ((1, 1), Tile::Finish),
                ((0, 0), Tile::Wall),
                ((0, 2), Tile::Wall),
                ((0, -2), Tile::Wall),
                ((2, 0), Tile::Wall),
                ((-2, 0), Tile::Wall),
            ])
            .build(),
        LevelBuilder::new()
            .action_limit(6)
            .block((-1, -1), (1, 1), Tile::Basic)
            .block((-3, -1), (3, -1), Tile::Basic)
            .block((-3, 1), (3, 1), Tile::Basic)
            .insert([
                ((0, 0), Tile::Wall),
                ((1, 2), Tile::Wall),
                ((1, -2), Tile::Wall),
                ((-2, -2), Tile::Wall),
                ((-2, 2), Tile::Wall),
                ((-3, -2), Tile::Start),
                ((-3, 2), Tile::Basic),
                ((3, -1), Tile::Finish),
                ((3, 1), Tile::Finish),
            ])
            .insert([])
            .build(),
        LevelBuilder::new()
            .action_limit(6)
            .block((-3, -2), (-1, -1), Tile::Basic)
            .block((-3, 1), (-1, 2), Tile::Basic)
            .block((-1, -1), (0, 1), Tile::Basic)
            // .block((2, -1), (2, 1), Tile::Wall)
            .block((1, 0), (2, 0), Tile::Basic)
            .insert([
                ((-3, 2), Tile::Start),
                ((3, 0), Tile::Finish),
                ((-1, 0), Tile::Wall),
                ((-1, -2), Tile::Wall),
                ((-1, 2), Tile::Wall),
            ])
            .build(),
        LevelBuilder::new()
            .action_limit(0)
            .insert([((0, 0), Tile::Start)])
            .actions([])
            .build(),
    ]
});

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, spawn_level)
            .observe(level_completed)
            .observe(load_next_level);
    }
}

#[derive(Debug, Resource, Deref)]
pub struct TileMesh(Handle<Mesh>);

#[derive(Debug, Resource)]
pub struct TileMaterials {
    pub basic: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
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
    let wall = materials.add(Color::srgb_u8(0x56, 0x6c, 0x86));

    commands.insert_resource(TileMaterials {
        basic,
        finish,
        wall,
    });

    commands.insert_resource(LEVELS[0].clone());

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

#[derive(Component)]
pub struct LevelRoot;

fn spawn_level(
    mut commands: Commands,
    level: Res<Level>,
    tile_mesh: Res<TileMesh>,
    tile_material: Res<TileMaterials>,
) {
    if !level.is_changed() {
        return;
    }

    commands
        .spawn((LevelRoot, SpatialBundle::INHERITED_IDENTITY))
        .with_children(|root| {
            for ((x, y), tile) in &level.tiles {
                let position = match tile {
                    Tile::Wall => Vec3::new(*x as f32, 0.2, *y as f32),
                    _ => Vec3::new(*x as f32, 0.0, *y as f32),
                };

                let mut entity = root.spawn((
                    *tile,
                    PbrBundle {
                        mesh: tile_mesh.clone(),
                        material: match tile {
                            Tile::Basic | Tile::Start => tile_material.basic.clone(),
                            Tile::Wall => tile_material.wall.clone(),
                            Tile::Finish => tile_material.finish.clone(),
                        },
                        transform: Transform {
                            translation: position - Vec3::Y * 10.0,
                            rotation: Quat::default(),
                            scale: match tile {
                                Tile::Wall => Vec3::ONE + Vec3::Y * 0.4,
                                _ => Vec3::ONE,
                            },
                        },
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

                if matches!(tile, Tile::Start) {
                    entity.insert(Start);
                }
            }
        });

    commands.trigger(RespawnPlayer);
}

#[derive(Debug, Event)]
pub struct GameFinished;

fn level_completed(
    _trigger: Trigger<LevelCompleted>,
    mut commands: Commands,
    level_root: Query<(Entity, &Children), With<LevelRoot>>,
    tiles: Query<&Transform, With<Tile>>,
) {
    let (level_root, children) = level_root.single();

    for tile in children {
        let transform = tiles.get(*tile).unwrap();

        commands.entity(*tile).insert(Animator::new(Tween::new(
            EaseFunction::CubicIn,
            Duration::from_secs_f32(1. + rand::random::<f32>()),
            TransformPositionLens {
                start: transform.translation,
                end: transform.translation - Vec3::Y * 10.0,
            },
        )));
    }

    commands.spawn(DelayedCommand::new(2., move |commands| {
        commands.entity(level_root).despawn_recursive();
        commands.trigger(LoadNextLevel);
    }));
}

#[derive(Debug, Event)]
pub struct LoadNextLevel;

fn load_next_level(
    _trigger: Trigger<LoadNextLevel>,
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut level_counter: Local<usize>,
) {
    *level_counter += 1;

    if let Some(next_level) = (*LEVELS).get(*level_counter) {
        *level = next_level.clone();
    } else {
        commands.trigger(GameFinished);
    }
}
