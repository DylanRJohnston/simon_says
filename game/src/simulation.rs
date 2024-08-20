use bevy::prelude::*;

use crate::{
    actions::{Action, ActionPlan},
    level::{Level, Tile},
    player::Player,
};

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
            .observe(simulation_stop)
            .observe(simulation_pause);
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct SimulationStart;

#[derive(Debug, Clone, Copy, Event)]
pub struct SimulationPause;

#[derive(Debug, Clone, Copy, Event)]
pub struct SimulationStop;

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut, Default)]
pub struct SimulationProgramCounter(pub usize);

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct SimulationTimer(pub Timer);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SimulationState {
    Running,
    Paused,
    #[default]
    Stopped,
}

pub const SIMULATION_SPEED: f32 = 0.5;

fn simulation_start(
    _trigger: Trigger<SimulationStart>,
    action_plan: Res<ActionPlan>,
    mut commands: Commands,
    mut program_counter: ResMut<SimulationProgramCounter>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
) {
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
    simulation_state.set(SimulationState::Stopped);
}

fn simulation_pause(
    _trigger: Trigger<SimulationPause>,
    mut simulation_state: ResMut<NextState<SimulationState>>,
) {
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

#[derive(Debug)]
pub enum SimulationEvent {
    Finished,
    Died(usize),
}

pub fn run_simulation_step(
    level: &Level,
    players: &[Player],
    action: Action,
) -> Vec<(Player, Option<SimulationEvent>)> {
    let movement = |player: Player| match player.rotation.to_combinator()(&action) {
        Action::Forward => Player {
            position: (player.position.0 + 1, player.position.1),
            ..player
        },
        Action::Backward => Player {
            position: (player.position.0 - 1, player.position.1),
            ..player
        },
        Action::Left => Player {
            position: (player.position.0, player.position.1 - 1),
            ..player
        },
        Action::Right => Player {
            position: (player.position.0, player.position.1 + 1),
            ..player
        },
    };

    players
        .iter()
        .enumerate()
        .map(|(index, player)| {
            let mut player = *player;
            let mut tile = level.get(player.position);

            // Ice & Walls
            loop {
                let next = movement(player);
                let next_tile = level.get(next.position);

                if !matches!(next_tile, Some(Tile::Wall)) {
                    player = next;
                    tile = next_tile;
                }

                if !matches!(next_tile, Some(Tile::Ice)) {
                    break;
                }
            }

            // Rotation Blocks
            player.rotation = match tile {
                Some(Tile::CWRot) => player.rotation.rotate_cw(),
                Some(Tile::CCWRot) => player.rotation.rotate_ccw(),
                _ => player.rotation,
            };

            // Triggers: Finish & Fall
            let event = match tile {
                Some(Tile::Finish) => Some(SimulationEvent::Finished),
                None => Some(SimulationEvent::Died(index)),
                Some(_) => None,
            };

            (player, event)
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use std::{cmp::Ordering, rc::Rc};

    use bevy::utils::HashSet;

    use itertools::Itertools;

    use crate::level::{self, SCENES};

    use super::*;
    use similar_asserts::assert_eq;

    use Action::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Solution {
        pub path: Vec<Action>,
        pub solution_size: usize,
        pub steps: usize,
    }

    fn action_iter(action_list: &[Action]) -> impl Iterator<Item = Action> + Clone {
        Vec::from(action_list).into_iter()
    }

    fn solution_iter(level: &Level) -> impl Iterator<Item = Vec<Action>> {
        let actions = Rc::new(level.actions.clone());

        (1..=level.action_limit).flat_map(move |depth| {
            (1..=depth)
                .map(|_| action_iter(&actions))
                .multi_cartesian_product()
        })
    }

    fn depth_first_search(level: &Level) -> Vec<Solution> {
        let mut solutions = Vec::new();

        let start = level
            .tiles
            .iter()
            .filter(|(_, tile)| matches!(tile, Tile::Start(_)))
            .collect::<Vec<_>>();

        for plan in solution_iter(level) {
            let mut players = start
                .iter()
                .map(|(position, tile)| Player {
                    position: **position,
                    rotation: tile.rotation(),
                })
                .collect::<Vec<_>>();

            let mut previous_positions = HashSet::<(usize, Vec<Player>)>::new();

            for (step_count, (step_index, action)) in plan.iter().enumerate().cycle().enumerate() {
                let new_state = run_simulation_step(level, &players, *action);

                if !previous_positions.insert((
                    step_index,
                    new_state.iter().map(|(player, _)| *player).collect(),
                )) {
                    break;
                }

                players = new_state.iter().map(|(player, _)| *player).collect();

                if new_state
                    .iter()
                    .all(|(_, event)| matches!(event, Some(SimulationEvent::Finished)))
                {
                    solutions.push(Solution {
                        path: plan.clone(),
                        solution_size: plan.len(),
                        steps: step_count + 1,
                    });
                    break;
                }

                if new_state
                    .iter()
                    .any(|(_, event)| matches!(event, Some(SimulationEvent::Died(_))))
                {
                    break;
                }
            }
        }

        solutions
    }

    fn tracing_init() {
        let _ = tracing_subscriber::fmt().pretty().try_init();
    }

    #[test]
    pub fn all_novel_plans() {
        tracing_init();

        let mut all_novel_solutions = solution_iter(&Level {
            action_limit: 5,
            actions: vec![Forward, Backward, Left, Right],
            ..default()
        })
        .map(|plan| {
            ActionPlan(plan.clone())
                .canonicalize_rotation()
                .canonicalize_mirror()
        })
        .into_grouping_map_by(|plan| plan.canonicalize_phase())
        .aggregate(|acc, _key, value| match acc {
            None => Some(vec![value.0]),
            Some(mut acc) => {
                acc.push(value.0);
                Some(acc)
            }
        })
        .into_values()
        .map(|value| {
            let mut value = value.into_iter().collect::<Vec<_>>();
            value.sort();
            value.dedup();
            value
        })
        .collect::<Vec<_>>();

        all_novel_solutions.sort_by(|a, b| match a[0].len().cmp(&b[0].len()) {
            Ordering::Equal => a[0].cmp(&b[0]),
            order => order,
        });

        assert_eq!(
            all_novel_solutions,
            vec![
                vec![vec![Forward]],
                vec![vec![Forward, Forward]],
                vec![vec![Forward, Right]],
                vec![vec![Forward, Backward]],
                vec![vec![Forward, Forward, Forward]],
                vec![
                    vec![Forward, Forward, Right],
                    vec![Forward, Right, Forward],
                    vec![Forward, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Backward],
                    vec![Forward, Backward, Forward],
                    vec![Forward, Backward, Backward]
                ],
                vec![
                    vec![Forward, Right, Backward],
                    vec![Forward, Right, Left],
                    vec![Forward, Backward, Right]
                ],
                vec![vec![Forward, Forward, Forward, Forward]],
                vec![
                    vec![Forward, Forward, Forward, Right],
                    vec![Forward, Forward, Right, Forward],
                    vec![Forward, Right, Forward, Forward],
                    vec![Forward, Right, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Forward, Backward],
                    vec![Forward, Forward, Backward, Forward],
                    vec![Forward, Backward, Forward, Forward],
                    vec![Forward, Backward, Backward, Backward]
                ],
                vec![
                    vec![Forward, Forward, Right, Right],
                    vec![Forward, Right, Right, Forward]
                ],
                vec![
                    vec![Forward, Forward, Right, Backward],
                    vec![Forward, Right, Backward, Forward],
                    vec![Forward, Right, Left, Left],
                    vec![Forward, Backward, Backward, Right]
                ],
                vec![
                    vec![Forward, Forward, Right, Left],
                    vec![Forward, Right, Right, Backward],
                    vec![Forward, Right, Left, Forward],
                    vec![Forward, Backward, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Backward, Right],
                    vec![Forward, Right, Right, Left],
                    vec![Forward, Right, Backward, Backward],
                    vec![Forward, Backward, Right, Forward]
                ],
                vec![
                    vec![Forward, Forward, Backward, Backward],
                    vec![Forward, Backward, Backward, Forward]
                ],
                vec![vec![Forward, Right, Forward, Right]],
                vec![
                    vec![Forward, Right, Forward, Backward],
                    vec![Forward, Right, Left, Right],
                    vec![Forward, Backward, Forward, Right],
                    vec![Forward, Backward, Right, Backward]
                ],
                vec![
                    vec![Forward, Right, Forward, Left],
                    vec![Forward, Right, Backward, Right]
                ],
                vec![vec![Forward, Right, Backward, Left]],
                vec![
                    vec![Forward, Right, Left, Backward],
                    vec![Forward, Backward, Right, Left]
                ],
                vec![vec![Forward, Backward, Forward, Backward]],
                vec![vec![Forward, Forward, Forward, Forward, Forward]],
                vec![
                    vec![Forward, Forward, Forward, Forward, Right],
                    vec![Forward, Forward, Forward, Right, Forward],
                    vec![Forward, Forward, Right, Forward, Forward],
                    vec![Forward, Right, Forward, Forward, Forward],
                    vec![Forward, Right, Right, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Forward, Forward, Backward],
                    vec![Forward, Forward, Forward, Backward, Forward],
                    vec![Forward, Forward, Backward, Forward, Forward],
                    vec![Forward, Backward, Forward, Forward, Forward],
                    vec![Forward, Backward, Backward, Backward, Backward]
                ],
                vec![
                    vec![Forward, Forward, Forward, Right, Right],
                    vec![Forward, Forward, Right, Right, Forward],
                    vec![Forward, Forward, Right, Right, Right],
                    vec![Forward, Right, Right, Forward, Forward],
                    vec![Forward, Right, Right, Right, Forward]
                ],
                vec![
                    vec![Forward, Forward, Forward, Right, Backward],
                    vec![Forward, Forward, Right, Backward, Forward],
                    vec![Forward, Right, Backward, Forward, Forward],
                    vec![Forward, Right, Left, Left, Left],
                    vec![Forward, Backward, Backward, Backward, Right]
                ],
                vec![
                    vec![Forward, Forward, Forward, Right, Left],
                    vec![Forward, Forward, Right, Left, Forward],
                    vec![Forward, Right, Right, Right, Backward],
                    vec![Forward, Right, Left, Forward, Forward],
                    vec![Forward, Backward, Right, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Forward, Backward, Right],
                    vec![Forward, Forward, Backward, Right, Forward],
                    vec![Forward, Right, Right, Right, Left],
                    vec![Forward, Right, Backward, Backward, Backward],
                    vec![Forward, Backward, Right, Forward, Forward]
                ],
                vec![
                    vec![Forward, Forward, Forward, Backward, Backward],
                    vec![Forward, Forward, Backward, Backward, Forward],
                    vec![Forward, Forward, Backward, Backward, Backward],
                    vec![Forward, Backward, Backward, Forward, Forward],
                    vec![Forward, Backward, Backward, Backward, Forward]
                ],
                vec![
                    vec![Forward, Forward, Right, Forward, Right],
                    vec![Forward, Right, Forward, Forward, Right],
                    vec![Forward, Right, Forward, Right, Forward],
                    vec![Forward, Right, Forward, Right, Right],
                    vec![Forward, Right, Right, Forward, Right]
                ],
                vec![
                    vec![Forward, Forward, Right, Forward, Backward],
                    vec![Forward, Right, Forward, Backward, Forward],
                    vec![Forward, Right, Left, Right, Right],
                    vec![Forward, Backward, Forward, Forward, Right],
                    vec![Forward, Backward, Backward, Right, Backward]
                ],
                vec![
                    vec![Forward, Forward, Right, Forward, Left],
                    vec![Forward, Right, Forward, Forward, Left],
                    vec![Forward, Right, Forward, Left, Forward],
                    vec![Forward, Right, Right, Backward, Right],
                    vec![Forward, Right, Backward, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Right, Right, Backward],
                    vec![Forward, Forward, Right, Left, Left],
                    vec![Forward, Right, Right, Backward, Forward],
                    vec![Forward, Right, Left, Left, Forward],
                    vec![Forward, Backward, Backward, Right, Right]
                ],
                vec![
                    vec![Forward, Forward, Right, Right, Left],
                    vec![Forward, Forward, Backward, Right, Right],
                    vec![Forward, Right, Right, Backward, Backward],
                    vec![Forward, Right, Right, Left, Forward],
                    vec![Forward, Backward, Right, Right, Forward]
                ],
                vec![
                    vec![Forward, Forward, Right, Backward, Right],
                    vec![Forward, Right, Forward, Left, Left],
                    vec![Forward, Right, Right, Forward, Left],
                    vec![Forward, Right, Backward, Right, Forward],
                    vec![Forward, Right, Backward, Backward, Right]
                ],
                vec![
                    vec![Forward, Forward, Right, Backward, Backward],
                    vec![Forward, Forward, Backward, Backward, Right],
                    vec![Forward, Right, Right, Left, Left],
                    vec![Forward, Right, Backward, Backward, Forward],
                    vec![Forward, Backward, Backward, Right, Forward]
                ],
                vec![
                    vec![Forward, Forward, Right, Backward, Left],
                    vec![Forward, Right, Right, Backward, Left],
                    vec![Forward, Right, Backward, Backward, Left],
                    vec![Forward, Right, Backward, Left, Forward],
                    vec![Forward, Right, Backward, Left, Left]
                ],
                vec![
                    vec![Forward, Forward, Right, Left, Right],
                    vec![Forward, Right, Right, Forward, Backward],
                    vec![Forward, Right, Left, Right, Forward],
                    vec![Forward, Backward, Forward, Right, Right],
                    vec![Forward, Backward, Right, Right, Backward]
                ],
                vec![
                    vec![Forward, Forward, Right, Left, Backward],
                    vec![Forward, Right, Left, Backward, Forward],
                    vec![Forward, Right, Left, Left, Backward],
                    vec![Forward, Backward, Right, Left, Left],
                    vec![Forward, Backward, Backward, Right, Left]
                ],
                vec![
                    vec![Forward, Forward, Backward, Forward, Right],
                    vec![Forward, Right, Forward, Forward, Backward],
                    vec![Forward, Right, Right, Left, Right],
                    vec![Forward, Backward, Forward, Right, Forward],
                    vec![Forward, Backward, Right, Backward, Backward]
                ],
                vec![
                    vec![Forward, Forward, Backward, Forward, Backward],
                    vec![Forward, Backward, Forward, Forward, Backward],
                    vec![Forward, Backward, Forward, Backward, Forward],
                    vec![Forward, Backward, Forward, Backward, Backward],
                    vec![Forward, Backward, Backward, Forward, Backward]
                ],
                vec![
                    vec![Forward, Forward, Backward, Right, Backward],
                    vec![Forward, Right, Forward, Backward, Backward],
                    vec![Forward, Right, Left, Left, Right],
                    vec![Forward, Backward, Right, Backward, Forward],
                    vec![Forward, Backward, Backward, Forward, Right]
                ],
                vec![
                    vec![Forward, Forward, Backward, Right, Left],
                    vec![Forward, Right, Right, Left, Backward],
                    vec![Forward, Right, Left, Backward, Backward],
                    vec![Forward, Backward, Right, Right, Left],
                    vec![Forward, Backward, Right, Left, Forward]
                ],
                vec![
                    vec![Forward, Right, Forward, Right, Backward],
                    vec![Forward, Right, Forward, Left, Right],
                    vec![Forward, Right, Backward, Forward, Right],
                    vec![Forward, Right, Left, Forward, Left],
                    vec![Forward, Backward, Right, Backward, Right]
                ],
                vec![
                    vec![Forward, Right, Forward, Right, Left],
                    vec![Forward, Right, Forward, Backward, Right],
                    vec![Forward, Right, Backward, Right, Backward],
                    vec![Forward, Right, Left, Forward, Right],
                    vec![Forward, Backward, Right, Forward, Right]
                ],
                vec![
                    vec![Forward, Right, Forward, Backward, Left],
                    vec![Forward, Right, Backward, Right, Left],
                    vec![Forward, Right, Backward, Left, Backward],
                    vec![Forward, Right, Left, Backward, Right],
                    vec![Forward, Backward, Right, Forward, Left]
                ],
                vec![
                    vec![Forward, Right, Forward, Left, Backward],
                    vec![Forward, Right, Backward, Forward, Left],
                    vec![Forward, Right, Backward, Left, Right],
                    vec![Forward, Right, Left, Backward, Left],
                    vec![Forward, Backward, Right, Backward, Left]
                ],
                vec![
                    vec![Forward, Right, Backward, Forward, Backward],
                    vec![Forward, Right, Left, Right, Left],
                    vec![Forward, Backward, Forward, Right, Backward],
                    vec![Forward, Backward, Forward, Backward, Right],
                    vec![Forward, Backward, Right, Forward, Backward]
                ],
                vec![
                    vec![Forward, Right, Left, Forward, Backward],
                    vec![Forward, Right, Left, Right, Backward],
                    vec![Forward, Backward, Forward, Right, Left],
                    vec![Forward, Backward, Right, Left, Right],
                    vec![Forward, Backward, Right, Left, Backward]
                ]
            ]
        );
    }

    fn smallest_solutions(solutions: &[Solution]) -> Vec<Solution> {
        let minimum_size = solutions
            .iter()
            .min_by_key(|solution| solution.solution_size)
            .unwrap()
            .solution_size;

        solutions
            .iter()
            .filter(|solution| solution.solution_size == minimum_size)
            .cloned()
            .collect()
    }

    fn fastest_solutions(solutions: &[Solution]) -> Vec<Solution> {
        let minimum_steps = solutions
            .iter()
            .min_by_key(|solution| solution.steps)
            .unwrap()
            .steps;

        solutions
            .iter()
            .filter(|solution| solution.steps == minimum_steps)
            .cloned()
            .collect()
    }

    fn slowest_solutions(solutions: &[Solution]) -> Vec<Solution> {
        let maximum_steps = solutions
            .iter()
            .max_by_key(|solution| solution.steps)
            .unwrap()
            .steps;

        solutions
            .iter()
            .filter(|solution| solution.steps == maximum_steps)
            .cloned()
            .collect()
    }

    #[test]
    fn basic_test() {
        tracing_init();

        let level = Level::builder()
            .action_limit(1)
            .insert([((0, 0), Tile::DEFAULT_START), ((1, 0), Tile::Finish)])
            .build();

        let solutions = depth_first_search(&level);

        assert_eq!(
            solutions,
            vec![Solution {
                path: vec![Action::Forward],
                solution_size: 1,
                steps: 1,
            }]
        );
    }

    #[test]
    fn basic_test_2() {
        tracing_init();

        let level = Level::builder()
            .action_limit(2)
            .insert([
                ((0, 0), Tile::DEFAULT_START),
                ((1, 0), Tile::Basic),
                ((2, 0), Tile::Finish),
            ])
            .build();

        let solutions = depth_first_search(&level);

        assert_eq!(
            solutions,
            vec![
                Solution {
                    path: vec![Action::Forward],
                    solution_size: 1,
                    steps: 2,
                },
                Solution {
                    path: vec![Action::Forward, Action::Forward],
                    solution_size: 2,
                    steps: 2,
                }
            ]
        );
    }

    #[test]
    fn basic_test_3() {
        tracing_init();

        let level = Level::builder()
            .action_limit(2)
            .insert([
                ((0, 0), Tile::DEFAULT_START),
                ((1, 0), Tile::Basic),
                ((1, 1), Tile::Basic),
                ((2, 1), Tile::Finish),
            ])
            .build();

        let solutions = depth_first_search(&level);

        assert_eq!(
            solutions,
            vec![Solution {
                path: vec![Action::Forward, Action::Right],
                solution_size: 2,
                steps: 3,
            },]
        );
    }

    fn level_from_name(name: &str) -> &Level {
        SCENES
            .iter()
            .find_map(|scene| match scene {
                level::Scene::Level(level) if level.name == name => Some(level),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn level_lost() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_name("Lost")),
            vec![Solution {
                path: vec![Action::Forward],
                solution_size: 1,
                steps: 4
            }]
        );
    }

    #[test]
    fn level_arbitrary() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_name("Arbitrary")),
            vec![
                Solution {
                    path: vec![Action::Forward, Action::Right],
                    solution_size: 2,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Right, Action::Forward],
                    solution_size: 2,
                    steps: 6
                }
            ]
        );
    }

    #[test]
    fn level_pothole() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_name("Pothole")),
            vec![Solution {
                path: vec![Action::Left, Action::Forward],
                solution_size: 2,
                steps: 6
            }]
        );
    }

    #[test]
    fn level_noise() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_name("Noise")),
            vec![Solution {
                path: vec![Left, Left, Forward],
                solution_size: 3,
                steps: 6
            }]
        );
    }

    #[test]
    fn level_obstructions() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_name("Obstructions")),
            vec![
                Solution {
                    path: vec![Action::Forward, Action::Right],
                    solution_size: 2,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Right, Action::Forward],
                    solution_size: 2,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Forward, Action::Forward, Action::Right],
                    solution_size: 3,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Forward, Action::Right, Action::Right],
                    solution_size: 3,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Right, Action::Forward, Action::Forward],
                    solution_size: 3,
                    steps: 6
                },
                Solution {
                    path: vec![Action::Right, Action::Right, Action::Forward],
                    solution_size: 3,
                    steps: 6
                }
            ]
        );
    }

    #[test]
    fn level_choices() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Choices"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Action::Forward, Action::Forward, Action::Right],
                    solution_size: 3,
                    steps: 14
                },
                Solution {
                    path: vec![Action::Forward, Action::Right, Action::Forward],
                    solution_size: 3,
                    steps: 13
                },
                Solution {
                    path: vec![Action::Right, Action::Forward, Action::Forward],
                    solution_size: 3,
                    steps: 12
                }
            ]
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Forward, Forward],
                solution_size: 4,
                steps: 8
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            [Solution {
                path: vec![Forward, Forward, Right, Left, Right],
                solution_size: 5,
                steps: 22
            }]
        );
    }

    #[test]
    fn level_precarious() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Precarious"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Forward, Forward],
                solution_size: 4,
                steps: 8
            }]
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Forward, Forward],
                solution_size: 4,
                steps: 8
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            [Solution {
                path: vec![Forward, Forward, Forward, Left, Backward],
                solution_size: 5,
                steps: 13
            }]
        );
    }

    #[test]
    fn level_hook() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Hook"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Forward, Right, Left],
                    solution_size: 4,
                    steps: 7
                },
                Solution {
                    path: vec![Forward, Left, Forward, Right],
                    solution_size: 4,
                    steps: 12
                },
                Solution {
                    path: vec![Left, Forward, Forward, Right],
                    solution_size: 4,
                    steps: 8
                }
            ]
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Forward, Right, Left],
                solution_size: 4,
                steps: 7
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Forward, Right, Forward],
                    solution_size: 5,
                    steps: 14
                },
                Solution {
                    path: vec![Forward, Left, Forward, Right, Right],
                    solution_size: 5,
                    steps: 14
                }
            ]
        );
    }

    #[test]
    fn level_crucible() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Crucible"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Forward, Right, Right, Backward, Forward, Left,],
                    solution_size: 7,
                    steps: 19,
                },
                Solution {
                    path: vec![Forward, Right, Right, Backward, Forward, Left, Forward,],
                    solution_size: 7,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Left, Forward, Forward, Right, Right, Backward,],
                    solution_size: 7,
                    steps: 14,
                },
                Solution {
                    path: vec![Left, Forward, Forward, Right, Right, Backward, Forward,],
                    solution_size: 7,
                    steps: 20,
                },
            ]
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left, Forward, Forward, Right, Right, Backward,],
                solution_size: 7,
                steps: 14,
            },]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Right, Forward, Right, Right, Backward, Forward,],
                solution_size: 8,
                steps: 23,
            },]
        );
    }

    #[test]
    fn level_rift() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Rift"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Forward, Right, Right, Backward],
                solution_size: 6,
                steps: 24
            }],
            "smallest"
        );
        assert_eq!(
            fastest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Right, Backward, Right, Forward, Forward, Forward, Left],
                    solution_size: 7,
                    steps: 14
                },
                Solution {
                    path: vec![Left, Forward, Forward, Forward, Right, Backward, Right],
                    solution_size: 7,
                    steps: 14
                },
                Solution {
                    path: vec![Left, Forward, Forward, Right, Right, Right, Backward],
                    solution_size: 7,
                    steps: 14
                }
            ],
            "fastest"
        );
        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Forward, Right, Right, Backward],
                solution_size: 6,
                steps: 24
            }],
            "slowest"
        );
    }

    #[test]
    fn level_overshoot() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Overshoot"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left, Backward, Right,],
                solution_size: 4,
                steps: 4,
            }],
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        // assert_eq!(fastest_solutions(&solutions), vec![]);

        // assert_eq!(slowest_solutions(&solutions), vec![]);
    }

    #[test]
    fn level_glide() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Glide"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Backward, Left, Forward, Right],
                solution_size: 4,
                steps: 4,
            }],
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        // assert_eq!(fastest_solutions(&solutions), vec![]);

        // assert_eq!(slowest_solutions(&solutions), vec![]);
    }

    #[test]
    fn level_loops() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Loops"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Backward, Right],
                    solution_size: 4,
                    steps: 13
                },
                Solution {
                    path: vec![Left, Backward, Right, Forward],
                    solution_size: 4,
                    steps: 12
                }
            ],
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            [Solution {
                path: vec![Left, Backward, Right, Forward, Forward],
                solution_size: 5,
                steps: 9,
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left, Backward, Right],
                solution_size: 4,
                steps: 13,
            },]
        );
    }

    #[test]
    fn level_gauntlet() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Gauntlet"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Right, Forward, Left, Forward, Backward,],
                    solution_size: 5,
                    steps: 9,
                },
                Solution {
                    path: vec![Left, Forward, Backward, Right, Forward,],
                    solution_size: 5,
                    steps: 7,
                },
            ]
        );

        // assert_eq!(largest_solutions(&solutions), vec![]);

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Backward, Right, Forward,],
                solution_size: 5,
                steps: 7,
            },]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Left, Forward, Backward,],
                solution_size: 5,
                steps: 9,
            }]
        );
    }

    #[test]
    fn level_esky() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Esky"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Backward, Left],
                    solution_size: 4,
                    steps: 13
                },
                Solution {
                    path: vec![Backward, Left, Forward, Right],
                    solution_size: 4,
                    steps: 15
                },
                Solution {
                    path: vec![Left, Forward, Right, Backward],
                    solution_size: 4,
                    steps: 14
                }
            ],
            "smallest"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Forward, Backward, Left],
                solution_size: 5,
                steps: 8
            }],
            "fastest"
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Backward, Left, Forward, Right],
                solution_size: 4,
                steps: 15
            }],
            "slowest"
        );
    }

    #[test]
    fn level_divert() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Divert"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward],
                solution_size: 1,
                steps: 4
            }]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward],
                solution_size: 1,
                steps: 4
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward],
                solution_size: 1,
                steps: 4
            }]
        );
    }

    #[test]
    fn level_pivot() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Pivot"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left],
                solution_size: 2,
                steps: 2
            }],
            "smallest solutions"
        );

        // assert_eq!(
        //     fastest_solutions(&solutions),
        //     vec![Solution {
        //         path: vec![Forward, Left],
        //         solution_size: 2,
        //         steps: 2
        //     }],
        //     "fastest solutions",
        // );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Left, Backward],
                solution_size: 4,
                steps: 4
            }],
            "slowest solutions",
        );
    }

    #[test]
    fn level_twirl() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Twirl"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Forward, Right,],
                    solution_size: 4,
                    steps: 10,
                },
                Solution {
                    path: vec![Forward, Left, Backward, Right,],
                    solution_size: 4,
                    steps: 6,
                },
                Solution {
                    path: vec![Forward, Left, Backward, Left,],
                    solution_size: 4,
                    steps: 10,
                },
            ],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left, Backward, Right,],
                solution_size: 4,
                steps: 6,
            }],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Forward, Right,],
                    solution_size: 4,
                    steps: 10,
                },
                Solution {
                    path: vec![Forward, Left, Backward, Left,],
                    solution_size: 4,
                    steps: 10,
                }
            ],
            "slowest solutions",
        );
    }

    #[test]
    fn level_dizzy() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Dizzy"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Left, Backward],
                solution_size: 4,
                steps: 8,
            }],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Left, Backward],
                solution_size: 4,
                steps: 8,
            }],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Left, Backward],
                solution_size: 4,
                steps: 8,
            }],
            "slowest solutions",
        );
    }

    #[test]
    fn level_zigzag() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("ZigZag"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Left, Forward,],
                solution_size: 4,
                steps: 10,
            },],
            "smallest solutions"
        );

        // assert_eq!(fastest_solutions(&solutions), vec![], "fastest solutions",);

        assert_eq!(
            slowest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Forward, Left, Right, Forward, Left, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Forward, Left, Right, Backward, Right, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Forward, Left, Right, Backward, Backward, Right,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Forward, Left, Backward, Right, Left, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Forward, Forward, Left, Right, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Forward, Forward, Left, Backward, Right,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Forward, Left, Forward, Left, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Forward, Left, Backward, Right, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Forward, Left, Backward, Backward, Right,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Backward, Right, Forward, Left, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Backward, Right, Backward, Right, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Backward, Right, Backward, Backward, Right,],
                    solution_size: 8,
                    steps: 18,
                },
                Solution {
                    path: vec![Forward, Right, Left, Backward, Backward, Right, Left, Forward,],
                    solution_size: 8,
                    steps: 18,
                },
            ],
            "slowest solutions",
        );
    }

    #[test]
    fn level_binary() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Binary"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right],
                solution_size: 2,
                steps: 7,
            },],
            "smallest solutions"
        );

        // // assert_eq!(fastest_solutions(&solutions), vec![], "fastest solutions",);

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Backward, Right, Backward, Right,],
                solution_size: 6,
                steps: 19,
            },],
            "slowest solutions",
        );
    }

    #[test]
    fn level_two_step() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Two-Step"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Forward, Left,],
                    solution_size: 4,
                    steps: 17,
                },
                Solution {
                    path: vec![Forward, Right, Backward, Right,],
                    solution_size: 4,
                    steps: 7,
                },
            ],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Backward, Right,],
                solution_size: 4,
                steps: 7,
            },],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Forward, Left,],
                solution_size: 4,
                steps: 17,
            },],
            "slowest solutions",
        );
    }

    #[test]
    fn level_chess() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Chess"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Right, Forward, Right,],
                solution_size: 4,
                steps: 9,
            },],
            "smallest solutions"
        );

        // assert_eq!(
        //     fastest_solutions(&solutions),
        //     vec![Solution {
        //         path: vec![Left, Right, Forward, Right,],
        //         solution_size: 4,
        //         steps: 9,
        //     }],
        //     "fastest solutions",
        // );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Right, Backward, Right, Right],
                solution_size: 5,
                steps: 25,
            }],
            "slowest solutions",
        );
    }

    #[test]
    fn level_restricted() {
        tracing_init();

        let level = level_from_name("Restricted");
        let solutions = depth_first_search(level);

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward],
                solution_size: level.command_challenge.unwrap_or_default(),
                steps: 6
            }],
            "smallest solutions"
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Forward, Backward],
                solution_size: 4,
                steps: level.waste_challenge.unwrap_or_default()
            }],
            "slowest solutions",
        );
    }

    #[test]
    fn level_progress() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Progress"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward],
                solution_size: 1,
                steps: 4,
            },],
            "smallest solutions"
        );

        // assert_eq!(
        //     fastest_solutions(&solutions),
        //     vec![
        //         Solution {
        //             path: vec![Forward],
        //             solution_size: 1,
        //             steps: 4,
        //         },
        //         Solution {
        //             path: vec![Forward, Forward],
        //             solution_size: 2,
        //             steps: 4,
        //         }
        //     ],
        //     "fastest solutions",
        // );

        // assert_eq!(slowest_solutions(&solutions), vec![], "slowest solutions",);
    }

    #[test]
    fn level_support() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Support"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Forward, Right],
                solution_size: 3,
                steps: 6
            }],
            "smallest solutions"
        );

        // assert_eq!(fastest_solutions(&solutions), vec![], "fastest solutions",);

        // assert_eq!(slowest_solutions(&solutions), vec![], "slowest solutions",);
    }

    #[test]
    fn level_snail() {
        tracing_init();

        let level = level_from_name("Snail");
        let solutions = depth_first_search(level);

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Left, Backward,],
                solution_size: 3,
                steps: 7,
            },],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Left, Backward,],
                solution_size: 3,
                steps: 7,
            },],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Left, Backward,],
                solution_size: 3,
                steps: 7,
            },],
            "slowest solutions",
        );
    }

    #[test]
    fn level_trapped() {
        tracing_init();

        let level = level_from_name("Trapped");
        let solutions = depth_first_search(level);

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Right, Backward],
                solution_size: level.command_challenge.unwrap_or_default(),
                steps: 9
            },],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Left, Left, Forward],
                solution_size: 4,
                steps: level.step_challenge.unwrap_or_default()
            },],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Backward, Backward, Right,],
                solution_size: 5,
                steps: level.waste_challenge.unwrap_or_default(),
            },],
            "slowest solutions",
        );
    }

    #[test]
    fn level_squeeze() {
        tracing_init();

        let level = level_from_name("Squeeze");
        let solutions = depth_first_search(level);

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Forward, Left, Right],
                    solution_size: level.command_challenge.unwrap_or_default(),
                    steps: 21
                },
                Solution {
                    path: vec![Left, Right, Forward, Right, Forward],
                    solution_size: level.command_challenge.unwrap_or_default(),
                    steps: 23
                }
            ],
            "smallest solutions"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Right, Forward, Forward, Left, Right],
                solution_size: 6,
                steps: level.step_challenge.unwrap_or_default()
            }],
            "fastest solutions",
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Right, Forward, Right, Forward],
                solution_size: 5,
                steps: level.waste_challenge.unwrap_or_default(),
            }],
            "slowest solutions",
        );
    }

    #[ignore]
    #[test]
    fn level_spinors() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Spinors"));

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Backward],
                solution_size: 2,
                steps: 12
            }]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Backward, Forward, Right, Left],
                    solution_size: 5,
                    steps: 10
                },
                Solution {
                    path: vec![Left, Forward, Forward, Right, Forward],
                    solution_size: 5,
                    steps: 10
                }
            ]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Forward, Left, Left, Backward, Right],
                solution_size: 5,
                steps: 36
            }]
        );
    }

    #[test]
    fn level_popsicle() {
        tracing_init();

        let level = level_from_name("Popsicle");
        let solutions = depth_first_search(level);

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Left, Left, Right, Right, Right],
                    solution_size: level.command_challenge.unwrap_or_default(),
                    steps: 17,
                },
                Solution {
                    path: vec![Left, Left, Left, Right, Right,],
                    solution_size: level.command_challenge.unwrap_or_default(),
                    steps: 20,
                },
            ]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Left, Right, Right, Left, Left],
                solution_size: 6,
                steps: level.step_challenge.unwrap_or_default()
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Left, Forward, Backward, Right, Forward, Left],
                solution_size: 7,
                steps: level.waste_challenge.unwrap_or_default(),
            },]
        );
    }

    #[test]
    fn level_swirl() {
        tracing_init();

        let level = level_from_name("Swirl");
        let solutions = depth_first_search(level);

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Backward, Backward,],
                solution_size: level.command_challenge.unwrap_or_default(),
                steps: 16,
            },],
            "smallest solution"
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Right, Forward, Right,],
                solution_size: 5,
                steps: level.step_challenge.unwrap_or_default(),
            },],
            "fastest solution"
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Backward, Backward,],
                solution_size: 3,
                steps: level.waste_challenge.unwrap_or_default(),
            },],
            "slowest solution"
        );
    }

    #[test]
    fn level_blizzard() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Blizzard"));

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));
        // panic!();

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Backward],
                    solution_size: 3,
                    steps: 37
                },
                Solution {
                    path: vec![Forward, Right, Left],
                    solution_size: 3,
                    steps: 36
                },
                Solution {
                    path: vec![Forward, Backward, Left],
                    solution_size: 3,
                    steps: 32
                },
                Solution {
                    path: vec![Right, Backward, Forward],
                    solution_size: 3,
                    steps: 36
                },
                Solution {
                    path: vec![Right, Backward, Left],
                    solution_size: 3,
                    steps: 37
                },
                Solution {
                    path: vec![Right, Left, Forward],
                    solution_size: 3,
                    steps: 35
                },
                Solution {
                    path: vec![Backward, Forward, Right],
                    solution_size: 3,
                    steps: 38
                },
                Solution {
                    path: vec![Backward, Left, Forward],
                    solution_size: 3,
                    steps: 31
                },
                Solution {
                    path: vec![Backward, Left, Right],
                    solution_size: 3,
                    steps: 30
                },
                Solution {
                    path: vec![Left, Forward, Right],
                    solution_size: 3,
                    steps: 28
                },
                Solution {
                    path: vec![Left, Forward, Backward],
                    solution_size: 3,
                    steps: 27
                },
                Solution {
                    path: vec![Left, Right, Backward],
                    solution_size: 3,
                    steps: 32
                }
            ]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Backward, Backward, Forward, Right, Backward, Right],
                solution_size: 8,
                steps: 8
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Right, Backward, Forward, Backward, Forward, Right, Backward],
                solution_size: 8,
                steps: 70
            }]
        );
    }

    #[test]
    fn mirror() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Mirror"));

        assert_eq!(
            solutions,
            vec![Solution {
                path: vec![Forward],
                solution_size: 1,
                steps: 3
            }]
        );
    }

    // #[test]
    // fn staggerd() {
    //     tracing_init();

    //     let solutions = depth_first_search(level_from_name("Staggerd"));

    //     assert_eq!(
    //         solutions,
    //         vec![Solution {
    //             path: vec![Forward],
    //             solution_size: 1,
    //             steps: 6
    //         }]
    //     );
    // }

    #[test]
    fn convergence() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Convergence"));

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));
        // panic!();

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Right, Left],
                    solution_size: 4,
                    steps: 21
                },
                Solution {
                    path: vec![Forward, Left, Left, Right],
                    solution_size: 4,
                    steps: 21
                },
                Solution {
                    path: vec![Right, Right, Left, Forward],
                    solution_size: 4,
                    steps: 20
                },
                Solution {
                    path: vec![Left, Left, Right, Forward],
                    solution_size: 4,
                    steps: 20
                }
            ]
        );
        assert_eq!(
            fastest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Right, Forward, Left, Forward, Forward, Right],
                    solution_size: 6,
                    steps: 11
                },
                Solution {
                    path: vec![Right, Right, Left, Forward, Forward, Forward],
                    solution_size: 6,
                    steps: 11
                },
                Solution {
                    path: vec![Left, Forward, Right, Forward, Forward, Left],
                    solution_size: 6,
                    steps: 11
                },
                Solution {
                    path: vec![Left, Left, Right, Forward, Forward, Forward],
                    solution_size: 6,
                    steps: 11
                }
            ]
        );
        assert_eq!(
            slowest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Right, Right, Right, Left],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Right, Right, Left, Right, Left],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Right, Right, Left, Left, Right],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Right, Left, Right, Right, Left],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Right, Left, Left, Left, Right],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Left, Right, Right, Right, Left],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Left, Right, Left, Left, Right],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Left, Left, Right, Right, Left],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Left, Left, Right, Left, Right],
                    solution_size: 6,
                    steps: 31
                },
                Solution {
                    path: vec![Forward, Left, Left, Left, Left, Right],
                    solution_size: 6,
                    steps: 31
                }
            ]
        );
    }

    #[test]
    fn perpendicular() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Perpendicular"));

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));
        // panic!();

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Forward, Right],
                    solution_size: 4,
                    steps: 7
                },
                Solution {
                    path: vec![Left, Forward, Right, Forward],
                    solution_size: 4,
                    steps: 9
                }
            ]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Left, Forward, Right],
                    solution_size: 4,
                    steps: 7
                },
                Solution {
                    path: vec![Forward, Left, Forward, Right, Forward, Left],
                    solution_size: 6,
                    steps: 7
                }
            ]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Left, Backward, Left, Forward],
                solution_size: 6,
                steps: 15
            }]
        );
    }

    #[test]
    fn transcendence() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Transcendence"));

        // tracing::info!(smallest = ?smallest_solutions(&solutions));
        // tracing::info!(fastest = ?fastest_solutions(&solutions));
        // tracing::info!(slowest = ?slowest_solutions(&solutions));
        // panic!();

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Forward, Left, Left, Forward],
                    solution_size: 6,
                    steps: 18
                },
                Solution {
                    path: vec![Forward, Right, Left, Left, Forward, Right],
                    solution_size: 6,
                    steps: 23
                },
                Solution {
                    path: vec![Forward, Right, Left, Left, Right, Forward],
                    solution_size: 6,
                    steps: 19
                },
                Solution {
                    path: vec![Forward, Left, Forward, Right, Right, Forward],
                    solution_size: 6,
                    steps: 18
                },
                Solution {
                    path: vec![Forward, Left, Right, Right, Forward, Left],
                    solution_size: 6,
                    steps: 23
                },
                Solution {
                    path: vec![Forward, Left, Right, Right, Left, Forward],
                    solution_size: 6,
                    steps: 19
                }
            ]
        );
        assert_eq!(
            fastest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Forward, Right, Forward, Left, Left, Forward],
                    solution_size: 6,
                    steps: 18
                },
                Solution {
                    path: vec![Forward, Left, Forward, Right, Right, Forward],
                    solution_size: 6,
                    steps: 18
                }
            ]
        );
        println!("{:?}", slowest_solutions(&solutions));
        assert_eq!(
            slowest_solutions(&solutions),
            [
                Solution {
                    path: vec![Forward, Right, Left, Left, Forward, Right],
                    solution_size: 6,
                    steps: 23
                },
                Solution {
                    path: vec![Forward, Left, Right, Right, Forward, Left],
                    solution_size: 6,
                    steps: 23
                }
            ]
        );
    }

    #[test]
    fn samsara() {
        tracing_init();

        let solutions = depth_first_search(level_from_name("Samsara"));

        assert_eq!(
            smallest_solutions(&solutions),
            vec![
                Solution {
                    path: vec![Backward, Backward, Right],
                    solution_size: 3,
                    steps: 16
                },
                Solution {
                    path: vec![Left, Backward, Left],
                    solution_size: 3,
                    steps: 15
                }
            ]
        );

        assert_eq!(
            fastest_solutions(&solutions),
            vec![Solution {
                path: vec![Left, Forward, Backward, Left, Backward, Right],
                solution_size: 6,
                steps: 11
            }]
        );

        assert_eq!(
            slowest_solutions(&solutions),
            vec![Solution {
                path: vec![Right, Forward, Backward, Left, Right, Backward],
                solution_size: 6,
                steps: 55
            }]
        );
    }
}
