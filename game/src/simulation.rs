use bevy::prelude::*;

use crate::actions::ActionPlan;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .insert_resource(SimulationProgramCounter::default())
            .add_systems(
                Update,
                run_simulation.run_if(in_state(SimulationState::Running)),
            )
            .observe(simulation_start)
            .observe(simulation_stop);
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct SimulationStart;

#[derive(Debug, Clone, Copy, Event)]
pub struct SimulationStop;

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut, Default)]
pub struct SimulationProgramCounter(pub usize);

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct SimulationTimer(pub Timer);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SimulationState {
    Running,
    #[default]
    Paused,
}

pub const SIMULATION_SPEED: f32 = 0.5;

fn simulation_start(
    _trigger: Trigger<SimulationStart>,
    action_plan: Res<ActionPlan>,
    mut commands: Commands,
    mut program_counter: ResMut<SimulationProgramCounter>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
) {
    tracing::info!("simulation started");

    program_counter.0 = 0;
    simulation_state.set(SimulationState::Running);
    commands.trigger(action_plan[0]);
    commands.insert_resource(SimulationTimer(Timer::from_seconds(
        SIMULATION_SPEED,
        TimerMode::Once,
    )));
}

fn simulation_stop(
    _trigger: Trigger<SimulationStop>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
) {
    tracing::info!("simulation stopped");

    simulation_state.set(SimulationState::Paused);
}

fn run_simulation(
    mut commands: Commands,
    mut pc: ResMut<SimulationProgramCounter>,
    mut timer: ResMut<SimulationTimer>,
    action_plan: Res<ActionPlan>,
    time: Res<Time>,
) {
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    **timer = Timer::from_seconds(SIMULATION_SPEED, TimerMode::Once);

    **pc = (**pc + 1) % action_plan.len();

    commands.trigger(action_plan[**pc]);
}
