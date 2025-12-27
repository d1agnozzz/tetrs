use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, RemAssign, Sub},
    time::Instant,
};

use macroquad::{
    color::{Color, RED},
    input::KeyCode,
};
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::tetramino_shape::{RotationDirection, RotationResult, Tetramino, TetraminoKind};

mod tetramino_shape;
#[derive(Debug)]
pub struct InputEvent {
    pub keys: HashSet<KeyCode>,
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub color: Color,
    pub coordinates: Position,
}

impl From<Position> for Block {
    fn from(value: Position) -> Self {
        Block {
            color: RED,
            coordinates: value,
        }
    }
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

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: isize,
    pub col: isize,
}

impl Position {
    pub fn new(row: isize, col: isize) -> Position {
        Position { row, col }
    }
    fn is_inbound(&self, rows: isize, cols: isize) -> bool {
        self.row < rows && self.row >= 0 && self.col < cols && self.col >= 0
    }
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.row, &mut self.col);
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.col += rhs.col;
        self.row += rhs.row;
    }
}

impl RemAssign<PlayfieldSize> for Position {
    fn rem_assign(&mut self, rhs: PlayfieldSize) {
        self.col %= rhs.cols;
        self.row %= rhs.rows;
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Position {
            row: self.row - rhs.row,
            col: self.col - rhs.col,
        }
    }
}

pub struct ActiveTetramino {
    shape: Tetramino,
    offset: Position,
}

impl ActiveTetramino {
    fn new(shape: Tetramino) -> ActiveTetramino {
        ActiveTetramino {
            shape,
            offset: Position::default(),
        }
    }

    fn with_offset(self, offset: Position) -> ActiveTetramino {
        Self {
            shape: self.shape,
            offset,
        }
    }

    fn translate_with_offset(&mut self, offset: Position) {
        self.offset += offset;
    }

    fn get_rotation_result(&self, direction: RotationDirection) -> RotationResult {
        self.shape.get_rotated_and_offsets(direction)
    }

    pub fn get_blocks_with_offset(&self) -> HashSet<Block> {
        self.shape
            .get_blocks()
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates + self.offset,
            })
            .collect()
    }
}

struct Playfield {
    size: PlayfieldSize,
    placed_blocks: PlacedBlocks,
}

impl Playfield {
    pub fn new(size: PlayfieldSize) -> Playfield {
        Playfield {
            size,
            placed_blocks: PlacedBlocks::default(),
        }
    }
    pub fn put_blocks(&mut self, blocks: &HashSet<Block>) {
        self.placed_blocks.put_blocks(blocks);
    }
    fn check_intersections(&self, blocks: &HashSet<Block>) -> bool {
        let stationary_blocks = self.placed_blocks.get_blocks();
        for block in blocks {
            if stationary_blocks.contains(&block)
                || !block.coordinates.is_inbound(self.size.rows, self.size.cols)
            {
                return true;
            }
        }
        false
    }

