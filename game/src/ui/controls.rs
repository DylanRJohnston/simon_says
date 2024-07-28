use bevy::prelude::*;

use crate::{
    actions::ActionPlan,
    player::SpawnPlayer,
    simulation::{SimulationState, SimulationStop},
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

#[derive(Debug, Component)]
pub struct ResetButton;

impl ControlsPlugin {
    pub fn spawn_controls(container: &mut ChildBuilder) {
        button::Button::builder()
            .text("Reset".into())
            .background_color(*BUTTON_CANCEL_COLOR)
            .on_click(Box::new(|commands, _entity| {
                commands.trigger(SimulationStop);
                commands.trigger(SpawnPlayer);
            }))
            .disabled()
            .build(container)
            .insert(ResetButton);

        button::Button::builder()
            .text("Start".into())
            .background_color(*BUTTON_SUCCESS_COLOR)
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
    mut play_button: Query<&mut button::Button, (With<PlayButton>, Without<ResetButton>)>,
    mut reset_button: Query<&mut button::Button, (With<ResetButton>, Without<PlayButton>)>,
) {
    if !(action_plan.is_changed() || simulation_state.is_changed()) {
        return;
    }

    for mut button in &mut play_button {
        button.disabled = action_plan.is_empty() || *simulation_state != SimulationState::Stopped;
    }

    for mut button in &mut reset_button {
        button.disabled = *simulation_state != SimulationState::Running;
    }
}
