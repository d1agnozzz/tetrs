use std::{cell, collections::HashSet};

use macroquad::{color::Color, prelude::*};
use tetrs::{process_logic, FallingTetramino, GameState, InputEvent, PlayfieldSize};

fn draw_current_tetramino(cur_tetramino: &FallingTetramino, grid: &SquareBitGridDrawer) {
    for block in &cur_tetramino.shape.blocks {
        grid.draw_grid_cell(block.coordinates.row + cur_tetramino.offset.y_offset, block.coordinates.col + cur_tetramino.offset.x_offset, block.color);
    }
}

struct Coordinate {
    x: f32,
    y: f32,
}

struct GridSize {
    rows: isize,
    cols: isize,
}

struct SquareBitGridDrawer {
    grid_size: GridSize,
    deactivated_color: Color,
    origin: Coordinate,
    cell_size: f32,
    grid_spacing: f32,
}

impl SquareBitGridDrawer {
    fn new(
        size: GridSize,
        default_color: Color,
        position_origin: Coordinate,
        cell_size: f32,
        cells_spacing: f32,
    ) -> Self {
        SquareBitGridDrawer {
            grid_size: size,
            deactivated_color: default_color,
            origin: position_origin,
            cell_size: cell_size,
            grid_spacing: cells_spacing,
        }
    }

    pub fn cell_origin(&self, row: isize, col: isize) -> Coordinate {
        Coordinate {
            x: col as f32 * self.cell_size + col as f32 * self.grid_spacing + self.origin.x,
            y: row as f32 * self.cell_size + row as f32 * self.grid_spacing + self.origin.y,
        }
    }

    pub fn draw_empty_grid(&self) {
        for r in 0..self.grid_size.rows {
            for c in 0..self.grid_size.cols {
                draw_rectangle(
                    self.cell_origin(r, c).x,
                    self.cell_origin(r, c).y,
                    self.cell_size,
                    self.cell_size,
                    self.deactivated_color,
                );
            }
        }
    }

    pub fn draw_grid_cell(&self, row: isize, col: isize, color: Color) {
        let cell_origin = Coordinate {
            x: col as f32 * self.cell_size + col as f32 * self.grid_spacing + self.origin.x,
            y: row as f32 * self.cell_size + row as f32 * self.grid_spacing + self.origin.y,
        };
        draw_rectangle(
            cell_origin.x,
            cell_origin.y,
            self.cell_size,
            self.cell_size,
            color,
        );
    }
}


fn draw_game_frame(game_state: &GameState) {
    let game_grid_drawer = SquareBitGridDrawer::new(GridSize { rows: game_state.playfield_size.rows, cols: game_state.playfield_size.cols },GRAY, Coordinate { x: 50., y: 50. }, 10.0, 5.0);
    game_grid_drawer.draw_empty_grid();
    draw_current_tetramino(&game_state.current_tetramino, &game_grid_drawer);
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut game_state = GameState::new(PlayfieldSize { rows: 20, cols: 10 });
    let mut frame_counter = 0;
    let mut input_buf = HashSet::<KeyCode>::new();
    let mut time_start = std::time::Instant::now();

    loop {
        let inputs = InputEvent {
            keys: get_keys_pressed(),
        };

        process_logic(&mut game_state, inputs);
        clear_background(BLACK);
        draw_game_frame(&game_state);
        draw_fps();
        frame_counter += 1;
        next_frame().await;
    }
}