    pub fn check_collisions(&self, subject: &HashSet<Block>) -> CollisionResult {
        let stationary_blocks = self.placed_blocks.get_blocks();

        let mut collision_result = CollisionResult::new();

        for block in subject {
            for direction in CollisionDirection::iter() {
                let neighbour_coords = block.coordinates + direction.offset();
                if stationary_blocks.contains(&neighbour_coords.into())
                    || !neighbour_coords.is_inbound(self.size.rows, self.size.cols)
                {
                    match direction {
                        CollisionDirection::Down => collision_result.down = true,
                        CollisionDirection::Left => collision_result.left = true,
                        CollisionDirection::Right => collision_result.right = true,
                    }
                    dbg!(subject);
                    dbg!(stationary_blocks);
                    dbg!(block);
                    dbg!(direction);
                    dbg!(neighbour_coords);
                }
            }
        }
        collision_result
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
    // merge into Playfield
    playfield: Playfield,
    // merge into TetraminoManager?
    // pub current_tetramino: ActiveTetramino,
    // pub next_tetramino: TetraminoKind,
    tetramino_manager: TetraminoManager,
    // merge into TimerManager
    pub descend_delay_timer: TimerMs,
    pub place_delay_ms: usize,

    // move to TetraminoManager
    collision_state: CollisionState,
}

#[derive(EnumIter, Debug, PartialEq)]
enum CollisionDirection {
    Down,
    Left,
    Right,
}

impl CollisionDirection {
    pub fn offset(&self) -> Position {
        match self {
            CollisionDirection::Down => Position::new(1, 0),
            CollisionDirection::Left => Position::new(0, -1),
            CollisionDirection::Right => Position::new(0, 1),
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
    Delaying,
}

struct PlacementDelayManager {
    collision_state: CollisionState,
    delay_ms: usize,
    timer: TimerMs,
}

impl PlacementDelayManager {
    fn new(delay_ms: usize) -> PlacementDelayManager {
        PlacementDelayManager {
            collision_state: CollisionState::Idle,
            delay_ms,
            timer: TimerMs::new(0),
        }
    }
    fn delay_passed(&mut self, is_colliding: bool) -> bool {
        match self.collision_state {
            CollisionState::Idle => {
                if is_colliding {
                    self.collision_state = CollisionState::Delaying;
                    self.timer = TimerMs::new(self.delay_ms);
                }
                false
            }
            CollisionState::Delaying => {
                if self.timer.update() {
                    self.collision_state = CollisionState::Idle;
                    true
                } else {
                    false
                }
            }
        }
    }
}

struct TetraminoManager {
    active: ActiveTetramino,
    gravity_delay: TimerMs,
    placement_delay: PlacementDelayManager,
    next: TetraminoKind,
    hold: Option<Tetramino>,
}

impl TetraminoManager {
    pub fn new(gravity_delay_ms: usize, placement_delay_ms: usize) -> TetraminoManager {
        TetraminoManager {
            active: ActiveTetramino::new(Tetramino::construct(rand::random())),
            gravity_delay: TimerMs::new(gravity_delay_ms),
            placement_delay: PlacementDelayManager::new(placement_delay_ms),
            next: rand::random(),
            hold: None,
        }
    }
    pub fn propogate_gravity(&mut self) {
        self.active
            .translate_with_offset(Position { row: 1, col: 0 });
    }
    pub fn with_offset(self, offset: Position) -> TetraminoManager {
        TetraminoManager {
            active: self.active.with_offset(offset),
            gravity_delay: self.gravity_delay,
            placement_delay: self.placement_delay,
            next: self.next,
            hold: self.hold,
        }
    }
    pub fn next_tetramino(&mut self) {
        self.active = ActiveTetramino::new(Tetramino::construct(self.next));
        self.next = rand::random();
    }
    pub fn rotate(&self, direction: RotationDirection) -> RotationResult {
        self.active.get_rotation_result(direction)
    }
}

impl GameState {
    pub fn new(
        playfield_size: PlayfieldSize,
        gravity_delay_ms: usize,
        placement_delay_ms: usize,
    ) -> GameState {
        GameState {
            playfield: Playfield::new(playfield_size),
            descend_delay_timer: TimerMs::new(200),
            place_delay_ms: 1000,
            collision_state: CollisionState::Idle,
            tetramino_manager: TetraminoManager::new(gravity_delay_ms, placement_delay_ms)
                .with_offset(Position::new(
                    playfield_size.rows / 2,
                    playfield_size.cols / 2,
                )),
        }
    }

    pub fn try_rotate(&mut self, direction: RotationDirection) {
        let rotation_result = self.tetramino_manager.rotate(direction);

        for kick_offset in rotation_result.kick_offsets {
            if !self.playfield.check_intersections(
                &rotation_result
                    .tetramino
                    .get_blocks_with_offset(self.tetramino_manager.active.offset + kick_offset),
            ) {
                self.tetramino_manager.active.shape = rotation_result.tetramino;
                self.tetramino_manager.active.offset += kick_offset;
                break;
            }
        }
    }
    pub fn update(&mut self) {}
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
            wait_ms,
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
        game_state.translate_cur_tetramino(Position { row: 0, col: -1 });
    }
    if input.keys.contains(&KeyCode::D) && !collision.right {
        game_state.translate_cur_tetramino(Position { row: 0, col: 1 });
    }
    if input.keys.contains(&KeyCode::E) {
        game_state.try_rotate(RotationDirection::Clockwise);
    }
    if input.keys.contains(&KeyCode::Q) {
        game_state.try_rotate(RotationDirection::CounterClockwise);
    }
    if input.keys.contains(&KeyCode::N) {
        game_state.next_turn();
    }

    if !collision.down && game_state.descend_delay_timer.update() {
        game_state.translate_cur_tetramino(Position::new(1, 0));
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
                CollisionState::Done
            } else {
                CollisionState::Delaying { timer }
            }
        }
        CollisionState::Done => CollisionState::Idle,
    };
}
