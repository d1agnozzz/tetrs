use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    usize,
};

use macroquad::{
    color::{Color, GREEN},
    input::KeyCode,
    time::get_frame_time,
};

#[derive(Debug)]
pub struct InputEvent {
    pub keys: HashSet<KeyCode>,
}

#[derive(Clone, Copy)]
pub struct Block {
    pub color: Color,
}

const PLAYFIELD_ROWS: usize = 20;
const PLAYFIELD_COLS: usize = 10;

pub struct GameState {
    pub playfield: Playfield,
    pub descend_delay: Timer_ms,
    pub column_toggling: isize,
    pub row_toggleing: isize,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            playfield: Playfield::new(None, None),
            descend_delay: Timer_ms::new(10000),
            column_toggling: 5,
            row_toggleing: 0,
        }
    }
    pub fn toggle_block(&mut self, row: usize, col: usize) {
        self.playfield.field[row][col] = match self.playfield.field[row][col] {
            Some(_) => None,
            None => Some(Block { color: GREEN }),
        }
    }
    pub fn toggle_block_in_row(&mut self) {
        self.descend_delay.update();
        if self.descend_delay.is_out() {
            self.playfield.field[self.row_toggleing as usize][self.column_toggling as usize] =
                match self.playfield.field[self.row_toggleing as usize]
                    [self.column_toggling as usize]
                {
                    Some(_) => None,
                    None => Some(Block { color: GREEN }),
                };
            self.row_toggleing = (self.row_toggleing + 1).rem_euclid(PLAYFIELD_ROWS as isize);
            self.descend_delay.reset();
        }
    }
    pub fn update_column_toggling_relative(&mut self, value: isize) {
        self.column_toggling = match value.cmp(&(0 as isize)) {
            std::cmp::Ordering::Less => {
                (self.column_toggling - value.abs()).rem_euclid(PLAYFIELD_COLS as isize)
            }
            std::cmp::Ordering::Greater => {
                (self.column_toggling + value).rem_euclid(PLAYFIELD_COLS as isize)
            }
            _ => self.column_toggling,
        };
    }
}

#[derive(Clone)]
pub struct Playfield {
    pub field: Vec<Vec<Option<Block>>>,
}

#[derive(Clone, Copy)]
pub struct Timer_ms {
    time_start: std::time::Instant,
    elapsed: i64,
    wait: i64,
}

impl Timer_ms {
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

impl Playfield {
    fn new(rows: Option<usize>, cols: Option<usize>) -> Playfield {
        Playfield {
            field: vec![vec![None; rows.unwrap_or(PLAYFIELD_COLS)]; cols.unwrap_or(PLAYFIELD_ROWS)],
        }
    }
}

pub fn process_logic(game_state: &mut GameState, input: InputEvent) {
    // let mut descend_delay = Timer_ms::new(10);
    // descend_delay.update(frame_time_s.elapsed().as_millis() as i64);
    // if descend_delay.is_out() {
    // descend_delay.reset();
    // frame_time_s = std::time::Instant::now();
    game_state.toggle_block_in_row();
    // }
}
