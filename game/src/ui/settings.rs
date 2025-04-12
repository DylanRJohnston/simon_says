use bevy::{prelude::*, ui::FocusPolicy};
use challenges::ChallengeState;

use crate::{
    assets::IconAssets,
    delayed_command::DelayedCommandExt,
    game_state::{GameState, ResetChallengeState},
    level::{self, DespawnLevel, LevelCounter, SCENES},
    music::{MasterVolume, PlayChangeLevelMusic},
    player::DespawnPlayer,
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
            .add_observer(create_settings_ui)
            .add_observer(destroy_settings_ui)
            .add_observer(toggle_volume);
    }
}

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Story,
    Challenge,
}

fn spawn_ui(mut commands: Commands, icons: Res<IconAssets>) {
    commands
        .spawn((
            Name::new("Settings UI Container"),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                ..default()
            },
        ))
        .with_children(|container| {
            button::Button::builder()
                .on_click(|commands| commands.trigger(CreateSettingsUI))
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
    icons: Res<IconAssets>,
    mut challenges: ResMut<ChallengeState>,
    level_counter: Res<LevelCounter>,
    master_volume: Res<MasterVolume>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_state.set(GameState::Paused);

    commands
        .spawn((
            Name::new("Settings UI Root"),
            SettingsUIRoot,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
            Interaction::default(),
        ))
        .with_children(|container| {
            container
                .spawn((
                    Name::new("Settings Panel"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Start,
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        row_gap: Val::Px(UI_CONTAINER_GAP),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    BackgroundColor(Color::srgba_u8(0x56, 0x6c, 0x86, 0xff)),
                    FocusPolicy::Block,
                ))
                .with_children(|container| {
                    // Header section with game mode and controls
                    container
                        .spawn((
                            Name::new("Header Section"),
                            Node {
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|container| {
                            // Game mode controls
                            container
                                .spawn((
                                    Name::new("Game Mode Controls"),
                                    Node {
                                        column_gap: Val::Px(UI_CONTAINER_GAP),
                                        ..default()
                                    },
                                ))
                                .with_children(|container| {
                                    container.spawn((
                                        Name::new("Game Mode Label"),
                                        Text("Game Mode:".into()),
                                        TextColor(*PRIMARY_TEXT_COLOR),
                                        TextFont {
                                            font_size: 45.,
                                            ..default()
                                        },
                                    ));

                                    let mut story = button::Button::builder()
                                        .on_click(|commands| {
                                            commands.insert_resource(GameMode::Story)
                                        })
                                        .text("Story".into());

                                    if *game_mode == GameMode::Story {
                                        story = story.border_color(*PRIMARY_TEXT_COLOR);
                                    }

                                    story.build(container).insert(StoryButton);

                                    let mut challenge = button::Button::builder()
                                        .on_click(|commands| {
                                            commands.insert_resource(GameMode::Challenge)
                                        })
                                        .text("Challenge".into());

                                    if *game_mode == GameMode::Challenge {
                                        challenge = challenge.border_color(*PRIMARY_TEXT_COLOR);
                                    }

                                    challenge.build(container).insert(ChallengeButton);
                                });

                            // Control buttons
                            container
                                .spawn((
                                    Name::new("Control Buttons"),
                                    Node {
                                        column_gap: Val::Px(UI_CONTAINER_GAP),
                                        ..default()
                                    },
                                ))
                                .with_children(|container| {
                                    button::Button::builder()
                                        .on_click(|commands| commands.trigger(ToggleVolume))
                                        .icon(match *master_volume {
                                            MasterVolume::Muted => icons.mute.clone(),
                                            MasterVolume::Unmuted => icons.unmute.clone(),
                                        })
                                        .size(24.)
                                        .build(container)
                                        .insert(VolumeButton);

                                    button::Button::builder()
                                        .background_color(*BUTTON_CANCEL_COLOR)
                                        .on_click(|commands| {
                                            commands.trigger(ResetChallengeState);
                                            commands.trigger(DestroySettingsUI);
                                            commands.trigger(DespawnLevel);
                                            commands.trigger(DespawnPlayer);
                                            commands.trigger(PlayChangeLevelMusic);
                                            commands.delayed(2., move |commands| {
                                                commands.trigger(level::LoadLevel(0))
                                            });
                                        })
                                        .text("Reset".into())
                                        .build(container);
                                });
                        });

                    // Game mode explanation
                    container.spawn((
                        Name::new("Game Mode Explanation"),
                        GameModeExplanation,
                        Text(
                            if *game_mode == GameMode::Story {
                                "Levels progress with any solution"
                            } else {
                                "Levels only progress once all challenges are completed"
                            }
                            .into(),
                        ),
                        TextColor(*PRIMARY_TEXT_COLOR),
                        TextFont {
                            font_size: 16.,
                            ..default()
                        },
                    ));

                    // Horizontal line
                    container.spawn((Name::new("Divider"), horizontal_line()));

                    // Level grid
                    container
                        .spawn((
                            Name::new("Level Grid"),
                            Node {
                                display: Display::Grid,
                                grid_template_columns: vec![RepeatedGridTrack::fr(6, 1.)],
                                grid_auto_rows: vec![GridTrack::fr(1.)],
                                row_gap: Val::Px(UI_CONTAINER_GAP),
                                column_gap: Val::Px(UI_CONTAINER_GAP),
                                ..default()
                            },
                        ))
                        .with_children(|container| {
                            let success_color = Color::srgba_u8(0x0c, 0xc4, 0x0f, 0xdd);
                            let incomplete_color = Color::srgb_u8(0x41, 0x53, 0x69);

                            for (index, level) in
                                SCENES
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(index, scene)| match scene {
                                        crate::level::Scene::Level(level) => Some((index, level)),
                                        _ => None,
                                    })
                            {
                                let challenge =
                                    challenges.entry(level.name.to_string()).or_default();

                                container
                                    .spawn((
                                        Name::new(format!("Level Card {}", index)),
                                        LevelCard(index),
                                        Interaction::default(),
                                        Node {
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
                                        BorderColor(if index == **level_counter {
                                            (*PRIMARY_TEXT_COLOR).into()
                                        } else {
                                            BorderColor::DEFAULT.0
                                        }),
                                        BackgroundColor(if challenge.level_completed {
                                            success_color
                                        } else {
                                            incomplete_color
                                        }),
                                        BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS * 2.0)),
                                    ))
                                    .with_children(|container| {
                                        container.spawn(Text(level.name.into()));

                                        container
                                            .spawn(Node {
                                                column_gap: Val::Px(UI_CONTAINER_GAP),
                                                ..default()
                                            })
                                            .with_children(|container| {
                                                let mut spawn_challenge_tracker =
                                                    |completed: bool| {
                                                        container.spawn((
                                                            Node {
                                                                width: Val::Px(24.),
                                                                height: Val::Px(24.),
                                                                border: UiRect::all(Val::Px(2.)),
                                                                ..default()
                                                            },
                                                            BorderRadius::all(Val::Px(
                                                                BUTTON_BORDER_RADIUS,
                                                            )),
                                                            BorderColor(*PRIMARY_TEXT_COLOR),
                                                            BackgroundColor(if completed {
                                                                (*PRIMARY_TEXT_COLOR).into()
                                                            } else {
                                                                incomplete_color.into()
                                                            }),
                                                        ));
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
        **text = match *game_mode {
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
    for (level, interaction, mut border_color) in &mut cards {
        let level_id = level.0;

        if level_id == **level_counter {
            *border_color = BorderColor(*PRIMARY_TEXT_COLOR);
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
            Interaction::Hovered => *border_color = BorderColor(*PRIMARY_TEXT_COLOR),
            Interaction::None => *border_color = BorderColor::default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct VolumeButton;

#[derive(Debug, Event)]
pub struct ToggleVolume;

fn toggle_volume(
    _trigger: Trigger<ToggleVolume>,
    icon_assets: Res<IconAssets>,
    mut master_volume: ResMut<MasterVolume>,
    mut icons: Query<&mut ImageNode>,
    volume_button: Query<&Children, With<VolumeButton>>,
) {
    *master_volume = match *master_volume {
        MasterVolume::Muted => MasterVolume::Unmuted,
        MasterVolume::Unmuted => MasterVolume::Muted,
    };

    for children in &volume_button {
        let mut icon = icons.get_mut(children[0]).unwrap();
        icon.image = match *master_volume {
            MasterVolume::Muted => icon_assets.mute.clone(),
            MasterVolume::Unmuted => icon_assets.unmute.clone(),
        }
    }
}
