use bevy::prelude::*;

use crate::actions::{ActionPlan, RemoveAction};

use super::{button, constants::*, horizontal_line};

pub struct ActionListPlugin;

impl Plugin for ActionListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_action_list);
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
                background_color: Color::srgb_u8(0x56, 0x6c, 0x86).into(),
                ..default()
            },
        ));
    }
}

fn update_action_list(
    mut commands: Commands,
    action_plan: Res<ActionPlan>,
    query: Query<Entity, With<ActionPlanUI>>,
) {
    if !action_plan.is_changed() {
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
                button::Button::builder()
                    .text((*action).into())
                    .on_click(Box::new(move |commands, _| {
                        commands.trigger(RemoveAction(index))
                    }))
                    .background_color(*UI_BACKGROUND_COLOR)
                    .border_color(*UI_BACKGROUND_COLOR)
                    .hover_background_color(*BUTTON_CANCEL_COLOR)
                    .build(parent);
            }
        });
}
