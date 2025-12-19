use std::collections::HashSet;

use macroquad::color::*;
use rand::{
    distr::{Distribution, StandardUniform},
    Rng,
};

use crate::{Block, BoundingBox, Bounds, Coordinates};

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
    pub blocks: HashSet<Block>,
}

impl TetraminoShape {
    fn stick() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 2, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 3, col: 0 },
                        color: BLUE,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    fn L() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 0 },
                        color: ORANGE,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 0 },
                        color: ORANGE,
                    },
                    Block {
                        coordinates: Coordinates { row: 2, col: 0 },
                        color: ORANGE,
                    },
                    Block {
                        coordinates: Coordinates { row: 2, col: 1 },
                        color: ORANGE,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    fn J() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 1 },
                        color: DARKBLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 1 },
                        color: DARKBLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 2, col: 1 },
                        color: DARKBLUE,
                    },
                    Block {
                        coordinates: Coordinates { row: 2, col: 0 },
                        color: DARKBLUE,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    fn S() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 2 },
                        color: GREEN,
                    },
                    Block {
                        coordinates: Coordinates { row: 0, col: 1 },
                        color: GREEN,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 1 },
                        color: GREEN,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 0 },
                        color: GREEN,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    fn Z() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 0 },
                        color: RED,
                    },
                    Block {
                        coordinates: Coordinates { row: 0, col: 1 },
                        color: RED,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 1 },
                        color: RED,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 2 },
                        color: RED,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }
    fn T() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 0 },
                        color: PURPLE,
                    },
                    Block {
                        coordinates: Coordinates { row: 0, col: 1 },
                        color: PURPLE,
                    },
                    Block {
                        coordinates: Coordinates { row: 0, col: 2 },
                        color: PURPLE,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 1 },
                        color: PURPLE,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    fn O() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: Coordinates { row: 0, col: 0 },
                        color: YELLOW,
                    },
                    Block {
                        coordinates: Coordinates { row: 0, col: 1 },
                        color: YELLOW,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 0 },
                        color: YELLOW,
                    },
                    Block {
                        coordinates: Coordinates { row: 1, col: 1 },
                        color: YELLOW,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }

    pub fn construct(kind: TetraminoKind) -> TetraminoShape {
        match kind {
            TetraminoKind::I => Self::stick(),
            TetraminoKind::L => Self::L(),
            TetraminoKind::J => Self::J(),
            TetraminoKind::S => Self::S(),
            TetraminoKind::Z => Self::Z(),
            TetraminoKind::O => Self::O(),
            TetraminoKind::T => Self::T(),
        }
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.blocks
            .iter()
            .fold(Bounds::new(), |acc: Bounds, b: &Block| {
                acc.update((b.coordinates.row, b.coordinates.col))
            })
            .into()
    }
}
