use std::{
    sync::{Arc, RwLock},
    thread,
};

use macroquad::prelude::*;
use tetrs::GameState;

#[macroquad::main("MyGame")]
async fn main() {
    let game_state = Arc::new(RwLock::new(GameState::new()));

    let state_clone = game_state.clone();

    thread::spawn(move || {
        tetrs::game_loop(state_clone);
    });

    loop {
        println!("Entered UI loop!");
        clear_background(BLACK);

        let mut x_shift = 0.;
        let mut y_shift = 0.;

        {
            let guard = game_state.read().unwrap();

            for row in &guard.playfield.field {
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
        }

        println!("Drew a frame!");
        next_frame().await
    }
}
