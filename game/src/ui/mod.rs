use action_list::ActionListPlugin;
use action_menu::ActionMenuPlugin;
use bevy::prelude::*;
use button::ButtonPlugin;
use challenges::ChallengePlugin;
use constants::*;
use controls::ControlsPlugin;
use dialogue::DialoguePlugin;
use end_screen::EndScreenPlugin;
use main_menu::MainMenuPlugin;
use settings::SettingsPlugin;

use crate::{actions::AddAction, game_state::GameState, simulation::SimulationStart};

pub mod action_list;
pub mod action_menu;
pub mod button;
pub mod challenges;
pub mod constants;
pub mod controls;
pub mod dialogue;
pub mod end_screen;
pub mod main_menu;
pub mod settings;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ButtonPlugin)
            .add_plugins(ActionListPlugin)
            .add_plugins(ActionMenuPlugin)
            .add_plugins(ControlsPlugin)
            .add_plugins(MainMenuPlugin)
            .add_plugins(ChallengePlugin)
            .add_plugins(DialoguePlugin)
            .add_plugins(EndScreenPlugin)
            .add_plugins(SettingsPlugin)
            .add_systems(OnExit(GameState::MainMenu), setup);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                column_gap: Val::Px(UI_CONTAINER_GAP),
                padding: UiRect::all(Val::Px(SCREEN_CONTAINER_PADDING)),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn(NodeBundle {
                    style: Style {
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(UI_CONTAINER_GAP),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|container| {
                    ActionMenuPlugin::spawn_ui(container);

                    container
                        .spawn(NodeBundle {
                            style: Style {
                                column_gap: Val::Px(UI_CONTAINER_GAP),
                                align_items: AlignItems::Start,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|container| {
                            ActionListPlugin::spawn_ui(container);
                            ControlsPlugin::spawn_controls(container);
                        });
                });

            ChallengePlugin::spawn_ui(commands);
        });
}

pub fn horizontal_line() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            border: UiRect::all(Val::Px(1.)),
            margin: UiRect::axes(Val::Px(0.), Val::Px(1. * UI_CONTAINER_GAP)),
            ..default()
        },
        border_radius: BorderRadius::all(Val::Px(1.)),
        border_color: (*GHOST_TEXT_COLOR).into(),
        ..default()
    }
}
