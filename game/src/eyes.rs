use std::f32::consts::PI;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    assets::TextureAssets, game_state::GameState, player::Player, ui::dialogue::DialogueStarted,
};

pub struct EyesPlugin;

impl Plugin for EyesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), spawn_eye)
            .add_systems(Update, eye_track_player)
            .add_systems(Update, animate_eye_direction)
            .add_systems(Update, eye_emotion)
            .add_systems(Update, animate_emotion)
            .add_systems(
                Update,
                animate_talking.run_if(|state: Res<State<GameState>>| {
                    !matches!(state.get(), GameState::Loading)
                }),
            )
            .observe(emotion_from_player_activity)
            .observe(trigger_talking_animation);
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

const IRIS_CENTER: Vec3 = Vec3::new(-0.31, -0.28, 0.1);

#[derive(Debug, Component, Default)]
pub struct Iris;

#[derive(Debug, Resource)]
struct IrisMaterialHandle(Handle<StandardMaterial>);

#[derive(Debug, Resource)]
struct EyeMaterialHandle(Handle<StandardMaterial>);

#[derive(SystemParam)]
struct EyeMaterial<'w> {
    eye_handle: Res<'w, EyeMaterialHandle>,
    iris_handle: Res<'w, IrisMaterialHandle>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

impl<'w> EyeMaterial<'w> {
    // This is safe because the handles are disjoin and the borrow is transmuted
    // to the 'w lifetime ensuring it doesn't escape the system
    fn eye_material(&mut self) -> &'w mut StandardMaterial {
        unsafe { std::mem::transmute(self.materials.get_mut(&self.eye_handle.0).unwrap()) }
    }

    fn iris_material(&mut self) -> &'w mut StandardMaterial {
        unsafe { std::mem::transmute(self.materials.get_mut(&self.iris_handle.0).unwrap()) }
    }
}

fn spawn_eye(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
) {
    let quad_handle = meshes.add(Rectangle::new(8., 8.));

    let eye_material = material.add(StandardMaterial {
        base_color_texture: Some(textures.eye.clone()),
        base_color: Color::BLACK,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let iris_material = material.add(StandardMaterial {
        base_color_texture: Some(textures.iris.clone()),
        base_color: Color::BLACK,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.insert_resource(EyeMaterialHandle(eye_material.clone()));
    commands.insert_resource(IrisMaterialHandle(iris_material.clone()));

    for (x, y, z) in [(3.0, 2.0, -6.0), (9.0, -1.0, -3.0), (-3.0, 1.0, -6.0)] {
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

        iris.translation.x *= transform.scale.x;
        iris.translation.y *= transform.scale.y;

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

fn random_boredom_target(position: Vec3, wander: f32) -> Vec3 {
    (position
        + wander
            * Vec3::new(
                rand::random::<f32>() * 10. - 5.,
                rand::random::<f32>() * 3. - 1.5,
                rand::random::<f32>() * 10. - 5.,
            ))
    .clamp(Vec3::new(-4., -1., -4.), Vec3::new(4., 2., 4.))
}

fn eye_emotion(
    mut eye: Query<(&mut Eye, &mut Emotion)>,
    players: Query<&Transform, With<Player>>,
    camera: Query<&Transform, With<Camera>>,
    time: Res<Time>,
    mut neutral_count: Local<usize>,
) {
    let player_position = players
        .iter()
        .next()
        .map(|it| it.translation)
        .unwrap_or_else(|| {
            camera
                .get_single()
                .map(|it| it.translation)
                .unwrap_or(Vec3::ZERO)
        });

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
                    eye.target = random_boredom_target(eye.target, 0.2);
                    *emotion = Emotion::neutral();
                }
            }
            Emotion::Bored(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::boredom();
                eye.target = random_boredom_target(eye.target, 1.);
            }
            Emotion::Surprised(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::focused();
            }
            Emotion::Focused(timer) => {
                eye.target = player_position;

                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

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

#[derive(Debug, Resource, Deref, DerefMut)]
struct TalkingTimer(Timer);

#[derive(Debug, Resource, Deref, DerefMut)]
struct ChangeColorTimer(Timer);

fn trigger_talking_animation(
    _trigger: Trigger<DialogueStarted>,
    mut commands: Commands,
    mut eye: Query<&mut Emotion>,
) {
    commands.insert_resource(TalkingTimer(Timer::from_seconds(3.5, TimerMode::Once)));
    commands.insert_resource(ChangeColorTimer(Timer::from_seconds(0., TimerMode::Once)));
    for mut emotion in &mut eye {
        match emotion.as_mut() {
            Emotion::Focused(timer) => timer.reset(),
            Emotion::Bored(_) | Emotion::Neutral(_) => *emotion = Emotion::focused(),
            _ => {}
        }
    }
}

fn animate_talking(
    talking_timer: Option<ResMut<TalkingTimer>>,
    change_color_timer: Option<ResMut<ChangeColorTimer>>,
    mut eye_material: EyeMaterial,
    time: Res<Time>,
) {
    if talking_timer.is_none() || change_color_timer.is_none() {
        return;
    }

    let mut talking_timer = talking_timer.unwrap();
    let mut change_color_timer = change_color_timer.unwrap();

    if talking_timer.0.finished() {
        return;
    }

    let eye = eye_material.eye_material();
    let iris = eye_material.iris_material();

    if talking_timer.tick(time.delta()).just_finished() {
        eye.base_color = Color::BLACK;
        iris.base_color = Color::BLACK;
        return;
    }

    if change_color_timer.tick(time.delta()).just_finished() {
        // eye_material.base_color =
        //     Color::hsv(talking_timer.remaining_secs() * 360. * 3. % 360., 1.0, 1.0);
        let color = Color::hsv(rand::random::<f32>() * 360.0, 1.0, 1.0);
        eye.base_color = color;
        iris.base_color = Color::hsv(rand::random::<f32>() * 360.0, 1.0, 1.0);

        *change_color_timer = ChangeColorTimer(Timer::from_seconds(0.05, TimerMode::Once));
    }
}
