use std::collections::HashSet;

use macroquad::color::*;
use rand::{
    distr::{Distribution, StandardUniform},
    Rng,
};

use crate::{Block, Coordinates};

#[derive(Clone, Copy)]
pub enum TetraminoKind {
    I,
    L,
    J,
    S,
    Z,
    O,
    T,
}

#[derive(Clone, Copy)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Default)]
enum RotationState {
    #[default]
    Init,
    Right,
    Flip,
    Left,
}

impl Distribution<TetraminoKind> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TetraminoKind {
        match rng.random_range(0usize..7) {
            0 => TetraminoKind::I,
            1 => TetraminoKind::L,
            2 => TetraminoKind::J,
            3 => TetraminoKind::T,
            4 => TetraminoKind::S,
            5 => TetraminoKind::Z,
            _ => TetraminoKind::O,
        }
    }
}
pub struct TetraminoShape {
    kind: TetraminoKind,
    rotation_center: Coordinates,
    rotation_state: RotationState,
    pub blocks: HashSet<Block>,
}

impl TetraminoShape {
    fn get_next_rotation_state(&self, direction: RotationDirection) -> RotationState {
        match direction {
            RotationDirection::Clockwise => match self.rotation_state {
                RotationState::Init => RotationState::Right,
                RotationState::Right => RotationState::Flip,
                RotationState::Flip => RotationState::Left,
                RotationState::Left => RotationState::Init,
            },
            RotationDirection::CounterClockwise => match self.rotation_state {
                RotationState::Init => RotationState::Left,
                RotationState::Right => RotationState::Init,
                RotationState::Flip => RotationState::Right,
                RotationState::Left => RotationState::Flip,
            },
        }
    }

