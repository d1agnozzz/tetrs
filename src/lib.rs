use macroquad::color::Color;

#[derive(Clone, Copy)]
pub struct Block {
    color: Color,
}

const PLAYFIELD_ROWS: usize = 20;
const PLAYFIELD_COLS: usize = 10;

pub static playfield: [[Option<Block>; PLAYFIELD_COLS]; PLAYFIELD_ROWS] =
    [[None; PLAYFIELD_COLS]; PLAYFIELD_ROWS];
