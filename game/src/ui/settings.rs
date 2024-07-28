use bevy::{prelude::*, ui::FocusPolicy};
use challenges::{ChallengeRecord, ChallengeState};

use crate::{
    delayed_command::DelayedCommandExt,
    game_state::{GameState, IconAssets},
    level::{self, DespawnLevel, Level, LevelCounter, SCENES},
    music::PlayChangeLevelMusic,
    player::{Death, DespawnPlayer},
};

use super::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameMode::Story)
            .add_systems(OnExit(GameState::MainMenu), spawn_ui)
            .add_systems(Update, update_settings_ui_state)
            .add_systems(Update, dismiss_settings_ui)
            .add_systems(Update, level_card_interactions)
            .observe(create_settings_ui)
            .observe(destroy_settings_ui);
    }
}

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Story,
    Challenge,
}

fn spawn_ui(mut container: Commands, icons: Res<IconAssets>) {
    container
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            button::Button::builder()
                .on_click(Box::new(|commands, _| {
                    commands.trigger(CreateSettingsUI);
                }))
                .icon(icons.bars.clone())
                .build(container);
        });
}

#[derive(Debug, Event)]
pub struct CreateSettingsUI;

#[derive(Debug, Event)]
pub struct DestroySettingsUI;

#[derive(Debug, Component)]
pub struct SettingsUIRoot;

#[derive(Debug, Component)]
pub struct StoryButton;

#[derive(Debug, Component)]
pub struct GameModeExplanation;

#[derive(Debug, Component)]
pub struct ChallengeButton;

#[derive(Debug, Component, Clone, Copy)]
pub struct LevelCard(usize);

fn create_settings_ui(
    _trigger: Trigger<CreateSettingsUI>,
    mut commands: Commands,
    game_mode: Res<GameMode>,
    challenges: Res<ChallengeState>,
    level_counter: Res<LevelCounter>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_state.set(GameState::Paused);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
                ..default()
            },
            SettingsUIRoot,
            Interaction::default(),
        ))
        .with_children(|container| {
            container
                .spawn((NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Start,
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        row_gap: Val::Px(UI_CONTAINER_GAP),
                        ..default()
                    },
                    border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    background_color: BackgroundColor(Color::srgba_u8(0x56, 0x6c, 0x86, 0xff)),
                    focus_policy: FocusPolicy::Block,
                    ..default()
                },))
                .with_children(|container| {
                    container
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(UI_CONTAINER_GAP),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|container| {
                            container.spawn(TextBundle {
                                text: Text::from_section(
                                    "Game Mode:",
                                    TextStyle {
                                        font_size: 45.,
                                        color: *PRIMARY_TEXT_COLOR,
                                        ..default()
                                    },
                                ),
                                style: Style { ..default() },
                                ..default()
                            });

                            let mut story = button::Button::builder()
                                .on_click(Box::new(|commands, _| {
                                    commands.insert_resource(GameMode::Story);
                                }))
                                .text("Story".into());

                            if *game_mode == GameMode::Story {
                                story = story.border_color(*PRIMARY_TEXT_COLOR);
                            }

                            story.build(container).insert(StoryButton);

                            let mut challenge = button::Button::builder()
                                .on_click(Box::new(|commands, _| {
                                    commands.insert_resource(GameMode::Challenge);
                                }))
                                .text("Challenge".into());

                            if *game_mode == GameMode::Challenge {
                                challenge = challenge.border_color(*PRIMARY_TEXT_COLOR);
                            }

                            challenge.build(container).insert(ChallengeButton);
                        });

                    container.spawn((
                        GameModeExplanation,
                        TextBundle {
                            text: Text::from_section(
                                if *game_mode == GameMode::Story {
                                    "Levels progress with any solution"
                                } else {
                                    "Levels only progress once all challenges are completed"
                                },
                                TextStyle {
                                    font_size: 16.,
                                    color: *PRIMARY_TEXT_COLOR,
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                    ));

                    let mut hr = horizontal_line();

                    hr.style.margin =
                        UiRect::axes(Val::Px(0.), Val::Px(UI_CONTAINER_PADDING / 2.0));
                    container.spawn(hr);

                    container
                        .spawn(NodeBundle {
                            style: Style {
                                display: Display::Grid,
                                grid_template_columns: vec![RepeatedGridTrack::fr(4, 1.)],
                                grid_auto_rows: vec![GridTrack::fr(1.)],
                                row_gap: Val::Px(UI_CONTAINER_GAP),
                                column_gap: Val::Px(UI_CONTAINER_GAP),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|container| {
                            let success_color = Color::srgba_u8(0x0c, 0xc4, 0x0f, 0xdd);
                            let incomplete_color = Color::srgb_u8(0x41, 0x53, 0x69);

                            for (index, (level, challenge)) in
                                SCENES.iter().zip(challenges.iter()).enumerate().filter_map(
                                    |(index, (scene, challenge))| match scene {
                                        crate::level::Scene::Level(level) => {
                                            Some((index, (level, challenge)))
                                        }
                                        _ => None,
                                    },
                                )
                            {
                                container
                                    .spawn((
                                        LevelCard(index),
                                        Interaction::default(),
                                        NodeBundle {
                                            style: Style {
                                                justify_content: JustifyContent::FlexStart,
                                                align_items: AlignItems::Center,
                                                flex_direction: FlexDirection::Column,
                                                padding: UiRect::all(Val::Px(
                                                    UI_CONTAINER_PADDING / 2.,
                                                )),
                                                row_gap: Val::Px(UI_CONTAINER_GAP * 2.),
                                                border: UiRect::all(Val::Px(4.)),
                                                ..default()
                                            },
                                            border_color: if index == **level_counter {
                                                (*PRIMARY_TEXT_COLOR).into()
                                            } else {
                                                BorderColor::DEFAULT
                                            },
                                            background_color: if challenge.level_completed {
                                                BackgroundColor(success_color)
                                            } else {
                                                BackgroundColor(incomplete_color)
                                            },
                                            border_radius: BorderRadius::all(Val::Px(
                                                BUTTON_BORDER_RADIUS * 2.0,
                                            )),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|container| {
                                        container.spawn(TextBundle {
                                            text: Text::from_section(
                                                format!("Level {}", index + 1),
                                                TextStyle::default(),
                                            ),
                                            ..default()
                                        });

                                        container
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    column_gap: Val::Px(UI_CONTAINER_GAP),
                                                    ..default()
                                                },
                                                ..default()
                                            })
                                            .with_children(|container| {
                                                let mut spawn_challenge_tracker =
                                                    |completed: bool| {
                                                        container.spawn(NodeBundle {
                                                            style: Style {
                                                                width: Val::Px(24.),
                                                                height: Val::Px(24.),
                                                                border: UiRect::all(Val::Px(2.)),
                                                                ..default()
                                                            },
                                                            border_radius: BorderRadius::all(
                                                                Val::Px(BUTTON_BORDER_RADIUS),
                                                            ),
                                                            border_color: (*PRIMARY_TEXT_COLOR)
                                                                .into(),
                                                            background_color: if completed {
                                                                (*PRIMARY_TEXT_COLOR).into()
                                                            } else {
                                                                incomplete_color.into()
                                                            },
                                                            ..default()
                                                        });
                                                    };

                                                if let Some(completed) = challenge.steps {
                                                    spawn_challenge_tracker(completed);
                                                }

                                                if let Some(completed) = challenge.commands {
                                                    spawn_challenge_tracker(completed);
                                                }

                                                if let Some(completed) = challenge.waste {
                                                    spawn_challenge_tracker(completed);
                                                }
                                            });
                                    });
                            }
                        });
                });
        });
}

