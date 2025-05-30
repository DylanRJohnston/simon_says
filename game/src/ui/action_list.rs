use bevy::{ecs::spawn::SpawnWith, prelude::*};

use crate::{
    actions::{ActionPlan, RemoveAction},
    assets::IconAssets,
    game_state::GameState,
    level::Level,
    simulation::{SimulationProgramCounter, SimulationState},
};

use super::{button, challenges::StepCount, constants::*, horizontal_line};

pub struct ActionListPlugin;

impl Plugin for ActionListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_action_list.run_if(in_state(GameState::InGame)),
        )
        .add_systems(Update, reorder_button.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Debug, Component)]
pub struct ActionPlanUI;

impl ActionListPlugin {
    pub fn spawn_ui() -> impl Bundle {
        (
            ActionPlanUI,
            Node {
                // width: Val::Px(350.),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                row_gap: Val::Px(UI_CONTAINER_GAP),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
            BackgroundColor(UI_BACKGROUND_COLOR),
        )
    }
}

#[derive(Debug)]
pub enum ButtonType {
    Up,
    Down,
}

#[derive(Debug, Component)]
pub struct ReorderButton {
    pub button_type: ButtonType,
    pub index: usize,
    pub disabled: bool,
}

#[allow(clippy::too_many_arguments)]
fn update_action_list(
    mut commands: Commands,
    query: Query<Entity, With<ActionPlanUI>>,
    icons: Res<IconAssets>,
    // System runs when these change,
    action_plan: Res<ActionPlan>,
    level: Res<Level>,
    simulation_state: Res<State<SimulationState>>,
    program_counter: Res<SimulationProgramCounter>,
    step_count: Res<StepCount>,
) {
    if !(action_plan.is_changed()
        || level.is_changed()
        || simulation_state.is_changed()
        || program_counter.is_changed()
        || step_count.is_changed())
    {
        return;
    }

    let ui = query.single().unwrap();
    commands
        .entity(ui)
        .despawn_related::<Children>()
        .insert(children![
            (
                Text("Simon Says".into()),
                Node {
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                TextColor(PRIMARY_TEXT_COLOR),
                TextFont {
                    font_size: 45.,
                    ..default()
                },
            ),
            (
                Node {
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                Text(format!(
                    "max {max} command{plural}{trailing}",
                    max = level.action_limit,
                    plural = if level.action_limit == 1 { "" } else { "s" },
                    trailing = if **step_count > 0 {
                        format!("; {} steps", **step_count)
                    } else {
                        "".into()
                    }
                )),
                TextColor(PRIMARY_TEXT_COLOR),
                TextFont {
                    font_size: 20.,
                    ..default()
                },
            ),
            horizontal_line(),
        ])
        .with_children(|parent| {
            if action_plan.is_empty() {
                parent.spawn((Text("No Commands".into()), TextColor(GHOST_TEXT_COLOR)));
            }

            for (index, action) in action_plan.iter().enumerate() {
                let prevent_interactions = simulation_state.get() != &SimulationState::Stopped;

                let background_color = match program_counter.0 == index && prevent_interactions {
                    true => GHOST_TEXT_COLOR,
                    false => Color::NONE,
                };

                let up = icons.up.clone();
                let down = icons.down.clone();

                let is_last = index == action_plan.len() - 1;

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            min_height: Val::Px(40.),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                        BackgroundColor(background_color),
                        children![
                            (
                                Node {
                                    width: Val::Px(24.),
                                    height: Val::Px(24.),
                                    ..default()
                                },
                                Children::spawn(SpawnWith(
                                    move |re_arrange_box: &mut ChildSpawner| {
                                        if prevent_interactions {
                                            return;
                                        }

                                        let up_disabled = index == 0;
                                        re_arrange_box.spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(24.),
                                                height: Val::Px(24.),
                                                top: Val::Px(-8.),
                                                position_type: PositionType::Absolute,
                                                ..default()
                                            },
                                            ImageNode::new(up).with_color(if up_disabled {
                                                GHOST_ATTENUATION_COLOR
                                            } else {
                                                Color::WHITE
                                            }),
                                            ReorderButton {
                                                button_type: ButtonType::Up,
                                                disabled: up_disabled,
                                                index,
                                            },
                                        ));

                                        let down_disabled = is_last;
                                        re_arrange_box.spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(24.),
                                                height: Val::Px(24.),
                                                bottom: Val::Px(-8.),
                                                position_type: PositionType::Absolute,
                                                ..default()
                                            },
                                            ImageNode::new(down).with_color(if down_disabled {
                                                GHOST_ATTENUATION_COLOR
                                            } else {
                                                Color::WHITE
                                            }),
                                            ReorderButton {
                                                button_type: ButtonType::Down,
                                                disabled: down_disabled,
                                                index,
                                            },
                                        ));
                                    }
                                ))
                            ),
                            (
                                Text((*action).into()),
                                Node {
                                    flex_grow: 1.,
                                    ..default()
                                },
                            )
                        ],
                    ))
                    .with_children(|row| {
                        if prevent_interactions {
                            return;
                        }

                        row.spawn(
                            button::Button::builder()
                                .icon(icons.remove.clone())
                                .on_click(move |commands| commands.trigger(RemoveAction(index)))
                                .background_color(UI_BACKGROUND_COLOR)
                                .border_color(UI_BACKGROUND_COLOR)
                                .hover_background_color(BUTTON_CANCEL_COLOR)
                                .build(),
                        );
                    });
            }
        });
}

fn reorder_button(
    mut buttons: Query<(&ReorderButton, &Interaction, &mut ImageNode), Changed<Interaction>>,
    mut action_plan: ResMut<ActionPlan>,
) {
    for (button, interaction, mut image) in &mut buttons {
        if button.disabled {
            continue;
        }

        match interaction {
            Interaction::None => {
                image.color = Color::WHITE;
            }
            Interaction::Hovered => {
                image.color = BUTTON_COLOR;
            }
            Interaction::Pressed => match button.button_type {
                ButtonType::Up => {
                    action_plan.swap(button.index, button.index - 1);
                }
                ButtonType::Down => {
                    action_plan.swap(button.index, button.index + 1);
                }
            },
        }
    }
}
