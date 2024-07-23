use std::f32::consts::PI;

use bevy::prelude::*;

use crate::player::Player;

pub struct EyesPlugin;

impl Plugin for EyesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_eye)
            .add_systems(Update, eye_track_player)
            .add_systems(Update, animate_eye_direction)
            .add_systems(Update, eye_emotion)
            .add_systems(Update, animate_emotion)
            .observe(emotion_from_player_activity);
    }
}

#[derive(Debug, Clone, Component)]
pub enum Emotion {
    Neutral(Timer),
    Bored(Timer),
    Surprised(Timer),
    Focused(Timer),
}

const BOREDOM_LOWER_BOUND: f32 = 2.5;
const BOREDOM_UPPER_BOUND: f32 = 5.0;

const NEUTRAL_LOWER_BOUND: f32 = 1.0;
const NEUTRAL_UPPER_BOUND: f32 = 2.5;

const SURPRISED_LOWER_BOUND: f32 = 0.5;
const SURPRISED_UPPER_BOUND: f32 = 0.5;

const FOCUSED_LOWER_BOUND: f32 = 2.5;
const FOCUSED_UPPER_BOUND: f32 = 5.0;

impl Emotion {
    fn neutral() -> Self {
        Self::Neutral(Timer::from_seconds(
            random_range(NEUTRAL_LOWER_BOUND, NEUTRAL_UPPER_BOUND),
            TimerMode::Once,
        ))
    }

    fn boredom() -> Self {
        Self::Bored(Timer::from_seconds(
            random_range(BOREDOM_LOWER_BOUND, BOREDOM_UPPER_BOUND),
            TimerMode::Once,
        ))
    }

    fn surprised() -> Self {
        Self::Surprised(Timer::from_seconds(
            random_range(SURPRISED_LOWER_BOUND, SURPRISED_UPPER_BOUND),
            TimerMode::Once,
        ))
    }

    fn focused() -> Self {
        Self::Focused(Timer::from_seconds(
            random_range(FOCUSED_LOWER_BOUND, FOCUSED_UPPER_BOUND),
            TimerMode::Once,
        ))
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct PlayerActivity;

impl Default for Emotion {
    fn default() -> Self {
        Emotion::focused()
    }
}

impl Emotion {
    pub fn target_scale(&self) -> f32 {
        match self {
            Self::Neutral(_) | Self::Bored(_) => 1.,
            Self::Surprised(_) => 1.3,
            Self::Focused(_) => 0.7,
        }
    }

    pub fn dilation(&self) -> f32 {
        match self {
            Self::Surprised(_) => 1.3,
            Self::Focused(_) => 0.7,
            _ => 1.0,
        }
    }

    pub fn emotion_speed(&self) -> f32 {
        match self {
            Self::Neutral(_) => 0.1,
            Self::Bored(_) => 1.0,
            Self::Surprised(_) => 4.0,
            Self::Focused(_) => 1.0,
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Eye {
    pub target: Vec3,
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
                Emotion::default(),
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
    mut commands: Commands,
) {
    if let Some(player) = players.iter().next() {
        for mut eye in &mut eye {
            commands.trigger(PlayerActivity);

            eye.target = player.translation;
        }
    }
}

fn animate_eye_direction(
    mut eye: Query<(&mut Transform, &Eye, &Emotion, &Children), Without<Iris>>,
    mut iris: Query<&mut Transform, With<Iris>>,
    time: Res<Time>,
) {
    for (mut transform, eye, emotion, children) in &mut eye {
        let mut iris = iris.get_mut(children[0]).unwrap();

        let base_rotation = Quat::from_euler(EulerRot::XYZ, 0., PI, 0.);

        let target = transform.looking_at(eye.target, Vec3::Y).rotation * base_rotation;
        let difference = (transform.rotation.conjugate() * target).xyz();

        iris.translation = iris.translation.lerp(
            IRIS_CENTER + 2. * Vec3::new(-difference.y, difference.x, 0.),
            4.0 * time.delta_seconds(),
        );

        if !matches!(emotion, Emotion::Neutral(_)) {
            let rotation = transform.rotation.lerp(
                transform.looking_at(eye.target, Vec3::Y).rotation * base_rotation,
                0.5 * time.delta_seconds(),
            );

            transform.rotation = rotation;
        }
    }
}

fn random_range(lower: f32, upper: f32) -> f32 {
    lower + rand::random::<f32>() * (upper - lower)
}

fn random_boredom_target() -> Vec3 {
    Vec3::new(
        rand::random::<f32>() * 10. - 5.,
        rand::random::<f32>() * 3. - 1.5,
        rand::random::<f32>() * 10. - 5.,
    )
}

fn eye_emotion(
    mut eye: Query<(&mut Eye, &mut Emotion)>,
    players: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut neutral_count: Local<usize>,
) {
    let player_position = players
        .iter()
        .next()
        .map(|it| it.translation)
        .unwrap_or(Vec3::ZERO);

    for (mut eye, mut emotion) in &mut eye {
        match emotion.as_mut() {
            Emotion::Neutral(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *neutral_count += 1;

                if *neutral_count >= 3 {
                    *neutral_count = 0;

                    *emotion = Emotion::boredom();
                } else {
                    eye.target = player_position + random_boredom_target() / 5.;
                    *emotion = Emotion::neutral();
                }
            }
            Emotion::Bored(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::boredom();
                eye.target = player_position + random_boredom_target();
            }
            Emotion::Surprised(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::focused();
            }
            Emotion::Focused(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                eye.target = player_position + random_boredom_target() / 3.;
                *emotion = Emotion::neutral();
            }
        }
    }
}

fn emotion_from_player_activity(_trigger: Trigger<PlayerActivity>, mut eye: Query<&mut Emotion>) {
    for mut emotion in &mut eye {
        match emotion.as_mut() {
            Emotion::Focused(timer) => timer.reset(),
            Emotion::Bored(_) | Emotion::Neutral(_) => *emotion = Emotion::focused(),
            _ => {}
        }
    }
}

fn animate_emotion(
    mut eyes: Query<(&mut Transform, &Children, &Emotion), With<Eye>>,
    mut iris: Query<&mut Transform, (With<Iris>, Without<Eye>)>,
    time: Res<Time>,
) {
    for (mut eye, children, emotion) in &mut eyes {
        let mut iris = iris.get_mut(children[0]).unwrap();

        eye.scale.y = eye.scale.y.lerp(
            emotion.target_scale(),
            time.delta_seconds() * emotion.emotion_speed(),
        );

        iris.scale.y = 1. / eye.scale.y;
        iris.scale = iris.scale.lerp(
            Vec3::ONE * emotion.dilation(),
            time.delta_seconds() * emotion.emotion_speed(),
        );
    }
}