fn update_settings_ui_state(
    game_mode: Res<GameMode>,
    mut story_buttons: Query<&mut button::Button, (With<StoryButton>, Without<ChallengeButton>)>,
    mut challenge_buttons: Query<
        &mut button::Button,
        (With<ChallengeButton>, Without<StoryButton>),
    >,
    mut explanation: Query<&mut Text, With<GameModeExplanation>>,
) {
    if !game_mode.is_changed() {
        return;
    }

    for mut button in &mut story_buttons {
        button.border_color = if *game_mode == GameMode::Story {
            Some(*PRIMARY_TEXT_COLOR)
        } else {
            None
        };
    }

    for mut button in &mut challenge_buttons {
        button.border_color = if *game_mode == GameMode::Challenge {
            Some(*PRIMARY_TEXT_COLOR)
        } else {
            None
        };
    }

    for mut text in &mut explanation {
        text.sections[0].value = match *game_mode {
            GameMode::Story => "Levels progress with any solution".into(),
            GameMode::Challenge => "Levels only progress once all challenges are completed".into(),
        };
    }
}

fn destroy_settings_ui(
    _trigger: Trigger<DestroySettingsUI>,
    roots: Query<Entity, With<SettingsUIRoot>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
) {
    state.set(GameState::InGame);
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn dismiss_settings_ui(
    mut commands: Commands,
    settings_ui: Query<&Interaction, (With<SettingsUIRoot>, Changed<Interaction>)>,
) {
    for interaction in &settings_ui {
        if interaction == &Interaction::Pressed {
            commands.trigger(DestroySettingsUI)
        }
    }
}

fn level_card_interactions(
    mut cards: Query<(&LevelCard, &Interaction, &mut BorderColor), Changed<Interaction>>,
    level_counter: Res<LevelCounter>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (level, interaction, mut cards) in &mut cards {
        let level_id = level.0;

        if level_id == **level_counter {
            *cards = BorderColor(*PRIMARY_TEXT_COLOR);
            continue;
        }

        match interaction {
            Interaction::Pressed => match SCENES.get(level_id) {
                Some(level::Scene::Level(_)) => {
                    commands.trigger(DestroySettingsUI);
                    commands.trigger(DespawnLevel);
                    commands.trigger(DespawnPlayer);
                    commands.trigger(PlayChangeLevelMusic);
                    game_state.set(GameState::InGame);
                    commands.delayed(2., move |commands| {
                        commands.trigger(level::LoadLevel(level_id))
                    });
                }
                other => tracing::warn!(?other, "attempted to select non-level scene"),
            },
            Interaction::Hovered => *cards = BorderColor(*PRIMARY_TEXT_COLOR),
            Interaction::None => *cards = BorderColor::default(),
        }
    }
}