    pub fn with_offset(&self, offset: Coordinates) -> HashSet<Block> {
        self.blocks
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates + offset,
            })
            .collect()
    }

    // values from SRS implementation by TTC: https://tetris.wiki/Super_Rotation_System#How_Guideline_SRS_Really_Works
    // (x, y) from site -> (-y, x) in code # because y-axis in my implementation is flipped
    fn get_offsets(&self, rotation_state: RotationState) -> [Coordinates; 5] {
        match self.kind {
            TetraminoKind::I => match rotation_state {
                RotationState::Init => [
                    Coordinates::new(0, 0),
                    Coordinates::new(0, -1),
                    Coordinates::new(0, 2),
                    Coordinates::new(0, -1),
                    Coordinates::new(0, 2),
                ],
                RotationState::Right => [
                    Coordinates::new(0, -1),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(-1, 0),
                    Coordinates::new(2, 0),
                ],
                RotationState::Flip => [
                    Coordinates::new(-1, -1),
                    Coordinates::new(-1, 1),
                    Coordinates::new(-1, -2),
                    Coordinates::new(0, 1),
                    Coordinates::new(0, -2),
                ],
                RotationState::Left => [
                    Coordinates::new(-1, 0),
                    Coordinates::new(-1, 0),
                    Coordinates::new(-1, 0),
                    Coordinates::new(1, 0),
                    Coordinates::new(-2, 0),
                ],
            },
            TetraminoKind::O => match rotation_state {
                RotationState::Init => [Coordinates::new(0, 0); 5],
                RotationState::Right => [Coordinates::new(1, 0); 5],
                RotationState::Flip => [Coordinates::new(1, -1); 5],
                RotationState::Left => [Coordinates::new(0, -1); 5],
            },
            _ => match rotation_state {
                RotationState::Init => [
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                ],
                RotationState::Right => [
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 1),
                    Coordinates::new(1, 1),
                    Coordinates::new(-2, 0),
                    Coordinates::new(-2, 1),
                ],
                RotationState::Flip => [
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                    Coordinates::new(0, 0),
                ],
                RotationState::Left => [
                    Coordinates::new(0, 0),
                    Coordinates::new(0, -1),
                    Coordinates::new(1, -1),
                    Coordinates::new(-2, 0),
                    Coordinates::new(-2, -1),
                ],
            },
        }
    }

    fn process_rotation(&self, direction: RotationDirection) -> HashSet<Block> {
        let rotation_origin_coords: HashSet<Block> = self
            .blocks
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates - self.rotation_center,
            })
            .collect();

        let rotated: HashSet<Block> = rotation_origin_coords
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: match direction {
                    RotationDirection::Clockwise => {
                        Coordinates::new(b.coordinates.col, -b.coordinates.row)
                    }
                    RotationDirection::CounterClockwise => {
                        Coordinates::new(-b.coordinates.col, b.coordinates.row)
                    }
                },
            })
            .collect();
        let game_coords: HashSet<Block> = rotated
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates + self.rotation_center,
            })
            .collect();
        game_coords
    }
    pub fn get_rotated_and_offsets(
        &self,
        direction: RotationDirection,
    ) -> (TetraminoShape, [Coordinates; 5]) {
        let rotated_shape = self.process_rotation(direction);

        let from_rotation = self.rotation_state;
        let to_rotation = self.get_next_rotation_state(direction);

        let from_offsets = self.get_offsets(from_rotation);
        let to_offsets = self.get_offsets(to_rotation);

        let mut res_offsets = [Coordinates::default(); 5];
        for (i, (from, to)) in from_offsets.iter().zip(to_offsets).enumerate() {
            res_offsets[i] = *from - to;
        }

        (
            TetraminoShape {
                rotation_state: to_rotation,
                kind: self.kind,
                blocks: rotated_shape,
                rotation_center: self.rotation_center,
            },
            res_offsets,
        )
    }
    pub fn construct(kind: TetraminoKind) -> TetraminoShape {
        match kind {
            TetraminoKind::I => TetraminoShape {
                blocks: {
                    [(0, 0), (0, 1), (0, 2), (0, 3)]
                        .iter()
                        .map(|(r, c)| -> Block {
                            Block {
                                color: BLUE,
                                coordinates: Coordinates::new(*r, *c),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::I,
                rotation_center: Coordinates::new(0, 1),
                rotation_state: Default::default(),
            },
            TetraminoKind::L => TetraminoShape {
                blocks: {
                    [(0, 2), (1, 0), (1, 1), (1, 2)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ORANGE,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::L,
                rotation_center: Coordinates::new(1, 1),
                rotation_state: Default::default(),
            },
            TetraminoKind::J => TetraminoShape {
                blocks: {
                    [(0, 0), (1, 0), (1, 1), (1, 2)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: DARKBLUE,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::J,
                rotation_center: Coordinates::new(1, 1),
                rotation_state: Default::default(),
            },
            TetraminoKind::S => TetraminoShape {
                blocks: {
                    [(0, 2), (0, 1), (1, 1), (1, 0)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: GREEN,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::S,
                rotation_center: Coordinates::new(1, 1),
                rotation_state: Default::default(),
            },
            TetraminoKind::Z => TetraminoShape {
                blocks: {
                    [(0, 0), (0, 1), (1, 1), (1, 2)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: RED,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::Z,
                rotation_center: Coordinates::new(1, 1),
                rotation_state: Default::default(),
            },
            TetraminoKind::O => TetraminoShape {
                blocks: {
                    [(0, 0), (0, 1), (1, 0), (1, 1)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: YELLOW,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::O,
                rotation_center: Coordinates::new(1, 0),
                rotation_state: Default::default(),
            },
            TetraminoKind::T => TetraminoShape {
                blocks: {
                    [(1, 0), (1, 1), (1, 2), (0, 1)]
                        .iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: PURPLE,
                                coordinates: Coordinates::new(*row, *col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::T,
                rotation_center: Coordinates::new(1, 1),
                rotation_state: Default::default(),
            },
        }
    }
}
