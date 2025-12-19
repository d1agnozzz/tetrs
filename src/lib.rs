use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, BitAnd, RemAssign, Sub, SubAssign},
    time::Instant,
    u8, usize,
};

use macroquad::{
    color::{Color, BLUE, DARKBLUE, GREEN, ORANGE, RED},
    input::KeyCode,
    miniquad::{native::linux_x11::libx11::Time, TextureKind},
};
use rand::{
    distr::{Distribution, StandardUniform},
    Rng,
};
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::tetramino_shape::{TetraminoKind, TetraminoShape};

mod tetramino_shape;
#[derive(Debug)]
pub struct InputEvent {
    pub keys: HashSet<KeyCode>,
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub color: Color,
    pub coordinates: Coordinates,
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
    width: isize,
    height: isize,
}

impl From<Bounds> for BoundingBox {
    fn from(value: Bounds) -> Self {
        BoundingBox {
            width: value.col_max - value.col_min,
            height: value.row_max - value.row_min,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Coordinates {
    pub row: isize,
    pub col: isize,
}

impl Coordinates {
    fn new(row: isize, col: isize) -> Coordinates {
        Coordinates { row: row, col: col }
    }
    fn clamp_inplace(&mut self, min_row: isize, max_row: isize, min_col: isize, max_col: isize) {
        self.row = self.row.clamp(min_row, max_row);
        self.col = self.col.clamp(min_col, max_col);
    }
    fn is_inbound(&self, rows: isize, cols: isize) -> bool {
        self.row < rows && self.row >= 0 && self.col < cols && self.col >= 0
    }
}

impl Add<Coordinates> for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Coordinates) -> Self::Output {
        Coordinates {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl AddAssign for Coordinates {
    fn add_assign(&mut self, rhs: Self) {
        self.col += rhs.col;
        self.row += rhs.row;
    }
}

impl RemAssign<PlayfieldSize> for Coordinates {
    fn rem_assign(&mut self, rhs: PlayfieldSize) {
        self.col %= rhs.cols;
        self.row %= rhs.rows;
    }
}

impl SubAssign<BoundingBox> for Coordinates {
    fn sub_assign(&mut self, rhs: BoundingBox) {
        self.col -= rhs.height;
        self.row -= rhs.width;
    }
}

pub struct MovingTetramino {
    pub shape: TetraminoShape,
    pub offset: Coordinates,
}

impl MovingTetramino {
    fn new(shape: TetraminoShape) -> MovingTetramino {
        MovingTetramino {
            shape: shape,
            offset: Coordinates::default(),
        }
    }

    fn get_blocks_with_offset(&self) -> HashSet<Block> {
        self.shape
            .blocks
            .iter()
            .map(|b| {
                let offset_row = b.coordinates.row + self.offset.row;
                let offset_col = b.coordinates.col + self.offset.col;
                Block {
                    color: b.color,
                    coordinates: Coordinates {
                        row: offset_row,
                        col: offset_col,
                    },
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct PlayfieldSize {
    pub rows: isize,
    pub cols: isize,
}

#[derive(Default)]
pub struct PlacedBlocks {
    storage: HashSet<Block>,
}

impl PlacedBlocks {
    pub fn get_blocks(&self) -> &HashSet<Block> {
        &self.storage
    }
}

impl PlacedBlocks {
    fn put_blocks(&mut self, blocks: &HashSet<Block>) {
        self.storage.extend(blocks.iter());
    }
}

pub struct GameState {
    pub playfield_size: PlayfieldSize,
    pub placed_blocks: PlacedBlocks,
    pub current_tetramino: MovingTetramino,
    pub next_tetramino: TetraminoKind,
    pub descend_delay_timer: TimerMs,
    pub place_delay_ms: usize,
    pub collision_state: CollisionState,
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

enum CollisionType {
    Terminal,
    NonTerminal,
}
#[derive(EnumIter, Debug, PartialEq)]
enum CollisionDirection {
    Down,
    Left,
    Right,
}

impl CollisionDirection {
    pub fn offset(&self) -> Coordinates {
        match self {
            CollisionDirection::Down => Coordinates::new(1, 0),
            CollisionDirection::Left => Coordinates::new(0, -1),
            CollisionDirection::Right => Coordinates::new(0, 1),
        }
    }
}
#[derive(Debug)]
struct CollisionResult {
    down: bool,
    left: bool,
    right: bool,
}

impl CollisionResult {
    pub fn new() -> CollisionResult {
        CollisionResult {
            down: false,
            left: false,
            right: false,
        }
    }
}
enum CollisionState {
    Idle,
    Delaying { timer: TimerMs },
    Reacting,
}

impl CollisionState {
    fn new() -> Self {
        CollisionState::Idle
    }
}

impl GameState {
    pub fn get_playfield_top_center(playfield_size: PlayfieldSize) -> Coordinates {
        Coordinates {
            col: playfield_size.cols / 2,
            row: 0,
        }
    }
    pub fn new(playfield_size: PlayfieldSize) -> GameState {
        let mut first_tetramino = MovingTetramino::new(TetraminoShape::construct(rand::random()));
        first_tetramino.offset = GameState::get_playfield_top_center(playfield_size);

        GameState {
            playfield_size: playfield_size,
            placed_blocks: Default::default(),
            descend_delay_timer: TimerMs::new(200),
            place_delay_ms: 1000,
            collision_state: CollisionState::Idle,
            current_tetramino: first_tetramino,
            next_tetramino: rand::random(),
            column_toggling: 5,
            row_toggleing: 0,
        }
    }
    pub fn reset_tetramino_offset(&mut self) {
        self.current_tetramino.offset = Coordinates {
            col: self.playfield_size.cols / 2,
            row: self.playfield_size.rows / 2,
        }
    }
    pub fn next_turn(&mut self) {
        let mut next_tetramino =
            MovingTetramino::new(TetraminoShape::construct(self.next_tetramino));
        next_tetramino.offset = GameState::get_playfield_top_center(self.playfield_size);
        self.current_tetramino = next_tetramino;
        self.next_tetramino = rand::random();
    }
    pub fn check_collision(&mut self) -> CollisionResult {
        let moving_blocks = &self.current_tetramino.get_blocks_with_offset();
        let stationary_blocks = self.placed_blocks.get_blocks();
        let mut collision_result = CollisionResult::new();

        for block in moving_blocks {
            for direction in CollisionDirection::iter() {
                let neighbour_coords = block.coordinates + direction.offset();
                if stationary_blocks
                    .iter()
                    .any(|b| b.coordinates == neighbour_coords)
                    || !neighbour_coords
                        .is_inbound(self.playfield_size.rows, self.playfield_size.cols)
                {
                    match direction {
                        CollisionDirection::Down => collision_result.down = true,
                        CollisionDirection::Left => collision_result.left = true,
                        CollisionDirection::Right => collision_result.right = true,
                    }
                    dbg!(moving_blocks);
                    dbg!(stationary_blocks);
                    dbg!(block);
                    dbg!(direction);
                    dbg!(neighbour_coords);
                }
            }
        }
        return collision_result;
    }

    pub fn place_current_tetramino(&mut self) {
        self.placed_blocks
            .put_blocks(&self.current_tetramino.get_blocks_with_offset());
    }

    pub fn translate_cur_tetramino(&mut self, offset: Coordinates) {
        self.current_tetramino.offset += offset;

        let bounding_box = self.current_tetramino.shape.get_bounding_box();
        let max_row = self.playfield_size.rows - bounding_box.height - 1;
        let max_col = self.playfield_size.cols - bounding_box.width - 1;

        self.current_tetramino
            .offset
            .clamp_inplace(0, max_row, 0, max_col);
    }
}

#[derive(Clone, Copy)]
pub struct TimerMs {
    deadline: Instant,
    wait_ms: usize,
}

impl TimerMs {
    pub fn new(wait_ms: usize) -> Self {
        Self {
            deadline: Instant::now() + Duration::from_millis(wait_ms as u64),
            wait_ms: wait_ms,
        }
    }
    pub fn reset(&self) -> Self {
        Self {
            deadline: Instant::now() + Duration::from_millis(self.wait_ms as u64),
            wait_ms: self.wait_ms,
        }
    }
    pub fn update(&mut self) -> bool {
        if self.deadline <= std::time::Instant::now() {
            *self = Self::new(self.wait_ms);
            true
        } else {
            false
        }
    }
}

pub fn process_logic(game_state: &mut GameState, input: InputEvent) {
    let collision = game_state.check_collision();
    if input.keys.contains(&KeyCode::A) && !collision.left {
        game_state.translate_cur_tetramino(Coordinates { row: 0, col: -1 });
    }
    if input.keys.contains(&KeyCode::D) && !collision.right {
        game_state.translate_cur_tetramino(Coordinates { row: 0, col: 1 });
    }

    if !collision.down && game_state.descend_delay_timer.update() {
        game_state.translate_cur_tetramino(Coordinates::new(1, 0));
        game_state.collision_state = CollisionState::Idle;
    }

    game_state.collision_state = match game_state.collision_state {
        CollisionState::Idle => {
            if collision.down {
                CollisionState::Delaying {
                    timer: TimerMs::new(game_state.place_delay_ms),
                }
            } else {
                CollisionState::Idle
            }
        }
        CollisionState::Delaying { mut timer } => {
            if timer.update() {
                game_state.place_current_tetramino();
                game_state.next_turn();
                CollisionState::Reacting
            } else {
                CollisionState::Delaying { timer: timer }
            }
        }
        CollisionState::Reacting => CollisionState::Idle,
    };
}
