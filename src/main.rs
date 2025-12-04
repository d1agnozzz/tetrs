use std::{
    collections::HashSet,
    sync::{mpsc, Arc, RwLock},
    thread,
    time::Duration,
};

use macroquad::prelude::*;
use tetrs::InputEvent;
use tetrs::{process_logic, GameState};

#[macroquad::main("MyGame")]
async fn main() {
    let mut game_state = GameState::new();
    let mut frame_counter = 0;
    let mut input_buf = HashSet::<KeyCode>::new();
    let mut time_start = std::time::Instant::now();

    loop {
        let inputs = InputEvent {
            keys: get_keys_pressed(),
        };

        process_logic(&mut game_state, inputs);

        println!("Entered UI loop!");
        clear_background(BLACK);

        let mut x_shift = 0.;
        let mut y_shift = 50.;

        for row in &game_state.playfield.field {
            for col in row {
                match col {
                    Some(block) => {
                        draw_rectangle(x_shift, y_shift, 10.0, 10.0, block.color);
                    }
                    None => {
                        draw_rectangle(x_shift, y_shift, 10.0, 10.0, GRAY);
                    }
                }

                x_shift += 15.;
            }
            y_shift += 15.;
            x_shift = 0.;
        }

        draw_fps();
        frame_counter += 1;

        println!("Drew a frame!");
        next_frame().await;
    }
}
