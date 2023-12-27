extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::path::Path;

use glutin_window::GlutinWindow as Window;
use graphics::rectangle::square;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs};
use piston::window::WindowSettings;
use piston::{
    Button, ButtonEvent, ButtonState, Key, MouseButton, MouseCursorEvent, MouseScrollEvent
};
use graphics::{Image, clear};
pub struct App {
    gl: GlGraphics
}

fn mix_colors(color1: [f32; 4], color2: [f32; 4], gradient: f32) -> [f32; 4] {
    let mut result: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
    for i in 0..4 {
        let diff: f32 = (color1[i] - color2[i]).abs();
        if color1[i] > color2[i] {
            result[i] = color2[i] + (diff * gradient);
        } else {
            result[i] = color1[i] + (diff * gradient);
        }
    }

    return result;
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACKGROUND: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
        const CHESS_BOARD_WHITE: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
        const CHESS_BOARD_BLACK: [f32; 4] = [0.4, 0.4, 0.4 , 1.0];

        let window_width = args.window_size[0];
        let window_length = args.window_size[1];
        let board_size = if window_width < window_length {window_width} else {window_length};


        fn draw_board(c: Context, gl: &mut GlGraphics, size: f64){

            let square_size = size/8.0;

            for i in 0..8{
                for j in 0..8{
                    rectangle(
                        if (i+j)%2==0 {CHESS_BOARD_WHITE} else {CHESS_BOARD_BLACK}, 
                        rectangle::square(square_size*(i as f64),square_size*(j as f64),square_size), 
                        c.transform.trans(0.0, 0.0), 
                        gl);
                }
            }

        }

        fn draw_pieces(c: Context, gl: &mut GlGraphics, size: f64){
            let image = Image::new().rect(square(10.0, 10.0, 400.0));
            //A texture to use with the image
            let texture = Texture::from_path(Path::new("assets/Chess_Pieces_Sprite.png"), &TextureSettings::new()).unwrap();
            image.draw(&texture, &DrawState::default(), c.transform, gl);
        }


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BACKGROUND, gl);
            draw_board(c, gl, board_size);
            draw_pieces(c, gl, board_size);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {


    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl: OpenGL = OpenGL::V3_2;
    let left_mouse = Button::Mouse(MouseButton::Left);
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Bot", [800, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl)
    };

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
    }
}
