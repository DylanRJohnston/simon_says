use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    math::Affine2,
    prelude::*,
    utils::HashMap,
};
use bevy_firework::{
    bevy_utilitarian::prelude::{Gradient, ParamCurve, RandF32, RandValue, RandVec3},
    core::{BlendMode, ParticleSpawnerBundle, ParticleSpawnerSettings},
    emission_shape::EmissionShape,
};
use bevy_tweening::{
    asset_animator_system, lens::TransformPositionLens, Animator, AssetAnimator, EaseFunction,
    EaseMethod, Lens, RepeatCount, Tween,
};
use rand::Rng;
use std::{f32::consts::PI, sync::LazyLock, time::Duration};

use crate::{
    actions::{Action, CWRotation},
    assets::TextureAssets,
    delayed_command::{DelayedCommand, DelayedCommandExt},
    game_state::GameState,
    player::{LevelCompleted, SpawnPlayer},
    ui::{challenges::ChallengeState, constants::BUTTON_SUCCESS_COLOR, settings::GameMode},
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(OnExit(GameState::Loading), create_textures)
            .add_systems(Update, spawn_level.run_if(in_state(GameState::InGame)))
            .add_systems(Update, asset_animator_system::<StandardMaterial>)
            .observe(level_completed)
            .observe(load_next_level)
            .observe(despawn_level)
            .observe(load_level);
    }
}

#[derive(Debug, Clone)]
pub enum Scene {
    Start,
    Dialogue,
    Level(Level),
    Loop,
    Finish,
}

impl From<Level> for Scene {
    fn from(value: Level) -> Self {
        Self::Level(value)
    }
}

#[derive(Debug, Clone, Resource, Default)]
pub struct Level {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub name: &'static str,
    pub actions: Vec<Action>,
    pub action_limit: usize,
    pub command_challenge: Option<usize>,
    pub step_challenge: Option<usize>,
    pub waste_challenge: Option<usize>,
}

impl Level {
    pub fn get(&self, position: (i32, i32)) -> Option<&Tile> {
        self.tiles.get(&position)
    }

    pub fn builder() -> LevelBuilder {
        LevelBuilder::new()
    }
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub enum Tile {
    Start(CWRotation),
    Basic,
    Ice,
    Wall,
    CWRot,
    CCWRot,
    Finish,
}

impl Tile {
    pub const DEFAULT_START: Self = Tile::Start(CWRotation::Zero);

