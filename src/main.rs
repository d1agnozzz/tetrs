use macroquad::prelude::*;
use tetrs::playfield;



#[macroquad::main("MyGame")]
async fn main() {
    loop {
        clear_background(BLACK);

        let mut x_shift = 0.;
        let mut y_shift = 0.;
        for row in playfield {
            for col in row {
                draw_rectangle(x_shift, y_shift, 10.0, 10.0, GREEN);
                x_shift += 15.;
            }
            y_shift += 15.;
            x_shift = 0.;
        }


        next_frame().await
    }
}
