use bevy::prelude::*;

use crate::{
    actions::{ActionPlan, RemoveAction},
    game_state::GameState,
    level::Level,
};

use super::{button, constants::*, horizontal_line};

pub struct ActionListPlugin;

impl Plugin for ActionListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_action_list.run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Debug, Component)]
pub struct ActionPlanUI;

impl ActionListPlugin {
    pub fn spawn_ui(parent: &mut ChildBuilder) {
        parent.spawn((
            ActionPlanUI,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                    row_gap: Val::Px(UI_CONTAINER_GAP),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                background_color: (*UI_BACKGROUND_COLOR).into(),
                ..default()
            },
        ));
    }
}

fn update_action_list(
    mut commands: Commands,
    action_plan: Res<ActionPlan>,
    query: Query<Entity, With<ActionPlanUI>>,
    level: Res<Level>,
    asset_server: Res<AssetServer>,
) {
    if !(action_plan.is_changed() || level.is_changed()) {
        return;
    }

    let ui = query.get_single().unwrap();
    commands
        .entity(ui)
        .despawn_descendants()
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style { ..default() },
                text: Text::from_section(
                    "Simon Says",
                    TextStyle {
                        color: *PRIMARY_TEXT_COLOR,
                        font_size: 45.,
                        ..default()
                    },
                ),
                ..default()
            });

            parent.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                text: Text::from_section(
                    format!(
                        "{count}/{max} command{plural}",
                        count = action_plan.len(),
                        max = level.action_limit,
                        plural = if action_plan.len() == 1 { "" } else { "s" }
                    ),
                    TextStyle {
                        color: *PRIMARY_TEXT_COLOR,
                        font_size: 20.,
                        ..default()
                    },
                ),
                ..default()
            });

            parent.spawn(horizontal_line());

            if action_plan.is_empty() {
                parent.spawn(TextBundle {
                    style: Style { ..default() },
                    text: Text::from_section(
                        "No Commands",
                        TextStyle {
                            color: *GHOST_TEXT_COLOR,
                            ..default()
                        },
                    ),
                    ..default()
                });
            }

            for (index, action) in action_plan.iter().enumerate() {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|row| {
                        // TODO: Drag and drop doesn't work yet
                        // row.spawn((
                        //     NodeBundle {
                        //         style: Style {
                        //             width: Val::Px(20.),
                        //             height: Val::Px(20.),
                        //             ..default()
                        //         },
                        //         ..default()
                        //     },
                        //     UiImage::new(asset_server.load("icons/drag.png")),
                        // ));

                        row.spawn(TextBundle {
                            style: Style {
                                flex_grow: 1.,
                                ..default()
                            },
                            text: Text::from_sections([TextSection::new(
                                *action,
                                TextStyle::default(),
                            )]),
                            ..default()
                        });

                        button::Button::builder()
                            .icon(asset_server.load("icons/remove.png"))
                            .on_click(Box::new(move |commands, _| {
                                commands.trigger(RemoveAction(index))
                            }))
                            .background_color(*UI_BACKGROUND_COLOR)
                            .border_color(*UI_BACKGROUND_COLOR)
                            .hover_background_color(*BUTTON_CANCEL_COLOR)
                            .build(row);
                    });
            }
        });
}