    pub fn rotation(self) -> CWRotation {
        match self {
            Tile::Start(rot) => rot,
            _ => {
                tracing::error!(tile =?self, "Tile::rotation called on non-rotatable tile");
                CWRotation::default()
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct Start;

pub struct LevelBuilder(Level);

impl LevelBuilder {
    pub fn new() -> Self {
        Self(Level {
            tiles: HashMap::new(),
            name: "Unnamed",
            actions: vec![
                Action::Forward,
                Action::Right,
                Action::Backward,
                Action::Left,
            ],
            action_limit: 1,
            command_challenge: None,
            step_challenge: None,
            waste_challenge: None,
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

    pub fn command_challenge(mut self, challenge: usize) -> Self {
        self.0.command_challenge = Some(challenge);
        self
    }

    pub fn step_challenge(mut self, challenge: usize) -> Self {
        self.0.step_challenge = Some(challenge);
        self
    }

    pub fn waste_challenge(mut self, challenge: usize) -> Self {
        self.0.waste_challenge = Some(challenge);
        self
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.0.name = name;
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

static DIALOGUE_LEVEL: LazyLock<Level> = LazyLock::new(|| {
    LevelBuilder::new()
        .action_limit(0)
        .insert([((0, 0), Tile::DEFAULT_START)])
        .actions([])
        .build()
});

pub static SCENES: LazyLock<Vec<Scene>> = LazyLock::new(|| {
    vec![
        LevelBuilder::new()
            .name("Lost")
            .action_limit(1)
            .actions([Action::Forward])
            .block((-2, 0), (2, 0), Tile::Basic)
            .insert([((-2, 0), Tile::DEFAULT_START), ((2, 0), Tile::Finish)])
            .insert(from_pictogram(&["🧑🟦🟦🟦🟩"]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Arbitrary")
            .action_limit(2)
            .actions([Action::Forward, Action::Right])
            .insert(from_pictogram(&[
                "🧑🟦🟦🟦",
                "🟦🟦🟦🟦",
                "🟦🟦🟦🟦",
                "🟦🟦🟦🟩",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Pothole")
            .action_limit(2)
            .insert(from_pictogram(&[
                "🟦🟦🟦🟩",
                "🟦🟦🟦⬛",
                "🟦🟦🟦🟦",
                "🧑🟦🟦🟦",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Noise")
            .action_limit(3)
            .insert(transform(
                ANTI_CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🟦🟦🟦🟦🟦",
                    "🟦⬛⬛🟦🟩",
                    "🟦⬛🟦🟦🟦",
                    "👩🟦🟦🟦🟦",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Obstructions")
            .action_limit(3)
            .command_challenge(2)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬛⬜⬛⬛",
                    "⬛🧑🟦🟦⬛",
                    "⬜🟦⬜🟦⬜",
                    "⬛🟦🟦🟩⬛",
                    "⬛⬛⬜⬛⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Choices")
            .action_limit(6)
            .command_challenge(3)
            .step_challenge(8)
            .waste_challenge(22)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🧑⬜⬛⬛⬜⬛⬛",
                    "🟦🟦🟦🟦🟦🟦🟩",
                    "⬛⬛🟦⬜🟦⬛⬛",
                    "🟦🟦🟦🟦🟦🟦🟩",
                    "🟦⬜⬛⬛⬜⬛⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Precarious")
            .action_limit(6)
            .command_challenge(4)
            .step_challenge(8)
            .waste_challenge(13)
            .waste_challenge(22)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🟦🟦⬜⬛⬛⬛⬛",
                    "🟦🟦🟦🟦⬛⬛⬛",
                    "⬛⬛⬜🟦🟦🟦🟩",
                    "🟦🟦🟦🟦⬛⬛⬛",
                    "🧑🟦⬜⬛⬛⬛⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Hook")
            .action_limit(5)
            .command_challenge(4)
            .step_challenge(7)
            .waste_challenge(14)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬛⬛⬛⬜⬛",
                    "⬜⬜🟦🟦🟦⬜",
                    "🧑🟦🟦⬜🟩🟦",
                    "⬛⬛⬜⬛⬛⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Crucible")
            .action_limit(8)
            .command_challenge(7)
            .step_challenge(14)
            .waste_challenge(23)
            .insert(from_pictogram(&[
                "⬛⬛🟦🟦🟦",
                "⬜🟦🟦⬜🟦",
                "🧑🟦⬜🟩🟦",
                "⬛⬜⬛⬛⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Rift")
            .action_limit(7)
            .command_challenge(6)
            .step_challenge(14)
            .waste_challenge(24)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "🟦⬜🟦⬜🟦⬜🟦",
                "🟦🟦🟦🟦🟦🟦🟦",
                "⬜🟦🧑⬛⬜🟦⬜",
                "🟦🟦🟦⬛🟦🟦🟦",
                "⬜🟦⬜⬛🟩🟦⬜",
                "🟦🟦🟦🟦🟦🟦🟦",
                "⬜🟦⬜🟦⬜🟦⬜",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Overshoot")
            .action_limit(4)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬛⬛🟦🟦",
                "🧑🏂🏂🟦",
                "⬛⬛🏂⬛",
                "⬛⬛🟩⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Glide")
            .action_limit(4)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬛⬜⬛⬛⬛",
                "⬛🏂🏂⬜⬛",
                "⬜🏂🏂🏂🧑",
                "⬛⬛🏂⬛⬛",
                "⬛⬛🟩⬛⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Loops")
            .action_limit(5)
            .command_challenge(4)
            .step_challenge(9)
            .waste_challenge(13)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬛⬛⬛🟦🏂🟦",
                "⬜🟦⬜🟦🟩🏂",
                "🟦🟦🏂🏂🏂🟦",
                "⬜🧑⬜⬛⬛⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Gauntlet")
            .action_limit(5)
            .command_challenge(5)
            .step_challenge(7)
            .waste_challenge(9)
            .insert(transform(
                ANTI_CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬜⬛⬛⬛⬛",
                    "⬛🏂🏂🏂⬜⬛",
                    "⬛🏂🏂🏂🏂⬛",
                    "⬜🏂🧑🏂🏂⬜",
                    "⬜🏂🏂🏂🏂⬛",
                    "⬛⬜🏂🏂⬜⬛",
                    "⬛🏂🏂🏂🏂⬛",
                    "⬜🏂🏂🏂🏂⬛",
                    "⬛🟩🏂⬜🏂⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Esky")
            .action_limit(5)
            .command_challenge(4)
            .step_challenge(8)
            .waste_challenge(15)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬜⬜⬜⬜⬜⬜⬜⬜",
                "⬜🧑🏂🏂🏂🏂🏂⬜",
                "⬜⬛🟦🟦🏂🏂🏂⬜",
                "⬜🟦🏂🏂🏂🟦🏂⬜",
                "⬜🏂🏂🟦🟩🏂🏂⬜",
                "⬜🟦🏂🟦🏂🟦🏂⬜",
                "⬜🏂🟦🏂🏂🏂🏂⬜",
                "⬜⬜⬜⬜⬜⬜⬜⬜",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Divert")
            .action_limit(1)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "🧑🟦🔃",
                "⬛⬛🟦",
                "⬛⬛🟩",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Pivot")
            .action_limit(2)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "🧑🔃🟩",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Twirl")
            .action_limit(4)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "🧑🔃🔃🔃🔃🔃🟩",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Dizzy")
            .action_limit(4)
            .insert(transform(
                CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬛⬛⬛🟩",
                    "⬛⬛⬛⬛🔃",
                    "⬛⬛⬛⬛🔃",
                    "⬛⬛⬛⬛🔃",
                    "🧑🔃🔃🔃🟦",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Zigzag")
            .action_limit(4)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🧑🔄🔄⬛⬛⬛",
                    "⬛⬛🔄⬛⬛⬛",
                    "⬛⬛🔄🔄🟩⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Alternating")
            .action_limit(4)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🧑🔄🔃🔄🔃🔄🔃🟩",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Chunks")
            .action_limit(4)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🧑🔄🔄🔃🔃🔄🔄🟩",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Chess")
            .action_limit(5)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "🔄🔃🔄🔃🔄🔃🟩",
                    "🔃🔄🔃🔄🔃🔄🔃",
                    "🔄🔃🔄🔃🔄🔃🔄",
                    "🧑🔄🔃🔄🔃🔄🔃",
                ]),
            ))
            .build()
            .into(),
        Scene::Start,
        LevelBuilder::new()
            .name("Progress")
            .actions([Action::Forward])
            .action_limit(1)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬛⬜",
                    "🟩🧑🔄",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Support")
            .action_limit(3)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬜",
                    "🧑🔄",
                    "⬛🟦",
                    "🟩🟦",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Trapped")
            .action_limit(4)
            .insert(transform(
                CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬜⬛⬛⬛",
                    "🧑🔄🟦⬛⬛",
                    "⬛⬜🟦🟦🟩",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Snail")
            .action_limit(3)
            .insert(transform(
                ANTI_CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬜",
                    "🧑🔄",
                    "🟦🟦",
                    "⬛🔃",
                    "⬛🟩",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Squeeze")
            .action_limit(6)
            .command_challenge(5)
            .step_challenge(11)
            .waste_challenge(23)
            .insert(transform(
                CLOCKWISE,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛🟩🟦🔄⬛",
                    "⬛⬜🟦⬜⬛",
                    "⬛🟦🔃🟦⬛",
                    "⬛🟦🔃🟦⬛",
                    "⬛🔄⬜🔄⬛",
                    "⬛🟦🟦🧑⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Popsicle")
            .action_limit(7)
            .command_challenge(5)
            .step_challenge(17)
            .waste_challenge(58)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬜⬜⬜⬛",
                    "⬜🔄🟦🔃⬜",
                    "⬜🟦🟦🔃⬜",
                    "⬜🔄🟦🔃⬜",
                    "⬜🟦⬛🟦⬜",
                    "⬜🔄⬛🔃⬜",
                    "⬛🧑⬛🟩⬛",
                ]),
            ))
            .build()
            .into(),
        // LevelBuilder::new()
        //     .name("Spinors")
        //     .action_limit(5)
        //     .command_challenge(2)
        //     .step_challenge(10)
        //     .waste_challenge(36)
        //     .insert(from_pictogram(&[
        //         #[rustfmt::ignore]
        //         "🔃🔃🔄🟩",
        //         "🔃🔃🔄🟦",
        //         "🔄🔄🔃🔄",
        //         "🔃🔄🔃🔄",
        //         "🟦🔄🔃🔃",
        //         "🧑🔃🔄🔃",
        //     ]))
        //     .build()
        //     .into(),
        LevelBuilder::new()
            .name("Swirl")
            .action_limit(4)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬛⬛⬛⬛⬛⬜⬛",
                    "⬛🧑🏂🔄🏂🔄⬛",
                    "⬛⬛🏂🏂🏂🏂⬛",
                    "⬛🟩🏂⬛🏂🔄⬛",
                    "⬛🏂🏂🏂🏂🏂⬛",
                    "⬛🔄🏂🔄🏂🔄⬜",
                    "⬛⬜⬛⬛⬛⬛⬛",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Blizzard")
            .action_limit(8)
            .command_challenge(3)
            .step_challenge(8)
            .waste_challenge(70)
            .insert(transform(
                IDENTITY,
                from_pictogram(&[
                    #[rustfmt::ignore]
                    "⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜",
                    "⬜🔄🏂🏂🏂🏂🏂🏂🏂🔄⬜",
                    "⬜🏂🔄🏂🏂🏂🏂🏂🟦🏂⬜",
                    "⬜🏂🏂🔃🏂🟦🔃🏂🏂🏂⬜",
                    "⬜🏂🏂🏂🧑⬜🟩🟦🏂🏂⬜",
                    "⬜🏂🏂🔃🏂🟦🔃🏂🏂🏂⬜",
                    "⬜🏂🟦🏂🏂🏂🏂🏂🏂🟦⬜",
                    "⬜🔄🏂🏂🔄⬜🏂🏂🔄🔄⬜",
                    "⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜",
                ]),
            ))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Mirror")
            .action_limit(1)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "🧑🟦🟦🟩",
                "⬛⬛⬛⬛",
                "🟩🟦🟦👩",
            ]))
            .build()
            .into(),
        // LevelBuilder::new()
        //     .name("Staggerd")
        //     .action_limit(1)
        //     .insert(from_pictogram(&[
        //         #[rustfmt::ignore]
        //         "🧑🟦🟦🟦🔃",
        //         "⬛⬛⬛⬛🟦",
        //         "⬛⬛⬛⬛🟩",
        //         "⬛⬛⬛⬛⬛",
        //         "⬜🟩🟦🟦👩",
        //     ]))
        //     .build()
        //     .into(),
        LevelBuilder::new()
            .name("Convergence")
            .action_limit(6)
            .command_challenge(4)
            .step_challenge(11)
            .waste_challenge(31)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬜⬜⬛⬛⬛⬛",
                "🧑⬜⬜⬜⬛⬛",
                "🟦🟦🟦🟦⬜⬛",
                "🟦🟦🟦🟦🟦🟩",
                "🟦🟦🟦🟦⬜⬛",
                "🧑⬜⬜⬜⬛⬛",
                "⬜⬛⬛⬛⬛⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Perpendicular")
            .action_limit(6)
            .command_challenge(4)
            .step_challenge(7)
            .waste_challenge(15)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬜⬜⬜⬜⬜⬜⬜⬜⬜",
                "⬜🟩🏂🏂⬜🏂🏂🟩⬜",
                "⬜🏂🏂🏂🏂🏂⬜🏂⬜",
                "⬜🏂⬜🏂🏂🏂🏂🏂⬜",
                "⬜🧑🏂🏂⬜🏂🏂🏂⬜",
                "⬜🏂⬜🏂🏂🏂⬜🏂⬜",
                "⬜🏂🏂⬜🏂🏂🏂🏂⬜",
                "⬜⬜🏂🏂🧓🏂🏂⬜⬜",
                "⬜⬜⬜⬜⬜⬜⬜⬜⬜",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Samsara")
            .action_limit(6)
            .command_challenge(3)
            .step_challenge(11)
            .waste_challenge(55)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬛⬜⬛⬛⬜⬛⬜⬛⬛⬜⬛",
                "⬜🟩🏂🏂🔃🏂🔃🏂🏂🔄⬜",
                "⬛🏂🏂🏂🏂🏂🏂🏂🏂🏂⬛",
                "⬛🏂🏂🏂🔄🏂🏂🏂🏂🏂⬛",
                "⬜🔃🏂🏂🧓🏂🧑🔄🏂🔄⬜",
                "⬛🏂🏂🏂🏂⬛🏂🏂🏂🏂⬛",
                "⬜🔃🏂🔄👩🏂🧒🏂🏂🔄⬜",
                "⬛🏂🏂🏂🏂🏂🔄🏂🏂🏂⬛",
                "⬛🏂🏂🏂🏂🏂🏂🏂🏂🏂⬛",
                "⬜🏂🏂🏂🔄🏂🔄🏂🏂🟩⬜",
                "⬛⬜⬛⬛⬜⬛⬜⬛⬛⬜⬛",
            ]))
            .build()
            .into(),
        LevelBuilder::new()
            .name("Transcendence")
            .action_limit(6)
            .command_challenge(6)
            .step_challenge(18)
            .waste_challenge(31)
            .insert(from_pictogram(&[
                #[rustfmt::ignore]
                "⬛⬛⬛⬛⬛⬛⬛🧒⬛⬛⬛⬛⬛⬛⬛",
                "⬛⬛⬛⬛⬛⬛🧒🟦🧒⬛⬛⬛⬛⬛⬛",
                "⬛⬛⬛⬛⬛🧒🟦🟦🟦🧒⬛⬛⬛⬛⬛",
                "⬛⬛⬛⬜🧒🟦🟦🟦🟦🟦🧒⬜⬛⬛⬛",
                "⬛⬛⬛🧑⬜🟦🟦🟦🟦🟦⬜👩⬛⬛⬛",
                "⬛⬛🧑🟦🟦⬜🟦🟦🟦⬜🟦🟦👩⬛⬛",
                "⬛🧑🟦🟦🟦🟦⬜🟦⬜🟦🟦🟦🟦👩⬛",
                "🧑🟦🟦🟦🟦🟦🟦🟩🟦🟦🟦🟦🟦🟦👩",
                "⬛🧑🟦🟦🟦🟦⬜🟦⬜🟦🟦🟦🟦👩⬛",
                "⬛⬛🧑🟦🟦⬜🟦🟦🟦⬜🟦🟦👩⬛⬛",
                "⬛⬛⬛🧑⬜🟦🟦🟦🟦🟦⬜👩⬛⬛⬛",
                "⬛⬛⬛⬜🧓🟦🟦🟦🟦🟦🧓⬜⬛⬛⬛",
                "⬛⬛⬛⬛⬛🧓🟦🟦🟦🧓⬛⬛⬛⬛⬛",
                "⬛⬛⬛⬛⬛⬛🧓🟦🧓⬛⬛⬛⬛⬛⬛",
                "⬛⬛⬛⬛⬛⬛⬛🧓⬛⬛⬛⬛⬛⬛⬛",
            ]))
            .build()
            .into(),
    ]
});

const ANTI_CLOCKWISE: ((i32, i32), (i32, i32)) = ((0, 1), (1, 0));
const CLOCKWISE: ((i32, i32), (i32, i32)) = ((0, -1), (1, 0));
const IDENTITY: ((i32, i32), (i32, i32)) = ((1, 0), (0, 1));

fn transform(
    rot: ((i32, i32), (i32, i32)),
    iter: impl IntoIterator<Item = ((i32, i32), Tile)>,
) -> impl IntoIterator<Item = ((i32, i32), Tile)> {
    iter.into_iter()
        .map(|((x, y), tile)| {
            (
                (x * rot.0 .0 + y * rot.0 .1, x * rot.1 .0 + y * rot.1 .1),
                tile,
            )
        })
        .collect::<Vec<_>>()
}

fn from_pictogram(lines: &[&str]) -> impl IntoIterator<Item = ((i32, i32), Tile)> {
    let width = lines[0].chars().count() as i32;
    let length = lines.len() as i32;

    lines
        .iter()
        .flat_map(|line| line.chars())
        .enumerate()
        .filter_map(|(index, c)| {
            let index = index as i32;
            let coords = (index % width - width / 2, index / width - length / 2);

            match c {
                '⬛' => None,
                '🟩' => Some((coords, Tile::Finish)),
                '⬜' => Some((coords, Tile::Wall)),
                '🟦' => Some((coords, Tile::Basic)),
                '🏂' => Some((coords, Tile::Ice)),
                '🧑' => Some((coords, Tile::Start(CWRotation::Zero))),
                '🧒' => Some((coords, Tile::Start(CWRotation::Ninety))),
                '👩' => Some((coords, Tile::Start(CWRotation::OneEighty))),
                '🧓' => Some((coords, Tile::Start(CWRotation::TwoSeventy))),
                '🔃' => Some((coords, Tile::CWRot)),
                '🔄' => Some((coords, Tile::CCWRot)),
                other => {
                    tracing::warn!(?other, "unrecognised pictogram");
                    None
                }
            }
        })
        .collect::<Vec<_>>()
}

#[derive(Debug, Resource, Deref)]
pub struct TileMesh(Handle<Mesh>);

#[derive(Debug, Resource)]
pub struct TileMaterials {
    pub basic: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub finish: Handle<StandardMaterial>,
    pub ice: Handle<StandardMaterial>,
    pub cw_rot: Handle<StandardMaterial>,
    pub ccw_rot: Handle<StandardMaterial>,
}

#[derive(Debug, Default)]
pub struct RotationLens {
    pub ccw: bool,
}

impl Lens<StandardMaterial> for RotationLens {
    fn lerp(&mut self, target: &mut dyn bevy_tweening::Targetable<StandardMaterial>, ratio: f32) {
        let multiplier = if self.ccw { 1.0 } else { -1.0 };

        let reflection = if self.ccw {
            Affine2::from_mat2(Mat2::from_cols_array_2d(&[[-1., 0.], [0., 1.]]))
        } else {
            Affine2::IDENTITY
        };

        target.uv_transform = Affine2::from_translation(Vec2::new(0.5, 0.5))
            * reflection
            * Affine2::from_angle(multiplier * ratio * std::f32::consts::TAU)
            * Affine2::from_translation(Vec2::new(-0.5, -0.5))
    }
}

fn create_textures(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    textures: Res<TextureAssets>,
) {
    let tile_handle = meshes.add(
        Mesh::from(Cuboid::new(0.95, 0.95, 0.95))
            .with_generated_tangents()
            .unwrap(),
    );
    commands.insert_resource(TileMesh(tile_handle));

    let basic = materials.add(Color::srgb_u8(0x3b, 0x5d, 0xc9));
    let finish = materials.add(*BUTTON_SUCCESS_COLOR);
    let wall = materials.add(Color::srgb_u8(0x56, 0x6c, 0x86));
    let ice = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x73, 0xef, 0xf7),
        perceptual_roughness: 0.2,
        diffuse_transmission: 0.0,
        specular_transmission: 0.65,
        thickness: 0.1,
        ior: 1.31,
        clearcoat_perceptual_roughness: 0.5,
        // normal_map_texture: Some(textures.ice_normal.clone()),
        // occlusion_texture: Some(textures.ice_occlusion.clone()),
        ..default()
    });

    let cw_rot = materials.add(StandardMaterial {
        base_color_texture: Some(textures.cw_rot.clone()),
        base_color: Color::srgb_u8(0xff, 0x7c, 0x5D),
        ..default()
    });

    let ccw_rot = materials.add(StandardMaterial {
        base_color_texture: Some(textures.ccw_rot.clone()),
        base_color: Color::srgb_u8(0xff, 0x7c, 0x5D),
        ..default()
    });

    commands.insert_resource(TileMaterials {
        basic,
        finish,
        wall,
        ice,
        cw_rot,
        ccw_rot,
    });
}

fn setup(mut commands: Commands) {
    let start = (SCENES
        .iter()
        .enumerate()
        .find(|(_, scene)| matches!(scene, Scene::Start))
        .map(|(i, _)| i as i32)
        .unwrap_or(-1)
        + 1) as usize;

    commands.insert_resource(LevelCounter(start));

    match &SCENES[start] {
        Scene::Level(level) => commands.insert_resource(level.clone()),
        Scene::Dialogue => {
            commands.insert_resource(DIALOGUE_LEVEL.clone());
        }
        other => panic!(
            "the scene after the start scene must be a level, found {:?}",
            other
        ),
    }

    let mut settings = ParticleSpawnerSettings {
        one_shot: false,
        rate: 80.0,
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
    };

    commands.spawn(ParticleSpawnerBundle::from_settings(settings.clone()));

    settings.one_shot = true;
    settings.rate = 800.0;
    settings.lifetime = RandF32 { min: 0., max: 10. };
    commands.spawn(ParticleSpawnerBundle::from_settings(settings));
}

#[derive(Component)]
pub struct LevelRoot;

fn spawn_level(
    mut commands: Commands,
    level: Res<Level>,
    tile_mesh: Res<TileMesh>,
    tile_material: Res<TileMaterials>,
    current_scene: CurrentScene,
) {
    if !level.is_changed() {
        return;
    }

    if let Scene::Dialogue = current_scene.current() {
        commands.delayed(7.5, |commands| {
            commands.trigger(LevelCompleted);
        });
    }

    // This ensures there's only one animator
    let mut cw_rot_animator = Some(AssetAnimator::new(
        Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(3.0),
            RotationLens::default(),
        )
        .with_repeat_strategy(bevy_tweening::RepeatStrategy::Repeat)
        .with_repeat_count(RepeatCount::Infinite),
    ));

