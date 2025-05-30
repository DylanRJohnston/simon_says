use std::f32::consts::PI;

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::{
    assets::TextureAssets, game_state::GameState, player::Player, ui::dialogue::DialogueStarted,
};

pub struct EyesPlugin;

#[derive(Debug, Resource, Deref, DerefMut)]
struct EyesRandom(SmallRng);

impl Plugin for EyesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EyesRandom(SmallRng::seed_from_u64(12398712837183)))
            .add_systems(OnExit(GameState::Loading), spawn_eye)
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
            .add_observer(emotion_from_player_activity)
            .add_observer(trigger_talking_animation);
    }
}

#[derive(Debug, Clone)]
pub enum FocusTarget {
    Camera,
    Player,
}

#[derive(Debug, Clone, Component)]
pub enum Emotion {
    Neutral(Timer),
    Bored(Timer),
    Surprised(Timer),
    Focused { timer: Timer, target: FocusTarget },
}

const BOREDOM_LOWER_BOUND: f32 = 2.5;
const BOREDOM_UPPER_BOUND: f32 = 5.0;

const NEUTRAL_LOWER_BOUND: f32 = 1.0;
const NEUTRAL_UPPER_BOUND: f32 = 2.5;

const FOCUSED_LOWER_BOUND: f32 = 2.5;
const FOCUSED_UPPER_BOUND: f32 = 5.0;

impl Emotion {
    fn neutral(rand: &mut SmallRng) -> Self {
        Self::Neutral(Timer::from_seconds(
            random_range(rand, NEUTRAL_LOWER_BOUND, NEUTRAL_UPPER_BOUND),
            TimerMode::Once,
        ))
    }

    fn boredom(rand: &mut SmallRng) -> Self {
        Self::Bored(Timer::from_seconds(
            random_range(rand, BOREDOM_LOWER_BOUND, BOREDOM_UPPER_BOUND),
            TimerMode::Once,
        ))
    }

