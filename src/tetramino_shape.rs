use std::collections::HashSet;

use rand::{
    distr::{Distribution, StandardUniform},
    Rng,
};

use crate::{Block, ColorType, Position};

#[derive(Clone, Copy, Default)]
pub enum TetraminoKind {
    #[default]
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
#[derive(Default, Clone)]
pub struct Tetramino {
    kind: TetraminoKind,
    rotation_center: Position,
    rotation_state: RotationState,
    blocks: HashSet<Block>,
}
pub struct RotationResult {
    pub tetramino: Tetramino,
    pub kick_offsets: [Position; 5],
}

impl Tetramino {
    pub fn get_bounding_box(&self) -> BoundingBox {
        self.blocks
            .iter()
            .fold(Bounds::new(), |acc: Bounds, b: &Block| {
                acc.update((b.coordinates.row, b.coordinates.col))
            })
            .into()
    }
    pub fn get_center_delta(&self) -> isize {
        (self.get_bounding_box().width + 1) / 2
    }
    pub fn get_kind(&self) -> TetraminoKind {
        self.kind
    }
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

    pub fn get_blocks_with_offset(&self, offset: Position) -> HashSet<Block> {
        self.blocks
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates + offset,
            })
            .collect()
    }
    pub fn get_blocks(&self) -> &HashSet<Block> {
        &self.blocks
    }
    // values from SRS implementation by TTC: https://tetris.wiki/Super_Rotation_System#How_Guideline_SRS_Really_Works
    // (x, y) from site -> (-y, x) in code # because y-axis in my implementation is flipped
    fn get_offsets(&self, rotation_state: RotationState) -> [Position; 5] {
        match self.kind {
            TetraminoKind::I => match rotation_state {
                RotationState::Init => [
                    Position::new(0, 0),
                    Position::new(0, -1),
                    Position::new(0, 2),
                    Position::new(0, -1),
                    Position::new(0, 2),
                ],
                RotationState::Right => [
                    Position::new(0, -1),
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(-1, 0),
                    Position::new(2, 0),
                ],
                RotationState::Flip => [
                    Position::new(-1, -1),
                    Position::new(-1, 1),
                    Position::new(-1, -2),
                    Position::new(0, 1),
                    Position::new(0, -2),
                ],
                RotationState::Left => [
                    Position::new(-1, 0),
                    Position::new(-1, 0),
                    Position::new(-1, 0),
                    Position::new(1, 0),
                    Position::new(-2, 0),
                ],
            },
            TetraminoKind::O => match rotation_state {
                RotationState::Init => [Position::new(0, 0); 5],
                RotationState::Right => [Position::new(1, 0); 5],
                RotationState::Flip => [Position::new(1, -1); 5],
                RotationState::Left => [Position::new(0, -1); 5],
            },
            _ => match rotation_state {
                RotationState::Init => [
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                ],
                RotationState::Right => [
                    Position::new(0, 0),
                    Position::new(0, 1),
                    Position::new(1, 1),
                    Position::new(-2, 0),
                    Position::new(-2, 1),
                ],
                RotationState::Flip => [
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                    Position::new(0, 0),
                ],
                RotationState::Left => [
                    Position::new(0, 0),
                    Position::new(0, -1),
                    Position::new(1, -1),
                    Position::new(-2, 0),
                    Position::new(-2, -1),
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
                        Position::new(b.coordinates.col, -b.coordinates.row)
                    }
                    RotationDirection::CounterClockwise => {
                        Position::new(-b.coordinates.col, b.coordinates.row)
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

    pub fn get_rotated_and_offsets(&self, direction: RotationDirection) -> RotationResult {
        let rotated_shape = self.process_rotation(direction);

        let from_rotation = self.rotation_state;
        let to_rotation = self.get_next_rotation_state(direction);

        let from_offsets = self.get_offsets(from_rotation);
        let to_offsets = self.get_offsets(to_rotation);

        let mut res_offsets = [Position::default(); 5];
        for (i, (from, to)) in from_offsets.iter().zip(to_offsets).enumerate() {
            res_offsets[i] = *from - to;
        }

        RotationResult {
            tetramino: Tetramino {
                rotation_state: to_rotation,
                kind: self.kind,
                blocks: rotated_shape,
                rotation_center: self.rotation_center,
            },
            kick_offsets: res_offsets,
        }
    }
    pub fn construct(kind: TetraminoKind) -> Tetramino {
        match kind {
            TetraminoKind::I => Tetramino {
                blocks: {
                    [(0, 0), (0, 1), (0, 2), (0, 3)]
                        .into_iter()
                        .map(|(r, c)| -> Block {
                            Block {
                                color: ColorType::I,
                                coordinates: Position::new(r, c),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::I,
                rotation_center: Position::new(0, 1),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::L => Tetramino {
                blocks: {
                    [(0, 2), (1, 0), (1, 1), (1, 2)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::L,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::L,
                rotation_center: Position::new(1, 1),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::J => Tetramino {
                blocks: {
                    [(0, 0), (1, 0), (1, 1), (1, 2)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::J,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::J,
                rotation_center: Position::new(1, 1),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::S => Tetramino {
                blocks: {
                    [(0, 2), (0, 1), (1, 1), (1, 0)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::S,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::S,
                rotation_center: Position::new(1, 1),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::Z => Tetramino {
                blocks: {
                    [(0, 0), (0, 1), (1, 1), (1, 2)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::Z,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::Z,
                rotation_center: Position::new(1, 1),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::O => Tetramino {
                blocks: {
                    [(0, 0), (0, 1), (1, 0), (1, 1)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::O,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::O,
                rotation_center: Position::new(1, 0),
                rotation_state: RotationState::default(),
            },
            TetraminoKind::T => Tetramino {
                blocks: {
                    [(1, 0), (1, 1), (1, 2), (0, 1)]
                        .into_iter()
                        .map(|(row, col)| -> Block {
                            Block {
                                color: ColorType::T,
                                coordinates: Position::new(row, col),
                            }
                        })
                        .collect()
                },
                kind: TetraminoKind::T,
                rotation_center: Position::new(1, 1),
                rotation_state: RotationState::default(),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bounds {
    pub col_min: isize,
    pub col_max: isize,
    pub row_min: isize,
    pub row_max: isize,
}

impl Bounds {
    fn new() -> Bounds {
        Bounds {
            row_min: 0,
            row_max: 0,
            col_min: 0,
            col_max: 0,
        }
    }
    fn update(&self, (row, col): (isize, isize)) -> Bounds {
        Bounds {
            row_min: self.row_min.min(row),
            row_max: self.row_max.max(row),
            col_min: self.col_min.min(col),
            col_max: self.col_max.max(col),
        }
    }
}

pub struct BoundingBox {
    pub width: isize,
    pub height: isize,
}

impl From<Bounds> for BoundingBox {
    fn from(value: Bounds) -> Self {
        BoundingBox {
            width: value.col_max - value.col_min + 1,
            height: value.row_max - value.row_min + 1,
        }
    }
}