    let mut ccw_rot_animator = Some(AssetAnimator::new(
        Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(3.0),
            RotationLens { ccw: true },
        )
        .with_repeat_strategy(bevy_tweening::RepeatStrategy::Repeat)
        .with_repeat_count(RepeatCount::Infinite),
    ));

    commands
        .spawn((LevelRoot, SpatialBundle::INHERITED_IDENTITY))
        .with_children(|root| {
            for ((x, y), tile) in &level.tiles {
                let position = match tile {
                    Tile::Wall => Vec3::new(*x as f32, 0.2, *y as f32),
                    _ => Vec3::new(*x as f32, 0.0, *y as f32),
                };

                let mut entity = root.spawn((
                    Name::from(format!("{tile:?}")),
                    *tile,
                    PbrBundle {
                        mesh: tile_mesh.clone(),
                        material: match tile {
                            Tile::Basic | Tile::Start(_) => tile_material.basic.clone(),
                            Tile::Wall => tile_material.wall.clone(),
                            Tile::Finish => tile_material.finish.clone(),
                            Tile::Ice => tile_material.ice.clone(),
                            Tile::CWRot => tile_material.cw_rot.clone(),
                            Tile::CCWRot => tile_material.ccw_rot.clone(),
                        },
                        transform: Transform {
                            translation: position - Vec3::Y * 20.0,
                            rotation: match tile {
                                // Tile::Ice => Quat::from_rotation_y(
                                //     rand::thread_rng().gen_range(0..4) as f32
                                //         * std::f32::consts::FRAC_PI_2,
                                // ),
                                _ => Quat::from_rotation_y(std::f32::consts::PI),
                            },
                            scale: match tile {
                                Tile::Wall => Vec3::ONE + Vec3::Y * 0.4,
                                _ => Vec3::ONE,
                            },
                        },
                        ..default()
                    },
                    Animator::new(Tween::new(
                        EaseFunction::QuadraticOut,
                        Duration::from_secs_f32(1. + rand::random::<f32>()),
                        TransformPositionLens {
                            start: position - Vec3::Y * 10.0,
                            end: position,
                        },
                    )),
                ));

                if let Tile::CWRot = tile {
                    if let Some(animator) = cw_rot_animator.take() {
                        entity.insert(animator);
                    }
                }

                if let Tile::CCWRot = tile {
                    if let Some(animator) = ccw_rot_animator.take() {
                        entity.insert(animator);
                    }
                }

                if matches!(tile, Tile::Start(_)) {
                    entity.insert(Start);
                }
            }
        });

