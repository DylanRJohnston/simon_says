use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    actions::{Action, ActionPlan},
    level::{self, Level, LevelCounter, SCENES},
    player::LevelCompleted,
    simulation::SimulationStop,
};

use super::*;

pub struct ChallengePlugin;

impl Plugin for ChallengePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_challenge_ui.run_if(in_state(GameState::InGame)),
        )
        .add_systems(Startup, init_challenge_state)
        .observe(update_challenges)
        .observe(count_steps)
        .observe(reset_steps);
    }
}

#[derive(Debug, Component)]
pub struct ChallengeRoot;

impl ChallengePlugin {
    pub fn spawn_ui(container: &mut ChildBuilder) {
        container.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    row_gap: Val::Px(UI_CONTAINER_GAP / 2.),
                    ..default()
                },
                ..default()
            },
            ChallengeRoot,
        ));
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct ChallengeRecord {
    pub commands: Option<bool>,
    pub steps: Option<bool>,
    pub waste: Option<bool>,
    pub level_completed: bool,
}

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct ChallengeState(Vec<ChallengeRecord>);

#[derive(SystemParam)]
pub struct ActiveChallenge<'w> {
    level_counter: Res<'w, LevelCounter>,
    challenge_state: ResMut<'w, ChallengeState>,
}

impl ActiveChallenge<'_> {
    pub fn get_record_mut(&mut self) -> Option<&mut ChallengeRecord> {
        self.challenge_state.0.get_mut(**self.level_counter)
    }

    pub fn get_record(&self) -> Option<&ChallengeRecord> {
        self.challenge_state.0.get(**self.level_counter)
    }

    pub fn is_changed(&self) -> bool {
        self.level_counter.is_changed() || self.challenge_state.is_changed()
    }
}

fn init_challenge_state(mut commands: Commands) {
    commands.insert_resource(ChallengeState(
        SCENES
            .iter()
            .map(|scene| {
                if let level::Scene::Level(level) = scene {
                    ChallengeRecord {
                        commands: level.command_challenge.is_some().then_some(false),
                        steps: level.step_challenge.is_some().then_some(false),
                        waste: level.waste_challenge.is_some().then_some(false),
                        level_completed: false,
                    }
                } else {
                    ChallengeRecord::default()
                }
            })
            .collect(),
    ));
    commands.insert_resource(StepCount(0));
}

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut)]
pub struct StepCount(usize);
fn count_steps(_trigger: Trigger<Action>, mut step_count: ResMut<StepCount>) {
    **step_count += 1;
}

fn reset_steps(_trigger: Trigger<SimulationStop>, mut step_count: ResMut<StepCount>) {
    **step_count = 0;
}

fn update_challenges(
    _trigger: Trigger<LevelCompleted>,
    mut challenge: ActiveChallenge,
    action_plan: Res<ActionPlan>,
    step_count: Res<StepCount>,
    level: Res<Level>,
) {
    if let Some(challenge) = challenge.get_record_mut() {
        challenge.level_completed = true;

        if let Some(command_challenge) = level.command_challenge {
            if action_plan.len() <= command_challenge {
                if let Some(completed) = &mut challenge.commands {
                    *completed = true;
                }
            }
        }

        if let Some(step_challenge) = level.step_challenge {
            if **step_count <= step_challenge {
                if let Some(completed) = &mut challenge.steps {
                    *completed = true;
                }
            }
        }

        if let Some(waste_challenge) = level.waste_challenge {
            if **step_count >= waste_challenge {
                if let Some(completed) = &mut challenge.waste {
                    *completed = true;
                }
            }
        }
    }
}

fn update_challenge_ui(
    mut commands: Commands,
    query: Query<Entity, With<ChallengeRoot>>,
    level: Res<Level>,
    challenge: ActiveChallenge,
) {
    if !(level.is_changed() || challenge.is_changed()) {
        return;
    }

    let challenge = challenge.get_record().cloned().unwrap_or_default();

    commands
        .entity(query.get_single().unwrap())
        .despawn_descendants()
        .with_children(|container| {
            let success_color = Color::srgba_u8(0x0c, 0xc4, 0x0f, 0xdd);

            let mut spawn_challenge_tracker = |text: String, completed: bool| {
                container
                    .spawn(NodeBundle {
                        style: Style {
                            padding: UiRect::axes(
                                Val::Px(UI_CONTAINER_GAP * 2.),
                                Val::Px(UI_CONTAINER_GAP),
                            ),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(UI_CONTAINER_GAP),
                            // border: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        border_radius: BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                        border_color: if completed {
                            success_color.into()
                        } else {
                            BorderColor::default()
                        },
                        background_color: (*UI_BACKGROUND_COLOR).into(),
                        ..default()
                    })
                    .with_children(|container| {
                        container.spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(24.),
                                height: Val::Px(24.),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            border_radius: BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                            border_color: (*PRIMARY_TEXT_COLOR).into(),
                            background_color: if completed {
                                success_color.into()
                            } else {
                                BackgroundColor::default()
                            },
                            ..default()
                        });

                        container.spawn(TextBundle {
                            text: Text::from_section(
                                text,
                                TextStyle {
                                    color: if completed {
                                        *GHOST_TEXT_COLOR
                                    } else {
                                        *PRIMARY_TEXT_COLOR
                                    },
                                    ..default()
                                },
                            ),
                            ..default()
                        });
                    });
            };

            if let Some(steps) = level.step_challenge {
                spawn_challenge_tracker(
                    format!("Alacrity: Take {steps} or fewer steps"),
                    challenge.steps.unwrap_or_default(),
                );
            }

            if let Some(commands) = level.command_challenge {
                spawn_challenge_tracker(
                    format!("Parsimony: Use {commands} or fewer commands"),
                    challenge.commands.unwrap_or_default(),
                );
            }

            if let Some(steps) = level.waste_challenge {
                spawn_challenge_tracker(
                    format!("Circuity: Take {steps} or more steps"),
                    challenge.waste.unwrap_or_default(),
                );
            }
        });
}
