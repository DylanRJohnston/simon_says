use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

impl Orientation {
    fn turn(&self, turn: Turn) -> Orientation {
        match (self, turn) {
            (Orientation::Up, Turn::Left) => Orientation::Left,
            (Orientation::Up, Turn::Right) => Orientation::Right,
            (Orientation::Down, Turn::Left) => Orientation::Right,
            (Orientation::Down, Turn::Right) => Orientation::Left,
            (Orientation::Left, Turn::Left) => Orientation::Down,
            (Orientation::Left, Turn::Right) => Orientation::Up,
            (Orientation::Right, Turn::Left) => Orientation::Up,
            (Orientation::Right, Turn::Right) => Orientation::Down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Turn {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Pipe {
    Straight(Orientation),
    Bend(Orientation, Turn),
}

impl Pipe {
    fn next_position(&self, position: Position) -> Position {
        match self {
            &Pipe::Straight(orientation) => position.move_towards(orientation),
            &Pipe::Bend(orientation, turn) => position.move_towards(orientation.turn(turn)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Forwards,
    Backwards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Piece {
    Pipe(Pipe),
    CrossOver {
        horizontal: Direction,
        vertical: Direction,
    },
}

impl Piece {
    fn all_possible_pieces() -> impl Iterator<Item = Piece> {
        [
            Piece::Pipe(Pipe::Straight(Orientation::Up)),
            Piece::Pipe(Pipe::Straight(Orientation::Down)),
            Piece::Pipe(Pipe::Straight(Orientation::Left)),
            Piece::Pipe(Pipe::Straight(Orientation::Right)),
            Piece::Pipe(Pipe::Bend(Orientation::Up, Turn::Left)),
            Piece::Pipe(Pipe::Bend(Orientation::Up, Turn::Right)),
            Piece::Pipe(Pipe::Bend(Orientation::Down, Turn::Left)),
            Piece::Pipe(Pipe::Bend(Orientation::Down, Turn::Right)),
            Piece::Pipe(Pipe::Bend(Orientation::Left, Turn::Left)),
            Piece::Pipe(Pipe::Bend(Orientation::Left, Turn::Right)),
            Piece::Pipe(Pipe::Bend(Orientation::Right, Turn::Left)),
            Piece::Pipe(Pipe::Bend(Orientation::Right, Turn::Right)),
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Component {
    Source(Orientation),
    Sink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Element {
    Piece(Piece),
    Component(Component),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Puzzle {
    dimension: (usize, usize),
    pieces: HashMap<Position, Element>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: i32,
    col: i32,
}

impl Position {
    fn new(row: i32, col: i32) -> Self {
        Position { row, col }
    }

    fn move_towards(&self, orientation: Orientation) -> Position {
        match orientation {
            Orientation::Up => Position {
                row: self.row - 1,
                col: self.col,
            },
            Orientation::Down => Position {
                row: self.row + 1,
                col: self.col,
            },
            Orientation::Left => Position {
                row: self.row,
                col: self.col - 1,
            },
            Orientation::Right => Position {
                row: self.row,
                col: self.col + 1,
            },
        }
    }
}

impl Puzzle {
    fn sources(&self) -> impl Iterator<Item = (Position, Orientation)> {
        self.pieces
            .iter()
            .filter_map(|(&position, element)| match element {
                &Element::Component(Component::Source(orientation)) => {
                    Some((position, orientation))
                }
                _ => None,
            })
    }

    fn sinks(&self) -> impl Iterator<Item = Position> {
        self.pieces
            .iter()
            .filter_map(|(&position, element)| match element {
                &Element::Component(Component::Sink) => Some(position),
                _ => None,
            })
    }
}

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn solve(puzzle: Puzzle) -> Result<Vec<Puzzle>> {
    let next_pieces = |element: Element| {
        let pieces = |orientation| {
            vec![
                Element::Piece(Piece::Pipe(Pipe::Straight(orientation))),
                Element::Piece(Piece::Pipe(Pipe::Bend(orientation, Turn::Left))),
                Element::Piece(Piece::Pipe(Pipe::Bend(orientation, Turn::Right))),
            ]
        };

        match element {
            Element::Component(Component::Sink) => unimplemented!(),
            Element::Component(Component::Source(orientation)) => pieces(orientation),
            Element::Piece(Piece::Pipe(Pipe::Straight(orientation))) => pieces(orientation),
            Element::Piece(Piece::Pipe(Pipe::Bend(orientation, turn))) => {
                pieces(orientation.turn(turn))
            }
            Element::Piece(Piece::CrossOver { .. }) => todo!(),
        }
    };

    let inner_solve = |puzzle: Puzzle, head: (Position, Element)| {};

    let (position, orientation) = puzzle.sources().next().unwrap();

    inner_solve(
        puzzle,
        (position, Element::Component(Component::Source(orientation))),
    );

    todo!()
}

fn is_solved(puzzle: &Puzzle) -> bool {
    let mut connected_sinks = HashSet::<Position>::new();

    for (source_position, source_orientation) in puzzle.sources() {
        let mut position = source_position.move_towards(source_orientation);
        let mut prev_orientation = source_orientation;

        loop {
            // Dead end
            let Some(next_piece) = puzzle.pieces.get(&position) else {
                return false;
            };

            match next_piece {
                Element::Component(Component::Source(_)) => return false,
                Element::Component(Component::Sink) => {
                    connected_sinks.insert(position);
                    break;
                }
                &Element::Piece(Piece::Pipe(pipe @ Pipe::Straight(orientation))) => {
                    if orientation != prev_orientation {
                        return false;
                    }

                    position = pipe.next_position(position);
                }
                &Element::Piece(Piece::Pipe(pipe @ Pipe::Bend(orientation, turn))) => {
                    if orientation != prev_orientation {
                        return false;
                    }

                    prev_orientation = orientation.turn(turn);
                    position = pipe.next_position(position);
                }
                other => todo!("{:?}", other),
            }
        }
    }

    puzzle
        .sinks()
        .all(|position| connected_sinks.contains(&position))
}

fn from_pictogram(pictogram: &[&str]) -> Result<Puzzle> {
    let mut pieces = HashMap::new();

    for (row_index, row) in pictogram.iter().enumerate() {
        for (col_index, col) in row.split_ascii_whitespace().enumerate() {
            let element = match col {
                "RR" => Element::Component(Component::Source(Orientation::Right)),
                "LL" => Element::Component(Component::Source(Orientation::Left)),
                "UU" => Element::Component(Component::Source(Orientation::Up)),
                "DD" => Element::Component(Component::Source(Orientation::Down)),
                "SS" => Element::Component(Component::Sink),
                "rr" => Element::Piece(Piece::Pipe(Pipe::Straight(Orientation::Right))),
                "uu" => Element::Piece(Piece::Pipe(Pipe::Straight(Orientation::Up))),
                "ll" => Element::Piece(Piece::Pipe(Pipe::Straight(Orientation::Left))),
                "dd" => Element::Piece(Piece::Pipe(Pipe::Straight(Orientation::Down))),
                "rl" => Element::Piece(Piece::Pipe(Pipe::Bend(Orientation::Right, Turn::Left))),
                ".." => continue,
                other => Err(format!("Invalid character in pictogram: `{}`", other))?,
            };
            pieces.insert(Position::new(row_index as i32, col_index as i32), element);
        }
    }

    Ok(Puzzle {
        dimension: (pictogram.len(), (pictogram[0].len() + 1) / 3),
        pieces,
    })
}

#[cfg(test)]
mod test {

    mod pictogram {
        use super::super::*;

        #[test]
        fn basic() -> Result<()> {
            assert_eq!(
                from_pictogram(&["RR rr SS"])?,
                Puzzle {
                    dimension: (1, 3),
                    pieces: HashMap::from([
                        (
                            Position::new(0, 0),
                            Element::Component(Component::Source(Orientation::Right))
                        ),
                        (
                            Position::new(0, 1),
                            Element::Piece(Piece::Pipe(Pipe::Straight(Orientation::Right)))
                        ),
                        (Position::new(0, 2), Element::Component(Component::Sink),),
                    ]),
                }
            );

            Ok(())
        }
    }

    mod solver {
        use super::super::*;

        #[test]
        fn is_solved_test() -> Result {
            let examples: &[(&[&str], bool)] = &[
                (&[""], true),
                (&["RR SS"], true),
                (&["RR .. SS"], false),
                (&["RR rr SS"], true),
                (&["RR uu SS"], false),
                (&["RR rr rr SS"], true),
                (&["RR rr uu SS"], false),
                (&["RR rr .. rr SS"], false),
                (&["RR rr rr SS SS"], false),
                (&["RR rr SS", "RR rr SS"], true),
                (
                    #[rustfmt::skip]
                    &[
                        "RR rr SS",
                        ".. .. uu",
                        ".. .. UU",
                    ],
                    true,
                ),
                (
                    #[rustfmt::skip]
                    &[
                        ".. .. SS",
                        ".. .. uu",
                        "RR rr uu",
                    ],
                    false,
                ),
                (
                    #[rustfmt::skip]
                    &[
                        ".. .. SS",
                        ".. .. uu",
                        "RR rr rl",
                    ],
                    true,
                ),
            ];

            for (puzzle, expected) in examples {
                assert_eq!(
                    is_solved(&from_pictogram(puzzle)?),
                    *expected,
                    "Expected {:?} to be {:?}",
                    puzzle,
                    expected
                );
            }

            Ok(())
        }

        #[test]
        fn basic() -> Result {
            assert_eq!(
                solve(from_pictogram(&["RR .. SS"])?)?,
                vec![from_pictogram(&["RR rr SS"])?]
            );

            Ok(())
        }
    }
}
