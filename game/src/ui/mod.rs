use action_list::ActionListPlugin;
use bevy::prelude::*;
use button::ButtonPlugin;
use constants::*;

use crate::{
    actions::{Action, AddAction},
    simulation::SimulationStart,
};

pub mod action_list;
pub mod button;
pub mod constants;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ButtonPlugin)
            .add_plugins(ActionListPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(UI_CONTAINER_GAP),
                padding: UiRect::all(Val::Px(SCREEN_CONTAINER_PADDING)),
                ..default()
            },
            ..default()
        })
        .with_children(|container| {
            container
                .spawn(NodeBundle {
                    style: Style {
                        border: UiRect::all(Val::Px(2.)),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                        ..default()
                    },
                    border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                    background_color: (*UI_BACKGROUND_COLOR).into(),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn(TextBundle {
                        text: Text::from_section(
                            "Commands",
                            TextStyle {
                                font_size: 45.,
                                color: *PRIMARY_TEXT_COLOR,
                                ..default()
                            },
                        ),
                        style: Style { ..default() },
                        ..default()
                    });
                    header.spawn(horizontal_line());
                    header
                        .spawn(NodeBundle {
                            style: Style {
                                column_gap: Val::Px(8.),
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|action_row| {
                            for action in [
                                Action::Forward,
                                Action::Backward,
                                Action::Left,
                                Action::Right,
                            ] {
                                button::Button::builder()
                                    .text(action.into())
                                    .on_click(Box::new(move |commands, _| {
                                        commands.trigger(AddAction(action))
                                    }))
                                    .build(action_row);
                            }
                        });
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
                    ActionListPlugin::spawn_ui(container);

                    button::Button::builder()
                        .text("Start".into())
                        .background_color(*BUTTON_SUCCESS_COLOR)
                        .border_color(*BUTTON_SUCCESS_COLOR)
                        .hover_background_color(*BUTTON_SUCCESS_COLOR)
                        .hover_border_color(*PRIMARY_TEXT_COLOR)
                        .on_click(Box::new(|commands, _entity| {
                            commands.trigger(SimulationStart);
                        }))
                        .build(container);
                });
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
