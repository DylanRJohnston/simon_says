use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_platform::collections::hash_map::HashMap;
use serde::{Deserialize, Serialize};

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
        app.insert_resource(StepCount::default())
            .add_systems(
                Update,
                update_challenge_ui.run_if(in_state(GameState::InGame)),
            )
            .add_observer(update_challenges)
            .add_observer(count_steps)
            .add_observer(reset_steps);
    }
}

#[derive(Debug, Component)]
pub struct ChallengeRoot;

impl ChallengePlugin {
    pub fn spawn_ui() -> impl Bundle {
        (
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                row_gap: Val::Px(UI_CONTAINER_GAP / 2.),
                ..default()
            },
            ChallengeRoot,
        )
    }
}

#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub commands: Option<bool>,
    pub steps: Option<bool>,
    pub waste: Option<bool>,
    pub level_completed: bool,
}

#[derive(Debug, Clone, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct ChallengeState(HashMap<String, ChallengeRecord>);

impl Default for ChallengeState {
    fn default() -> Self {
        Self(
            SCENES
                .iter()
                .filter_map(|scene| {
                    if let level::Scene::Level(level) = scene {
                        Some((
                            level.name.to_string(),
                            ChallengeRecord {
                                commands: level.command_challenge.is_some().then_some(false),
                                steps: level.step_challenge.is_some().then_some(false),
                                waste: level.waste_challenge.is_some().then_some(false),
                                level_completed: false,
                            },
                        ))
                    } else {
                        None
                    }
                })
                .collect(),
        )
    }
}

impl ChallengeState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(SystemParam)]
pub struct ActiveChallenge<'w> {
    level_counter: Res<'w, LevelCounter>,
    challenge_state: ResMut<'w, ChallengeState>,
}

impl ActiveChallenge<'_> {
    pub fn get_record_mut(&mut self) -> Option<&mut ChallengeRecord> {
        match SCENES.get(**self.level_counter)? {
            level::Scene::Level(level) => Some(
                self.challenge_state
                    .entry(level.name.to_string())
                    .or_default(),
            ),
            _ => None,
        }
    }

    pub fn is_changed(&self) -> bool {
        self.level_counter.is_changed() || self.challenge_state.is_changed()
    }
}

#[derive(Debug, Clone, Copy, Resource, Default, Deref, DerefMut)]
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
    mut challenge: ActiveChallenge,
) {
    if !(level.is_changed() || challenge.is_changed()) {
        return;
    }

    let challenge = challenge.get_record_mut().cloned().unwrap_or_default();

    commands
        .entity(query.single().unwrap())
        .despawn_related::<Children>()
        .with_children(|container| {
            let success_color = Color::srgba_u8(0x0c, 0xc4, 0x0f, 0xdd);

            let mut spawn_challenge_tracker = |text: String, completed: bool| {
                container
                    .spawn((
                        Node {
                            padding: UiRect::axes(
                                Val::Px(UI_CONTAINER_GAP * 2.),
                                Val::Px(UI_CONTAINER_GAP),
                            ),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(UI_CONTAINER_GAP),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(UI_CONTAINER_RADIUS)),
                        BorderColor(if completed {
                            success_color
                        } else {
                            Color::default()
                        }),
                        BackgroundColor(UI_BACKGROUND_COLOR),
                    ))
                    .with_children(|container| {
                        container.spawn((
                            Node {
                                width: Val::Px(24.),
                                height: Val::Px(24.),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                            BorderColor(PRIMARY_TEXT_COLOR),
                            BackgroundColor(if completed {
                                success_color
                            } else {
                                Color::NONE
                            }),
                        ));

                        container.spawn((
                            Text(text.into()),
                            TextColor(if completed {
                                GHOST_TEXT_COLOR
                            } else {
                                PRIMARY_TEXT_COLOR
                            }),
                        ));
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
