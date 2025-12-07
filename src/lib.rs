use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, Bound, Mul, Rem, RemAssign, Sub, SubAssign},
    process::Output,
};

use macroquad::{
    color::{Color, BLUE, GREEN, RED},
    input::KeyCode,
};
use std::time::Duration;

#[derive(Debug)]
pub struct InputEvent {
    pub keys: HashSet<KeyCode>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct BlockCoordinates {
    pub row: isize,
    pub col: isize,
}

impl Add<[isize; 2]> for BlockCoordinates {
    type Output = BlockCoordinates;

    fn add(self, rhs: [isize; 2]) -> Self::Output {
        BlockCoordinates {
            row: self.row + rhs[0],
            col: self.col + rhs[1],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Block {
    pub color: Color,
    pub coordinates: BlockCoordinates,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.coordinates == other.coordinates
    }
}

impl Eq for Block {}

impl Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.coordinates.hash(state);
    }
}

const PLAYFIELD_ROWS: isize = 20;
const PLAYFIELD_COLS: isize = 10;

pub struct TetraminoShape {
    pub blocks: HashSet<Block>,
}

#[derive(Clone, Copy)]
pub struct Bounds {
    pub x_min: isize,
    pub x_max: isize,
    pub y_min: isize,
    pub y_max: isize,
}

impl Bounds {
    fn new() -> Bounds {
        Bounds {
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
        }
    }
    fn update(&self, (x, y): (isize, isize)) -> Bounds {
        Bounds {
            x_min: self.x_min.min(x),
            x_max: self.x_max.max(x),
            y_min: self.y_min.min(y),
            y_max: self.y_max.max(y),
        }
    }
}

pub struct BoundingBox {
    width: isize,
    height: isize,
}

impl From<Bounds> for BoundingBox {
    fn from(value: Bounds) -> Self {
        BoundingBox {
            width: value.x_max - value.x_min,
            height: value.y_max - value.y_min,
        }
    }
}

impl TetraminoShape {
    fn stick() -> TetraminoShape {
        TetraminoShape {
            blocks: {
                vec![
                    Block {
                        coordinates: BlockCoordinates { row: 0, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: BlockCoordinates { row: 1, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: BlockCoordinates { row: 2, col: 0 },
                        color: BLUE,
                    },
                    Block {
                        coordinates: BlockCoordinates { row: 3, col: 0 },
                        color: BLUE,
                    },
                ]
                .into_iter()
                .collect()
            },
        }
    }
    fn get_bounding_box(&self) -> BoundingBox {
        self.blocks
            .iter()
            .fold(Bounds::new(), |acc: Bounds, b: &Block| {
                acc.update((b.coordinates.col, b.coordinates.row))
            })
            .into()
    }
}

#[derive(Default, Clone, Copy)]
pub struct OffsetPosition {
    pub x_offset: isize,
    pub y_offset: isize,
}

impl AddAssign for OffsetPosition {
    fn add_assign(&mut self, rhs: Self) {
        self.x_offset += rhs.x_offset;
        self.y_offset += rhs.y_offset;
    }
}

impl RemAssign<PlayfieldSize> for OffsetPosition {
    fn rem_assign(&mut self, rhs: PlayfieldSize) {
        self.x_offset %= rhs.cols;
        self.y_offset %= rhs.rows;
    }
}

impl SubAssign<BoundingBox> for OffsetPosition {
    fn sub_assign(&mut self, rhs: BoundingBox) {
        self.x_offset -= rhs.height;
        self.y_offset -= rhs.width;
    }
}

pub struct FallingTetramino {
    pub shape: TetraminoShape,
    pub offset: OffsetPosition,
}

impl FallingTetramino {
    fn new(shape: TetraminoShape) -> FallingTetramino {
        FallingTetramino {
            shape: shape,
            offset: OffsetPosition::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct PlayfieldSize {
    pub rows: isize,
    pub cols: isize,
}

pub struct PlacedBlocks {
    storage: HashSet<Block>,
}

pub struct GameState {
    pub playfield_size: PlayfieldSize,
    pub placed_blocks: HashSet<Block>,
    pub current_tetramino: FallingTetramino,
    pub next_tetramino: TetraminoShape,
    pub descend_delay: TimerMs,
    pub column_toggling: isize,
    pub row_toggleing: isize,
}

impl Sub<BoundingBox> for PlayfieldSize {
    type Output = PlayfieldSize;

    fn sub(self, rhs: BoundingBox) -> Self::Output {
        PlayfieldSize {
            rows: self.rows - rhs.height,
            cols: self.cols - rhs.width,
        }
    }
}

impl GameState {
    pub fn get_playfield_center(&self) -> OffsetPosition {
        OffsetPosition {
            x_offset: self.playfield_size.cols / 2,
            y_offset: self.playfield_size.rows / 2,
        }
    }
    pub fn new() -> GameState {
        GameState {
            playfield_size: PlayfieldSize { rows: 20, cols: 10 },
            placed_blocks: Default::default(),
            descend_delay: TimerMs::new(1000),
            current_tetramino: FallingTetramino::new(TetraminoShape::stick()),
            next_tetramino: TetraminoShape::stick(),
            column_toggling: 5,
            row_toggleing: 0,
        }
    }
    pub fn reset_tetramino_offset(&mut self) {
        self.current_tetramino.offset = OffsetPosition {
            x_offset: self.playfield_size.cols / 2,
            y_offset: self.playfield_size.rows / 2,
        }
    }
    pub fn check_collision(&mut self) -> bool {
        // TODO: надо переключать ход на следующую фигуру только когда колиззия происходит из-за
        // гравитации, а не просто из-за прикосновения фигуры к чему-либо во время свободного
        // падения
        let mut moving_blocks = &self.current_tetramino.shape.blocks;
        let mut stationary_blocks = &self.placed_blocks;
        let arrangements = [[-1, 0], [1, 0], [0, 1], [0, -1]];

        for block in moving_blocks {
            for arrangement in arrangements {
                let neighbour_coords = block.coordinates + arrangement;
                if stationary_blocks.contains(&Block {
                    color: RED,
                    coordinates: neighbour_coords,
                }) || neighbour_coords.col >= 0
                    && neighbour_coords.row >= 0
                    && neighbour_coords.col < self.playfield_size.cols
                    && neighbour_coords.row < self.playfield_size.rows
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn place_current_tetramino(&mut self) -> bool {
        self.placed_blocks
            .extend(self.current_tetramino.shape.blocks.iter());
        todo!("check if game over")
    }

    pub fn push_cur_tetramino(&mut self) {
        self.current_tetramino.offset += OffsetPosition {
            x_offset: 1,
            y_offset: 1,
        };
        self.current_tetramino.offset %=
            self.playfield_size - self.current_tetramino.shape.get_bounding_box();
    }
}

#[derive(Clone, Copy)]
pub struct TimerMs {
    time_start: std::time::Instant,
    elapsed: i64,
    wait: i64,
}

impl TimerMs {
    pub fn new(wait: i64) -> Self {
        Self {
            time_start: std::time::Instant::now(),
            elapsed: 0,
            wait: wait,
        }
    }
    pub fn update(&mut self) -> bool {
        self.elapsed += self.time_start.elapsed().as_millis() as i64;
        self.is_out()
    }
    pub fn is_out(&self) -> bool {
        self.wait <= (self.elapsed)
    }
    pub fn reset(&mut self) {
        while self.is_out() {
            self.elapsed -= self.wait;
        }
        self.time_start = std::time::Instant::now();
        // self.elapsed = 0;
    }
    pub fn elapsed(&self) -> i64 {
        self.elapsed
    }
    pub fn time_left(&self) -> i64 {
        self.wait - self.elapsed
    }
}

pub fn process_logic(game_state: &mut GameState, input: InputEvent) {
    if game_state.check_collision() {
        game_state.place_current_tetramino();
    } else {
        game_state.push_cur_tetramino();
    }
    std::thread::sleep(Duration::from_millis(100));
}
