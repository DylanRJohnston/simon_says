use bevy::{prelude::*, transform::commands};

use crate::actions::ActionPlan;

use super::*;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_control_state);
    }
}

#[derive(Debug, Component)]
pub struct PlayButton;

impl ControlsPlugin {
    pub fn spawn_controls(container: &mut ChildBuilder) {
        button::Button::builder()
            .text("Start".into())
            .background_color(*BUTTON_SUCCESS_COLOR)
            .border_color(*BUTTON_SUCCESS_COLOR)
            .hover_background_color(*BUTTON_SUCCESS_COLOR)
            .hover_border_color(*PRIMARY_TEXT_COLOR)
            .on_click(Box::new(|commands, _entity| {
                commands.trigger(SimulationStart);
            }))
            .build(container)
            .insert(PlayButton);
    }
}

fn update_control_state(
    mut commands: Commands,
    action_plan: Res<ActionPlan>,
    mut query: Query<(Entity, &mut button::Button), With<PlayButton>>,
) {
    if !action_plan.is_changed() {
        return;
    }

    let (entity, mut button) = query.get_single_mut().unwrap();

    if action_plan.is_empty() {
        button.background_color = *UI_BACKGROUND_COLOR;
        button.border_color = *UI_BACKGROUND_COLOR;
        button.hover_background_color = *UI_BACKGROUND_COLOR;
        button.hover_border_color = *UI_BACKGROUND_COLOR;
        button.text_color = *GHOST_TEXT_COLOR;

        commands.entity(entity).insert(button::Disabled);
    } else {
        button.background_color = *BUTTON_SUCCESS_COLOR;
        button.border_color = *BUTTON_SUCCESS_COLOR;
        button.hover_background_color = *BUTTON_SUCCESS_COLOR;
        button.hover_border_color = *PRIMARY_TEXT_COLOR;
        button.text_color = *PRIMARY_TEXT_COLOR;

        commands.entity(entity).remove::<button::Disabled>();
    }
}
