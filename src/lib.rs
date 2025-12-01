use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use macroquad::color::{Color, GREEN};

#[derive(Clone, Copy)]
pub struct Block {
    pub color: Color,
}

const PLAYFIELD_ROWS: usize = 20;
const PLAYFIELD_COLS: usize = 10;

pub struct GameState {
    pub playfield: Playfield,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            playfield: Playfield::new(None, None),
        }
    }
    pub fn update(&mut self, row: usize, col: usize) {
        self.playfield.field[row][col] = match self.playfield.field[row][col] {
            Some(_) => None,
            None => Some(Block { color: GREEN }),
        }
    }
}

pub struct Playfield {
    pub field: Vec<Vec<Option<Block>>>,
}

impl Playfield {
    fn new(rows: Option<usize>, cols: Option<usize>) -> Playfield {
        Playfield {
            field: vec![vec![None; rows.unwrap_or(PLAYFIELD_COLS)]; cols.unwrap_or(PLAYFIELD_ROWS)],
        }
    }
}

pub fn game_loop(game_state: Arc<RwLock<GameState>>) {
    let mut cur_row = 0;
    loop {
        println!("Entered WACKY loop!");
        {
            let mut guard = game_state.write().unwrap();
            guard.update(cur_row, 5);
            cur_row += 1;
            cur_row %= PLAYFIELD_ROWS;
        }
        println!("Processed logic!");
        std::thread::sleep(Duration::from_millis(50));
    }
}