    commands.trigger(SpawnPlayer);
}

#[derive(Debug, Event)]
pub struct GameFinished;

fn level_completed(_trigger: Trigger<LevelCompleted>, mut commands: Commands) {
    commands.trigger(DespawnLevel);

    commands.spawn(DelayedCommand::new(2., move |commands| {
        commands.trigger(LoadNextLevel);
    }));
}

#[derive(Debug, Event)]
pub struct DespawnLevel;

fn despawn_level(
    _trigger: Trigger<DespawnLevel>,
    mut commands: Commands,
    level_root: Query<(Entity, &Children), With<LevelRoot>>,
    tiles: Query<&Transform, With<Tile>>,
) {
    for (level_root, children) in &level_root {
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
            commands
                .get_entity(level_root)
                .map(EntityCommands::despawn_recursive);
        }));
    }
}

#[derive(Debug, Event)]
pub struct LoadNextLevel;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct LevelCounter(usize);

fn load_next_level(
    _trigger: Trigger<LoadNextLevel>,
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut level_counter: ResMut<LevelCounter>,
    challenges: Res<ChallengeState>,
    game_mode: Res<GameMode>,
) {
    if *game_mode == GameMode::Challenge {
        let challenges = challenges.get(level.name).unwrap();

        match (challenges.commands, challenges.steps, challenges.waste) {
            (Some(false), _, _) | (_, Some(false), _) | (_, _, Some(false)) => {
                *level = level.clone();
                return;
            }
            _ => {}
        }
    }

    **level_counter += 1;

    match (*SCENES).get(**level_counter) {
        Some(Scene::Level(next_level)) => {
            *level = next_level.clone();
        }
        Some(Scene::Loop) => {
            **level_counter -= 2;
            commands.trigger(LoadNextLevel);
        }
        Some(Scene::Start) => {
            commands.trigger(LoadNextLevel);
        }
        Some(Scene::Finish) | None => {
            *level = LevelBuilder::new()
                .action_limit(0)
                .insert([((0, 0), Tile::DEFAULT_START)])
                .actions([])
                .build();
            commands.trigger(GameFinished);
        }
        Some(Scene::Dialogue) => {
            *level = DIALOGUE_LEVEL.clone();

            commands.delayed(15., |commands| commands.trigger(LevelCompleted));
        }
    }
}

#[derive(SystemParam)]
pub struct CurrentScene<'w> {
    level_counter: Res<'w, LevelCounter>,
}

impl CurrentScene<'_> {
    pub fn current(&self) -> Scene {
        match (*SCENES).get(self.level_counter.0) {
            Some(scene) => scene.clone(),
            None => Scene::Finish,
        }
    }
}

#[derive(Debug, Event)]
pub struct LoadLevel(pub usize);

fn load_level(
    trigger: Trigger<LoadLevel>,
    mut level_counter: ResMut<LevelCounter>,
    mut level: ResMut<Level>,
) {
    let id = trigger.event().0;

    **level_counter = id;
    match (*SCENES).get(id) {
        Some(Scene::Level(next_level)) => {
            *level = next_level.clone();
        }
        other => tracing::warn!(?other, "tried to force loading on non level scene"),
    }
}
