use bevy::{prelude::*, ui::FocusPolicy};

use crate::game_state::{GameState, IconAssets};

use super::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameMode::Story)
            .add_systems(OnEnter(GameState::InGame), spawn_ui)
            .add_systems(Update, update_settings_ui_state)
            .add_systems(Update, dismiss_settings_ui)
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
pub struct ChallengeButton;

fn create_settings_ui(
    _trigger: Trigger<CreateSettingsUI>,
    mut commands: Commands,
    game_mode: Res<GameMode>,
) {
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
                        ..default()
                    },
                    border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    background_color: (*UI_BACKGROUND_COLOR).into(),
                    focus_policy: FocusPolicy::Block,
                    ..default()
                },))
                .with_children(|container| {
                    container
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
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
}

fn destroy_settings_ui(
    _trigger: Trigger<DestroySettingsUI>,
    roots: Query<Entity, With<SettingsUIRoot>>,
    mut commands: Commands,
) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn dismiss_settings_ui(
    mut commands: Commands,
    settings_ui: Query<&Interaction, (With<SettingsUIRoot>, Changed<Interaction>)>,
) {
    for interaction in &settings_ui {
        tracing::info!(?interaction, "settings ui interaction mode");

        if interaction == &Interaction::Pressed {
            commands.trigger(DestroySettingsUI)
        }
    }
}
