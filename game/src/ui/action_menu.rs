use bevy::prelude::*;

use crate::{actions::ActionPlan, level::Level, simulation::SimulationState};

use super::*;

pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_available_actions.run_if(in_state(GameState::InGame)),
        )
        .add_systems(Update, plan_is_full);
    }
}

#[derive(Debug, Component)]
pub struct ActionMenuUI;

impl ActionMenuPlugin {
    pub fn ui() -> impl Bundle {
        (
            Name::new("Command Menu UI"),
            ActionMenuUI,
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(UI_CONTAINER_PADDING)),
                ..default()
            },
            BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
            BackgroundColor(UI_BACKGROUND_COLOR),
        )
    }
}

#[derive(Debug, Component)]
pub struct ActionButton;

fn update_available_actions(
    mut commands: Commands,
    level: Res<Level>,
    action_menu: Query<Entity, With<ActionMenuUI>>,
) -> Result {
    if !level.is_changed() {
        return Ok(());
    }

    let ui = action_menu.single()?;
    commands
        .entity(ui)
        .despawn_related::<Children>()
        .with_children(|header| {
            header.spawn((
                Text("Available Commands".into()),
                TextColor(PRIMARY_TEXT_COLOR),
                TextFont {
                    font_size: 45.,
                    ..default()
                },
            ));
            header.spawn(horizontal_line());
            header
                .spawn((Node {
                    column_gap: Val::Px(8.),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },))
                .with_children(|action_row| {
                    for action in &level.actions {
                        let action = *action;

                        action_row.spawn((
                            ActionButton,
                            button::Button::builder()
                                .text(action.into())
                                .on_click(move |commands| commands.trigger(AddAction(action)))
                                .build(),
                        ));
                    }
                });
        });

    Ok(())
}

fn plan_is_full(
    level: Res<Level>,
    action_plan: Res<ActionPlan>,
    simulation: Res<State<SimulationState>>,
    mut buttons: Query<&mut button::Button, With<ActionButton>>,
) {
    if !action_plan.is_changed() && !level.is_changed() && !simulation.is_changed() {
        return;
    }

    let full = action_plan.len() >= level.action_limit;

    for mut button in &mut buttons {
        button.disabled = full || simulation.get() != &SimulationState::Stopped;
    }
}
