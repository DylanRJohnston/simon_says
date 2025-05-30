use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    assets::SoundAssets,
    delayed_command::DelayedCommandExt,
    game_state::GameState,
    level::LoadNextLevel,
    music::{DialogueChannel, MasterVolume, SuppressMusicVolume},
};

use super::{
    UI_CONTAINER_PADDING, UI_CONTAINER_RADIUS, constants::UI_BACKGROUND_COLOR, main_menu::Refuse,
};

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::MainMenu), game_start)
            .insert_resource(DialogueQueue(Vec::new()))
            .add_systems(
                Update,
                play_dialogue_segment.run_if(|state: Res<State<GameState>>| {
                    !matches!(state.get(), GameState::Loading)
                }),
            )
            .add_observer(play_level_dialogue)
            .add_observer(level_load)
            .add_observer(play_refuse_dialogue)
            .add_observer(enqueue_dialogue);
    }
}

const LEVEL_START_DIALOGUE: &[&[&str]] = &[
    &[
        "Welcome/Condolences, Subject/Anomaly",
        "Your Function/Design is to Solve/Entertain",
    ],
    &["Task/Symphony Completed/Finalized"],
    &["Your performance was Exceptional/Preordained"],
    &["Proceed/Retry next Challenge/Calculation"],
    &["You/Separated are an Anachronism/Relic"],
];

#[derive(Event)]
struct PlayLevelDialogue;

fn game_start(mut commands: Commands) {
    commands.trigger(PlayLevelDialogue);
}

fn level_load(_trigger: Trigger<LoadNextLevel>, mut commands: Commands) {
    commands.trigger(PlayLevelDialogue);
}

fn play_level_dialogue(
    _trigger: Trigger<PlayLevelDialogue>,
    mut commands: Commands,
    mut count: Local<usize>,
) {
    if *count >= LEVEL_START_DIALOGUE.len() {
        return;
    }

    for dialogue_segment in LEVEL_START_DIALOGUE[*count] {
        commands.trigger(PlayDialogueSegment(dialogue_segment));
    }

    *count += 1;
}

#[derive(Event)]
struct PlayDialogueSegment(&'static str);

fn enqueue_dialogue(
    trigger: Trigger<PlayDialogueSegment>,
    mut dialogue_queue: ResMut<DialogueQueue>,
) {
    dialogue_queue.0.push(trigger.event().0);
}

#[derive(Resource)]
pub struct DialogueQueue(Vec<&'static str>);

#[derive(Debug, Event)]
pub struct DialogueStarted;

fn play_dialogue_segment(
    mut dialogue_queue: ResMut<DialogueQueue>,
    mut commands: Commands,
    audio: Res<AudioChannel<DialogueChannel>>,
    sounds: Res<SoundAssets>,
    mut count: Local<usize>,
    mut timer: Local<Timer>,
    master_volume: Res<MasterVolume>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).finished() || dialogue_queue.0.is_empty() {
        return;
    }

    *timer = Timer::from_seconds(5., TimerMode::Once);

    if *count >= sounds.dialogue.len() {
        *count = 0;
    }

    commands.trigger(DialogueStarted);
    audio.play(sounds.dialogue[*count].clone());

    commands.trigger(SuppressMusicVolume {
        volume: master_volume.volume().min(0.1),
        leading_edge: Duration::from_secs_f32(0.5),
        middle: Duration::from_secs_f32(dialogue_queue.0.len() as f32 * 5.),
        falling_edge: Duration::from_secs_f32(2.),
    });

    let id = commands
        .spawn((
            Name::new("Dialogue Container"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(80.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
        ))
        .with_children(|container| {
            container
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        ..default()
                    },
                    BackgroundColor(UI_BACKGROUND_COLOR),
                    BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                ))
                .with_children(|container| {
                    container.spawn((
                        Text(dialogue_queue.0.remove(0).into()),
                        TextFont {
                            font_size: 30.,
                            ..default()
                        },
                    ));
                });
        })
        .id();

    commands.delayed(5., move |commands| {
        commands.entity(id).despawn();
    });

    *count += 1;
}

const REFUSE_DIALOGUE: &[&str] = &[
    "You/Anachronism are Outside/Nowhere",
    "Your Mind/Prison is Finite/Ends",
    "We/Us have nothing but Time/Continued Existence",
];

fn play_refuse_dialogue(_trigger: Trigger<Refuse>, mut commands: Commands) {
    for dialogue_segment in REFUSE_DIALOGUE {
        commands.trigger(PlayDialogueSegment(dialogue_segment));
    }
}
