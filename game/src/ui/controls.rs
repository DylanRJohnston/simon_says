use bevy::prelude::*;

use crate::{
    actions::{ActionPlan, ResetActionPlan},
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
    pub fn clear_button() -> impl Bundle {
        (
            button::Button::builder()
                .text("Clear".into())
                .background_color(BUTTON_CANCEL_COLOR)
                .on_click(|commands| commands.trigger(ResetActionPlan))
                .disabled()
                .build(),
            ResetButton,
            Name::from("Reset Button"),
        )
    }

    pub fn start_button() -> impl Bundle {
        (
            button::Button::builder()
                .text("Start".into())
                .background_color(BUTTON_SUCCESS_COLOR)
                .on_click(|commands| commands.trigger(SimulationStart))
                .disabled()
                .build(),
            PlayButton,
        )
    }
}

fn update_control_state(
    action_plan: Res<ActionPlan>,
    simulation_state: Res<State<SimulationState>>,
    mut play_button: Query<&mut button::Button, (With<PlayButton>, Without<ResetButton>)>,
    mut reset_button: Query<
        (&mut button::Button, &Children),
        (With<ResetButton>, Without<PlayButton>),
    >,
    mut button_text: Query<&mut Text>,
) {
    if !(action_plan.is_changed() || simulation_state.is_changed()) {
        return;
    }

    for mut button in &mut play_button {
        button.disabled = action_plan.is_empty() || *simulation_state != SimulationState::Stopped;
    }

    for (mut button, children) in &mut reset_button {
        let mut text = button_text.get_mut(children[0]).unwrap();

        match **simulation_state {
            SimulationState::Running => {
                button.disabled = false;
                **text = "Reset".into();
                button.on_click = Box::new(|commands| {
                    commands.trigger(SimulationStop);
                    commands.trigger(SpawnPlayer);
                });
            }
            SimulationState::Paused => {
                button.disabled = true;
            }
            SimulationState::Stopped => {
                button.disabled = action_plan.len() == 0;
                **text = "Clear".into();
                button.on_click = Box::new(|commands| {
                    commands.trigger(ResetActionPlan);
                });
            }
        }
    }
}
