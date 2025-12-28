use std::collections::HashSet;

use macroquad::{color::Color, prelude::*};
use tetrs::{Block, GameState, PlayerIntention, PlayfieldSize};


struct UIPosition {
    x: f32,
    y: f32,
}

struct GridSize {
    rows: isize,
    cols: isize,
}

struct SquareBitGridPainter {
    grid_size: GridSize,
    deactivated_color: Color,
    origin: UIPosition,
    cell_size: f32,
    grid_spacing: f32,
}

impl SquareBitGridPainter {
    fn new(
        size: GridSize,
        default_color: Color,
        position_origin: UIPosition,
        cell_size: f32,
        cells_spacing: f32,
    ) -> Self {
        SquareBitGridPainter {
            grid_size: size,
            deactivated_color: default_color,
            origin: position_origin,
            cell_size,
            grid_spacing: cells_spacing,
        }
    }

    pub fn cell_origin(&self, row: isize, col: isize) -> UIPosition {
        UIPosition {
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

    fn draw_grid_cell(&self, row: isize, col: isize, color: Color) {
        let cell_origin = UIPosition {
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
    pub fn draw_blocks(&self, blocks: &HashSet<Block>, override_color: Option<Color>) {
        for block in blocks {
            self.draw_grid_cell(block.coordinates.row, block.coordinates.col, override_color.unwrap_or(block.color.into()));
        }
    }
}

fn draw_game_frame(game_state: &GameState, bg_color: Color, cell_size: f32, cells_spacing: f32) {
    let playfield_painter = SquareBitGridPainter::new(
        GridSize {
            rows: game_state.get_playfield_size().rows,
            cols: game_state.get_playfield_size().cols,
        },
        bg_color,
        UIPosition { x: 100., y: 50. },
        cell_size,
        cells_spacing,
    );
    playfield_painter.draw_empty_grid();
    playfield_painter.draw_blocks(game_state.get_placed_blocks().get_blocks(), None);
    playfield_painter.draw_blocks(&game_state.get_hard_drop_blocks(), Some(GRAY));
    playfield_painter.draw_blocks(&game_state.get_active_tetramino().get_blocks_with_offset(None), None);

    draw_text("next", 300. , 40. , 30. , WHITE);
    let next_painter = SquareBitGridPainter::new(
        GridSize { rows: 4, cols: 4 },
        BLACK,
        UIPosition { x: 300., y: 50. },
        cell_size,
        cells_spacing,
    );
    next_painter.draw_empty_grid();
    next_painter.draw_blocks(game_state.get_next_blocks(), None);

    draw_text("hold", 20. , 40. , 30. , WHITE);
    let hold_painter = SquareBitGridPainter::new(
        GridSize { rows: 4, cols: 4 },
        BLACK,
        UIPosition { x: 20., y: 50. },
        cell_size,
        cells_spacing,
    );
    hold_painter.draw_empty_grid();
    if let Some(b) = &game_state.get_hold_blocks() { hold_painter.draw_blocks(b, None) }
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut game_state = GameState::new(PlayfieldSize::new(20, 10), 500, 1000);

    loop {
        let input_keys = get_keys_pressed();
        let mut player_intent = PlayerIntention::None;

        if input_keys.contains(&KeyCode::A) {
            player_intent = PlayerIntention::MoveLeft;
        } else if input_keys.contains(&KeyCode::D) {
            player_intent = PlayerIntention::MoveRight;
        } else if input_keys.contains(&KeyCode::E) {
            player_intent = PlayerIntention::RotateClockwise;
        } else if input_keys.contains(&KeyCode::Q) {
            player_intent = PlayerIntention::RotateCounterClockwise;
        } else if input_keys.contains(&KeyCode::S) {
            player_intent = PlayerIntention::DropSoft;
        } else if input_keys.contains(&KeyCode::Space) {
            player_intent = PlayerIntention::DropHard;
        } else if input_keys.contains(&KeyCode::LeftShift) {
            player_intent = PlayerIntention::HoldPiece;
        }

        game_state.update(player_intent);
        clear_background(BLACK);
        draw_game_frame(&game_state, DARKGRAY, 15.0, 1.0);
        draw_fps();
        next_frame().await;
    }
}
