use bevy::prelude::*;

use crate::level::Level;

use super::*;

pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_available_actions);
    }
}

#[derive(Debug, Component)]
pub struct ActionMenuUI;

impl ActionMenuPlugin {
    pub fn spawn_ui(container: &mut ChildBuilder) {
        container.spawn((
            ActionMenuUI,
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(2.)),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                    ..default()
                },
                border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                background_color: (*UI_BACKGROUND_COLOR).into(),
                ..default()
            },
        ));
    }
}

fn update_available_actions(
    mut commands: Commands,
    level: Res<Level>,
    action_menu: Query<Entity, With<ActionMenuUI>>,
) {
    if !level.is_changed() {
        return;
    }

    let ui = action_menu.get_single().unwrap();
    commands
        .entity(ui)
        .despawn_descendants()
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
                    for action in &level.actions {
                        let action = *action;

                        button::Button::builder()
                            .text(action.into())
                            .on_click(Box::new(move |commands, _| {
                                commands.trigger(AddAction(action))
                            }))
                            .build(action_row);
                    }
                });
        });
}