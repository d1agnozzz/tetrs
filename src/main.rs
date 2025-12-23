use macroquad::{color::Color, prelude::*};
use tetrs::{process_logic, GameState, InputEvent, MovingTetramino, PlacedBlocks, PlayfieldSize};

fn draw_current_tetramino(cur_tetramino: &MovingTetramino, grid_painter: &SquareBitGridPainter) {
    for block in &cur_tetramino.shape_with_offset() {
        grid_painter.draw_grid_cell(
            block.coordinates.row,
            block.coordinates.col,
            block.color,
        );
    }
}

fn draw_placed_blocks(placed: &PlacedBlocks, grid_painter: &SquareBitGridPainter) {
    for block in placed.get_blocks() {
        grid_painter.draw_grid_cell(block.coordinates.row, block.coordinates.col, block.color);
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

struct SquareBitGridPainter {
    grid_size: GridSize,
    deactivated_color: Color,
    origin: Coordinate,
    cell_size: f32,
    grid_spacing: f32,
}

impl SquareBitGridPainter {
    fn new(
        size: GridSize,
        default_color: Color,
        position_origin: Coordinate,
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
    let game_grid_painter = SquareBitGridPainter::new(
        GridSize {
            rows: game_state.playfield_size.rows,
            cols: game_state.playfield_size.cols,
        },
        GRAY,
        Coordinate { x: 50., y: 50. },
        10.0,
        5.0,
    );
    game_grid_painter.draw_empty_grid();
    draw_placed_blocks(&game_state.placed_blocks, &game_grid_painter);
    draw_current_tetramino(&game_state.current_tetramino, &game_grid_painter);
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut game_state = GameState::new(PlayfieldSize { rows: 20, cols: 10 });

    loop {
        let inputs = InputEvent {
            keys: get_keys_pressed(),
        };

        process_logic(&mut game_state, inputs);
        clear_background(BLACK);
        draw_game_frame(&game_state);
        draw_fps();
        next_frame().await;
    }
}
