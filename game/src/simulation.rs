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

pub enum SimulationEvent {
    Finished,
    Died,
}

pub fn run_simulation_step(
    level: &Level,
    player: Player,
    action: Action,
) -> (Player, Option<SimulationEvent>) {
    let movement = |position: (i32, i32)| match action {
        Action::Forward => (position.0 + 1, position.1),
        Action::Backward => (position.0 - 1, position.1),
        Action::Left => (position.0, position.1 - 1),
        Action::Right => (position.0, position.1 + 1),
        Action::Nothing => (position.0, position.1),
    };

    let mut position = player.position;

    loop {
        let next = movement(position);
        match level.get(next) {
            Some(Tile::Ice) => {
                position = next;
            }
            Some(Tile::Finish | Tile::Start | Tile::Basic) | None => {
                position = next;
                break;
            }
            Some(Tile::Wall) => break,
        }
    }

    match level.get(position) {
        Some(Tile::Finish) => (Player { position }, Some(SimulationEvent::Finished)),
        Some(Tile::Wall) => (player, None),
        Some(Tile::Basic | Tile::Start) => (Player { position }, None),
        Some(Tile::Ice) => (Player { position }, None),
        None => (Player { position }, Some(SimulationEvent::Died)),
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

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

        let (start, _) = level
            .tiles
            .iter()
            .find(|(_, tile)| **tile == Tile::Start)
            .unwrap();

        for plan in solution_iter(level) {
            let mut player = Player { position: *start };

            let mut previous_positions = HashSet::<(usize, Player)>::new();

            for (step_count, (step_index, action)) in plan.iter().enumerate().cycle().enumerate() {
                let (new_player, event) = run_simulation_step(level, player, *action);

                if !previous_positions.insert((step_index, player)) {
                    break;
                }

                player = new_player;

                match event {
                    Some(SimulationEvent::Finished) => {
                        solutions.push(Solution {
                            path: plan.clone(),
                            solution_size: plan.len(),
                            steps: step_count + 1,
                        });
                        break;
                    }
                    Some(SimulationEvent::Died) => break,
                    None => (),
                }
            }
        }

        solutions
    }

    fn tracing_init() {
        let _ = tracing_subscriber::fmt().pretty().try_init();
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

    // fn largest_solutions(solutions: &[Solution]) -> Vec<Solution> {
    //     let maximum_size = solutions
    //         .iter()
    //         .max_by_key(|solution| solution.solution_size)
    //         .unwrap()
    //         .solution_size;

    //     solutions
    //         .iter()
    //         .filter(|solution| solution.solution_size == maximum_size)
    //         .cloned()
    //         .collect()
    // }

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
            .insert([((0, 0), Tile::Start), ((1, 0), Tile::Finish)])
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
                ((0, 0), Tile::Start),
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
                ((0, 0), Tile::Start),
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

    fn level_from_scene(scene: &level::Scene) -> &Level {
        match scene {
            level::Scene::Level(level) => level,
            _ => panic!("expected level scene"),
        }
    }

    #[test]
    fn level_one() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_scene(&SCENES[0])),
            vec![Solution {
                path: vec![Action::Forward],
                solution_size: 1,
                steps: 4
            }]
        );
    }

    #[test]
    fn level_two() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_scene(&SCENES[1])),
            vec![
                Solution {
                    path: vec![Action::Forward, Action::Right],
                    solution_size: 2,
                    steps: 8
                },
                Solution {
                    path: vec![Action::Right, Action::Forward],
                    solution_size: 2,
                    steps: 8
                }
            ]
        );
    }

    #[test]
    fn level_three() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_scene(&SCENES[2])),
            vec![Solution {
                path: vec![Action::Left, Action::Forward],
                solution_size: 2,
                steps: 8
            }]
        );
    }

    #[test]
    fn level_four() {
        tracing_init();

        assert_eq!(
            depth_first_search(level_from_scene(&SCENES[3])),
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
    fn level_five() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[4]));

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
    fn level_six() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[5]));

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
    fn level_seven() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[6]));

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
    fn level_eight() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[7]));

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
    fn level_nine() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[8]));

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
    fn level_ten() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[9]));

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
    fn level_eleven() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[10]));

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
    fn level_twelve() {
        tracing_init();

        let solutions = depth_first_search(level_from_scene(&SCENES[11]));

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
}
