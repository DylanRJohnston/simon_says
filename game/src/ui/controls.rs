use bevy::prelude::*;

use crate::{
    actions::ActionPlan,
    simulation::{self, SimulationState},
};

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
            .disabled()
            .build(container)
            .insert(PlayButton);
    }
}

fn update_control_state(
    action_plan: Res<ActionPlan>,
    simulation_state: Res<State<SimulationState>>,
    mut query: Query<&mut button::Button, With<PlayButton>>,
) {
    if !(action_plan.is_changed() || simulation_state.is_changed()) {
        return;
    }

    for mut button in &mut query {
        button.disabled = action_plan.is_empty() || *simulation_state == SimulationState::Running;
    }
}
