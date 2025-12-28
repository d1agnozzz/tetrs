use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, AddAssign, RemAssign, Sub},
    time::Instant,
};

use itertools::Itertools;
use macroquad::color::Color;
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::tetramino_shape::{RotationDirection, RotationResult, Tetramino, TetraminoKind};

#[derive(Clone, Copy)]
pub enum PlayerIntention {
    None,
    MoveLeft,
    MoveRight,
    RotateClockwise,
    RotateCounterClockwise,
    DropSoft,
    DropHard,
    HoldPiece,
}

#[derive(Clone, Copy, Debug)]
pub enum ColorType {
    I,
    O,
    L,
    J,
    S,
    Z,
    T,
}

impl From<ColorType> for Color {
    fn from(value: ColorType) -> Self {
        match value {
            ColorType::I => Color::from_hex(0x00e6fe),
            ColorType::O => Color::from_hex(0xffde00),
            ColorType::L => Color::from_hex(0xff7308),
            ColorType::J => Color::from_hex(0x1801ff),
            ColorType::S => Color::from_hex(0x66fd00),
            ColorType::Z => Color::from_hex(0xfe103c),
            ColorType::T => Color::from_hex(0xb802fd),
        }
    }
}

mod tetramino_shape;

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub color: ColorType,
    pub coordinates: Position,
}