    fn focused(rand: &mut SmallRng) -> Self {
        Self::Focused {
            timer: Timer::from_seconds(
                random_range(rand, FOCUSED_LOWER_BOUND, FOCUSED_UPPER_BOUND),
                TimerMode::Once,
            ),
            target: FocusTarget::Player,
        }
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct PlayerActivity;

impl Emotion {
    pub fn target_scale(&self) -> f32 {
        match self {
            Self::Neutral(_) | Self::Bored(_) => 1.,
            Self::Surprised(_) => 1.3,
            Self::Focused { .. } => 0.7,
        }
    }

    pub fn dilation(&self) -> f32 {
        match self {
            Self::Surprised(_) => 1.3,
            Self::Focused { .. } => 0.7,
            _ => 1.0,
        }
    }

    pub fn emotion_speed(&self) -> f32 {
        match self {
            Self::Neutral(_) => 0.1,
            Self::Bored(_) => 1.0,
            Self::Surprised(_) => 4.0,
            Self::Focused { .. } => 1.0,
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Eye {
    pub target: Vec3,
}

#[derive(Debug, Component)]
pub struct Iris;

#[derive(Debug, Resource, Deref)]
struct IrisMaterialHandle(Handle<StandardMaterial>);

#[derive(Debug, Resource, Deref)]
struct EyeMaterialHandle(Handle<StandardMaterial>);

#[derive(SystemParam)]
struct EyeMaterial<'w> {
    eye_handle: Res<'w, EyeMaterialHandle>,
    iris_handle: Res<'w, IrisMaterialHandle>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

impl EyeMaterial<'_> {
    fn get_materials(&mut self) -> (&mut StandardMaterial, &mut StandardMaterial) {
        debug_assert_ne!(
            self.eye_handle.0, self.iris_handle.0,
            "Eye and iris material handles must be different"
        );

        // Get both materials at once
        let eye = self.materials.get_mut(&self.eye_handle.0).unwrap() as *mut _;
        let iris = self.materials.get_mut(&self.iris_handle.0).unwrap() as *mut _;

        // This is safe because the handles are disjoint
        // Return both at once - ensures they're only obtained once
        unsafe { (&mut *eye, &mut *iris) }
    }
}

fn spawn_eye(
    textures: Res<TextureAssets>,
    mut commands: Commands,
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
        // let transform =
        //     Vec3::new(x, y, z) + (Vec3::new(x, y, z) - camera.translation).normalize() * 5.0;

        commands.spawn((
            Name::from("Eye"),
            Eye::default(),
            Emotion::focused(&mut SmallRng::seed_from_u64(12398712837183)),
            Mesh3d(quad_handle.clone()),
            MeshMaterial3d(eye_material.clone()),
            Transform::from_xyz(x, y, z),
            children![(
                Iris,
                Mesh3d(quad_handle.clone()),
                MeshMaterial3d(iris_material.clone()),
            )],
        ));
    }
}

fn eye_track_player(
    players: Query<&Transform, (Changed<Transform>, With<Player>)>,
    mut commands: Commands,
) {
    if players.iter().next().is_some() {
        commands.trigger(PlayerActivity);
    }
}

fn animate_eye_direction(
    mut eye: Query<(&mut Transform, &Eye, &Emotion, &Children), Without<Iris>>,
    mut iris: Query<&mut Transform, With<Iris>>,
    time: Res<Time>,
    #[cfg(feature = "debug")] mut gizmos: Gizmos,
) {
    let base_rotation = Quat::from_euler(EulerRot::XYZ, 0., PI, 0.);

    for (mut transform, eye, emotion, children) in &mut eye {
        let eye_origin = transform.translation - transform.rotation * Vec3::Z;

        #[cfg(feature = "debug")]
        {
            // Focal point of the eye
            gizmos.cross(eye_origin, 0.1, SKY_BLUE);

            // Line to visual target
            #[cfg(feature = "debug")]
            gizmos.line(eye_origin, eye.target, RED);
            gizmos.cross(Isometry3d::from_translation(eye.target), 0.1, RED);

            gizmos.line(
                eye_origin,
                transform.translation
                    + transform.rotation * Vec3::Z * (eye.target - transform.translation).length(),
                GREEN,
            );
        }

        let mut iris = iris.get_mut(children[0]).unwrap();

        // The iris lerps to look at the target much faster, but instead of rotating
        // it moves up and down / side to side based on the intersection of the ray cast one unit beind the eye and the plane of the eye in that direction

        let iris_rotation = Transform::from_translation(eye_origin)
            .looking_at(iris.translation + transform.translation, transform.up())
            .rotation;

        let target_rotation = Transform::from_translation(eye_origin)
            .looking_at(eye.target, transform.up())
            .rotation;

        let new_rotation = iris_rotation
            .lerp(target_rotation, f32::max(4.0 * time.delta_secs(), 0.01))
            .normalize();

        iris.translation = eye_origin - new_rotation * Vec3::Z - transform.translation;
        // iris.translation.z = 0.;

        #[cfg(feature = "debug")]
        {
            gizmos.cross(iris.translation + transform.translation, 0.1, ORANGE);
            gizmos.line(
                eye_origin,
                eye_origin
                    + (iris.translation + transform.translation - eye_origin).normalize()
                        * (eye.target - eye_origin).length(),
                ORANGE,
            );
        }

        if !matches!(emotion, Emotion::Neutral { .. }) {
            let rotation = transform.rotation.lerp(
                transform.looking_at(eye.target, Vec3::Y).rotation * base_rotation,
                0.5 * time.delta_secs(),
            );

            transform.rotation = rotation;
        }
    }
}

fn random_range(rand: &mut SmallRng, lower: f32, upper: f32) -> f32 {
    lower + rand.random::<f32>() * (upper - lower)
}

fn random_boredom_target(rand: &mut SmallRng, position: Vec3, wander: f32) -> Vec3 {
    (position
        + wander
            * Vec3::new(
                rand.random::<f32>() * 10. - 5.,
                rand.random::<f32>() * 3. - 1.5,
                rand.random::<f32>() * 10. - 5.,
            ))
    .clamp(Vec3::new(-8., -2., -8.), Vec3::new(8., 3., 8.))
}

fn eye_emotion(
    mut eye: Query<(&mut Eye, &mut Emotion)>,
    players: Query<&Transform, With<Player>>,
    camera: Query<&Transform, With<Camera>>,
    time: Res<Time>,
    mut neutral_count: Local<usize>,
    mut rand: ResMut<EyesRandom>,
) {
    let player_position = players
        .iter()
        .next()
        .map(|it| it.translation)
        .or_else(|| Some(camera.single().ok()?.translation))
        .unwrap_or_default();

    let camera_position = camera.single().map(|it| it.translation).unwrap_or_default();

    for (mut eye, mut emotion) in &mut eye {
        match emotion.as_mut() {
            Emotion::Neutral(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *neutral_count += 1;

                if *neutral_count >= 3 {
                    *neutral_count = 0;

                    *emotion = Emotion::boredom(&mut rand);
                } else {
                    eye.target = random_boredom_target(&mut rand, eye.target, 0.2);
                    *emotion = Emotion::neutral(&mut rand);
                }
            }
            Emotion::Bored(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::boredom(&mut rand);
                eye.target = random_boredom_target(&mut rand, eye.target, 1.);
            }
            Emotion::Surprised(timer) => {
                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::focused(&mut rand);
            }
            Emotion::Focused { timer, target } => {
                eye.target = match target {
                    FocusTarget::Camera => camera_position,
                    FocusTarget::Player => player_position,
                };

                if !timer.tick(time.delta()).just_finished() {
                    continue;
                }

                *emotion = Emotion::neutral(&mut rand);
            }
        }
    }
}

fn emotion_from_player_activity(
    _trigger: Trigger<PlayerActivity>,
    mut eye: Query<&mut Emotion>,
    mut rand: ResMut<EyesRandom>,
) {
    for mut emotion in &mut eye {
        match emotion.as_mut() {
            Emotion::Focused { timer, target } => match target {
                FocusTarget::Player => {
                    timer.reset();
                }
                FocusTarget::Camera => {}
            },
            Emotion::Bored(_) | Emotion::Neutral(_) => *emotion = Emotion::focused(&mut rand),
            _ => {}
        }
    }
}

fn animate_emotion(
    mut eyes: Query<(&mut Transform, &Children, &Emotion), With<Eye>>,
    mut iris: Query<&mut Transform, (With<Iris>, Without<Eye>)>,
    time: Res<Time>,
) -> Result {
    for (mut eye, children, emotion) in &mut eyes {
        let mut iris = iris.get_mut(children[0])?;

        eye.scale.y = eye.scale.y.lerp(
            emotion.target_scale(),
            time.delta_secs() * emotion.emotion_speed(),
        );

        iris.scale.y = 1. / eye.scale.y;
        iris.scale = iris.scale.lerp(
            Vec3::ONE * emotion.dilation(),
            time.delta_secs() * emotion.emotion_speed(),
        );
    }

    Ok(())
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct TalkingTimer(Timer);

#[derive(Debug, Resource, Deref, DerefMut)]
struct ChangeColorTimer(Timer);

fn trigger_talking_animation(
    _trigger: Trigger<DialogueStarted>,
    mut commands: Commands,
    mut eye: Query<&mut Emotion>,
    mut rand: ResMut<EyesRandom>,
) {
    commands.insert_resource(TalkingTimer(Timer::from_seconds(3.5, TimerMode::Once)));
    commands.insert_resource(ChangeColorTimer(Timer::from_seconds(0., TimerMode::Once)));
    for mut emotion in &mut eye {
        match emotion.as_mut() {
            Emotion::Focused { timer, target } => {
                timer.reset();
                *target = FocusTarget::Player;
            }
            Emotion::Bored(_) | Emotion::Neutral(_) => *emotion = Emotion::focused(&mut rand),
            _ => {}
        }
    }
}

fn animate_talking(
    talking_timer: Option<ResMut<TalkingTimer>>,
    change_color_timer: Option<ResMut<ChangeColorTimer>>,
    mut eye_material: EyeMaterial,
    time: Res<Time>,
    mut rand: ResMut<EyesRandom>,
) {
    if talking_timer.is_none() || change_color_timer.is_none() {
        return;
    }

    let mut talking_timer = talking_timer.unwrap();
    let mut change_color_timer = change_color_timer.unwrap();

    if talking_timer.0.finished() {
        return;
    }

    let (eye, iris) = eye_material.get_materials();

    if talking_timer.tick(time.delta()).just_finished() {
        eye.base_color = Color::BLACK;
        iris.base_color = Color::BLACK;
        return;
    }

    if change_color_timer.tick(time.delta()).just_finished() {
        // eye_material.base_color =
        //     Color::hsv(talking_timer.remaining_secs() * 360. * 3. % 360., 1.0, 1.0);
        let color = Color::hsv(rand.random::<f32>() * 360.0, 1.0, 1.0);
        eye.base_color = color;
        iris.base_color = Color::hsv(rand.random::<f32>() * 360.0, 1.0, 1.0);

        *change_color_timer = ChangeColorTimer(Timer::from_seconds(0.05, TimerMode::Once));
    }
}
