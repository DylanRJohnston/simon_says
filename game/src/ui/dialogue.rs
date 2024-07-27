use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    delayed_command::DelayedCommand,
    game_state::{GameState, SoundAssets},
    level::LoadNextLevel,
    music::SuppressMusicVolume,
};

use super::{main_menu::Refuse, UI_BACKGROUND_COLOR, UI_CONTAINER_PADDING, UI_CONTAINER_RADIUS};

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), game_start)
            .insert_resource(DialogueQueue(Vec::new()))
            .add_systems(
                Update,
                play_dialogue_segment.run_if(|state: Res<State<GameState>>| {
                    !matches!(state.get(), GameState::Loading)
                }),
            )
            .observe(play_level_dialogue)
            .observe(level_load)
            .observe(play_refuse_dialogue)
            .observe(enqueue_dialogue);
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
    audio: Res<Audio>,
    sounds: Res<SoundAssets>,
    mut count: Local<usize>,
    mut timer: Local<Timer>,
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
    // commands.spawn(DelayedCommand::new(0.1, move |commands| {
    commands.trigger(SuppressMusicVolume {
        volume: 0.1,
        leading_edge: Duration::from_secs_f32(0.5),
        middle: Duration::from_secs_f32(dialogue_queue.0.len() as f32 * 5.),
        falling_edge: Duration::from_secs_f32(2.),
    });
    // }));

    let id = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(80.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            container
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        ..default()
                    },
                    background_color: (*UI_BACKGROUND_COLOR).into(),
                    border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    ..default()
                })
                .with_children(|container| {
                    container.spawn(TextBundle {
                        text: Text::from_section(
                            dialogue_queue.0.remove(0),
                            TextStyle {
                                font_size: 30.,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                });
        })
        .id();

    commands.spawn(DelayedCommand::new(5., move |commands| {
        commands.entity(id).despawn_recursive();
    }));

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