impl From<Position> for Block {
    fn from(value: Position) -> Self {
        Block {
            color: ColorType::I,
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
    fn spawn(tetramino: Tetramino, center: Position) -> ActiveTetramino {
        ActiveTetramino {
            offset: Position::new(center.row, center.col - tetramino.get_center_delta()),
            shape: tetramino,
        }
    }
    pub fn get_kind(&self) -> TetraminoKind {
        self.shape.get_kind()
    }

    fn translate_with_offset(&mut self, offset: Position) {
        self.offset += offset;
    }

    fn get_rotation_result(&self, direction: RotationDirection) -> RotationResult {
        self.shape.get_rotated_and_offsets(direction)
    }

    pub fn get_blocks_with_offset(&self, additional: Option<Position>) -> HashSet<Block> {
        self.shape
            .get_blocks()
            .iter()
            .map(|b| Block {
                color: b.color,
                coordinates: b.coordinates + self.offset + additional.unwrap_or_default(),
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
    pub fn line_completion(&mut self, subject: &HashSet<Block>) {
        let completed_lines = self
            .placed_blocks
            .filled_lines(subject, self.size.cols as usize);
        dbg!(&completed_lines);
        self.placed_blocks.remove_lines(completed_lines);
    }
    pub fn put_blocks(&mut self, blocks: &HashSet<Block>) {
        self.placed_blocks.put_blocks(blocks);
    }
    fn check_intersections(&self, blocks: &HashSet<Block>) -> bool {
        let stationary_blocks = self.placed_blocks.get_blocks();
        for block in blocks {
            if stationary_blocks.contains(block)
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

impl PlayfieldSize {
    pub fn new(rows: isize, cols: isize) -> PlayfieldSize {
        PlayfieldSize { rows, cols }
    }
}

#[derive(Default)]
pub struct PlacedBlocks {
    storage: HashSet<Block>,
}

impl PlacedBlocks {
    pub fn get_blocks(&self) -> &HashSet<Block> {
        &self.storage
    }
    fn filled_lines(&self, subject: &HashSet<Block>, line_width: usize) -> Vec<isize> {
        let rows = subject.iter().map(|b| b.coordinates.row).unique();

        let mut marked_rows = Vec::with_capacity(4);
        for row in rows {
            let blocks_in_row: Vec<&Block> = self
                .storage
                .iter()
                .filter(|b| b.coordinates.row == row)
                .collect();
            if blocks_in_row.len() == line_width {
                marked_rows.push(row);
            }
        }
        marked_rows
    }
    fn remove_lines(&mut self, mut lines: Vec<isize>) {
        lines.sort_unstable();
        for line in lines {
            self.storage = self
                .storage
                .iter()
                .filter(|b| b.coordinates.row != line)
                .map(|b| {
                    if b.coordinates.row < line {
                        Block {
                            color: b.color,
                            coordinates: Position::new(b.coordinates.row + 1, b.coordinates.col),
                        }
                    } else {
                        *b
                    }
                })
                .collect();
        }
    }
}

impl PlacedBlocks {
    fn put_blocks(&mut self, blocks: &HashSet<Block>) {
        self.storage.extend(blocks.iter());
    }
}

pub struct GameState {
    playfield: Playfield,
    tetramino_manager: TetraminoManager,
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
enum LockState {
    Idle,
    Delaying,
}

struct PlacementDelayManager {
    lock_state: LockState,
    delay_ms: usize,
    timer: TimerMs,
}

impl PlacementDelayManager {
    fn new(delay_ms: usize) -> PlacementDelayManager {
        PlacementDelayManager {
            lock_state: LockState::Idle,
            delay_ms,
            timer: TimerMs::new(0),
        }
    }
    fn delay_is_over(&mut self, is_colliding: bool) -> bool {
        match self.lock_state {
            LockState::Idle => {
                if is_colliding {
                    self.lock_state = LockState::Delaying;
                    self.timer = TimerMs::new(self.delay_ms);
                }
                false
            }
            LockState::Delaying => {
                if !is_colliding {
                    self.lock_state = LockState::Idle;
                    return false;
                }

                if self.timer.update() {
                    self.lock_state = LockState::Idle;
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
    spawn_center: Position,
    gravity_delay: TimerMs,
    placement_delay: PlacementDelayManager,
    next: Tetramino,
    hold: Option<Tetramino>,
    allow_hold: bool,
}

impl TetraminoManager {
    fn new(
        center_point: Position,
        gravity_delay_ms: usize,
        placement_delay_ms: usize,
    ) -> TetraminoManager {
        TetraminoManager {
            active: ActiveTetramino::spawn(Tetramino::construct(rand::random()), center_point),
            spawn_center: center_point,
            gravity_delay: TimerMs::new(gravity_delay_ms),
            placement_delay: PlacementDelayManager::new(placement_delay_ms),
            next: Tetramino::construct(rand::random()),
            hold: None,
            allow_hold: true,
        }
    }
    fn get_active(&self) -> &ActiveTetramino {
        &self.active
    }
    fn propogate_gravity(&mut self) {
        self.active
            .translate_with_offset(Position { row: 1, col: 0 });
    }
    fn next_tetramino(&mut self) {
        let next = std::mem::take(&mut self.next);
        self.active = ActiveTetramino::spawn(next, self.spawn_center);
        self.next = Tetramino::construct(rand::random());
    }
    fn rotate(&self, direction: RotationDirection) -> RotationResult {
        self.active.get_rotation_result(direction)
    }
    fn swap_hold(&mut self) {
        if !self.allow_hold {
            return;
        }

        match &self.hold {
            Some(_) => {
                let h = std::mem::take(&mut self.hold);
                self.hold = Some(Tetramino::construct(self.active.get_kind()));
                self.allow_hold = false;
                self.active = ActiveTetramino::spawn(h.unwrap(), self.spawn_center);
            }
            None => {
                self.hold = Some(Tetramino::construct(self.active.get_kind()));
                self.allow_hold = false;
                let n = std::mem::replace(&mut self.next, Tetramino::construct(rand::random()));
                self.active = ActiveTetramino::spawn(n, self.spawn_center);
            }
        }
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
            tetramino_manager: TetraminoManager::new(
                Position::new(0, playfield_size.cols / 2),
                gravity_delay_ms,
                placement_delay_ms,
            ),
        }
    }
    pub fn get_playfield_size(&self) -> &PlayfieldSize {
        &self.playfield.size
    }
    pub fn get_placed_blocks(&self) -> &PlacedBlocks {
        &self.playfield.placed_blocks
    }
    pub fn get_active_tetramino(&self) -> &ActiveTetramino {
        self.tetramino_manager.get_active()
    }
    pub fn get_next_blocks(&self) -> &HashSet<Block> {
        self.tetramino_manager.next.get_blocks()
    }
    pub fn get_hold_blocks(&self) -> Option<&HashSet<Block>> {
        match &self.tetramino_manager.hold {
            Some(t) => Some(t.get_blocks()),
            None => None,
        }
    }

    pub fn get_hard_drop_blocks(&self) -> HashSet<Block> {
        let mut collision = self
            .playfield
            .check_collisions(&self.tetramino_manager.active.get_blocks_with_offset(None));

        let mut offset = 1;
        while !collision.down {
            collision = self.playfield.check_collisions(
                &self
                    .tetramino_manager
                    .active
                    .get_blocks_with_offset(Some(Position::new(offset, 0))),
            );
            offset += 1;
        }

        self.tetramino_manager
            .active
            .get_blocks_with_offset(Some(Position::new(offset - 1, 0)))
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
    fn apply_gravity(&mut self, is_colliding: bool, reset_timer: bool) {
        if !is_colliding {
            self.tetramino_manager.propogate_gravity();
            if reset_timer {
                self.tetramino_manager.gravity_delay.reset();
            }
        }
    }
    fn swap_hold(&mut self) {
        self.tetramino_manager.swap_hold();
    }
    fn line_completion(&mut self, subject: &HashSet<Block>) {
        self.playfield.line_completion(subject);
    }
    fn next_turn(&mut self, active_blocks: HashSet<Block>) {
        self.playfield.put_blocks(&active_blocks);
        self.line_completion(&active_blocks);
        self.tetramino_manager.allow_hold = true;
        self.tetramino_manager.next_tetramino();
    }
    pub fn update(&mut self, player_intent: PlayerIntention) {
        let active_blocks_on_playfield = self.tetramino_manager.active.get_blocks_with_offset(None);

        let collision = self.playfield.check_collisions(&active_blocks_on_playfield);

        match player_intent {
            PlayerIntention::None => {
                if self.tetramino_manager.gravity_delay.update() {
                    self.apply_gravity(collision.down, false);
                }
            }
            PlayerIntention::MoveLeft => {
                if !collision.left {
                    self.tetramino_manager
                        .active
                        .translate_with_offset(Position::new(0, -1));
                }
            }
            PlayerIntention::MoveRight => {
                if !collision.right {
                    self.tetramino_manager
                        .active
                        .translate_with_offset(Position::new(0, 1));
                }
            }
            PlayerIntention::RotateClockwise => {
                self.try_rotate(RotationDirection::Clockwise);
            }
            PlayerIntention::RotateCounterClockwise => {
                self.try_rotate(RotationDirection::CounterClockwise);
            }
            PlayerIntention::DropSoft => {
                self.apply_gravity(collision.down, true);
            }
            PlayerIntention::DropHard => {
                self.next_turn(self.get_hard_drop_blocks());
            }
            PlayerIntention::HoldPiece => {
                self.swap_hold();
            }
        }

        if self
            .tetramino_manager
            .placement_delay
            .delay_is_over(collision.down)
        {
            self.next_turn(active_blocks_on_playfield);
        }
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
            wait_ms,
        }
    }
    pub fn reset(&mut self) {
        self.deadline = Instant::now() + Duration::from_millis(self.wait_ms as u64);
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
