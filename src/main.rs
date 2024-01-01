extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::path::Path;

use game_manager::GameManager;
use glutin_window::GlutinWindow as Window;
use graphics::rectangle::square;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs};
use piston::window::WindowSettings;
use piston::{
    Button, MouseButton, ButtonEvent, MouseCursorEvent, ButtonState, UpdateEvent
};
use graphics::{Image, clear};
use std::collections::HashMap;


mod game_manager;

pub struct App {
    gl: GlGraphics,
    piece_textures:HashMap<i8, Texture>,
    board_graphic_state: [[i8;8];8],
    game_manager:GameManager,
    piece_in_hand: i8,
    mouse_position: [f64; 2],
    board_size: f64,
    move_origin: (usize, usize),
    promotion_piece: Option<u8>,
    reverse_board: bool

}


fn _mix_colors(color1: [f32; 4], color2: [f32; 4], gradient: f32) -> [f32; 4] {
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
        self.board_size = if window_width < window_length {window_width} else {window_length};



        fn draw_board(c: Context, gl: &mut GlGraphics, size: f64, reverse: bool){

            let square_size = size/8.0;

            for i in 0..8{
                for j in 0..8{

                    let i = if reverse {7-i} else {i};
                    //draw chess square
                    rectangle(
                        if (i+j)%2==0 {CHESS_BOARD_WHITE} else {CHESS_BOARD_BLACK}, 
                        rectangle::square(square_size*(i as f64),square_size*(j as f64),square_size), 
                        c.transform, 
                        gl);

                    //TODO: draw board labels
                    /*const board_labels_font_size:u32 = 12;
                    let text_white:Text = Text::new_color(CHESS_BOARD_WHITE, board_labels_font_size);
                    let text_black:Text = Text::new_color(CHESS_BOARD_WHITE, board_labels_font_size);
                    const COLUMNS:[char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                    
                    let text:Text;
                    if j % 2 == 0 {
                        text = text_black;
                    }else{
                        text = text_white;
                    }
                    text.draw_pos(&(COLUMNS[j].to_string()), 
                        [square_size*(i as f64),square_size*(j as f64)], 
                        Glyphs::new(), 
                        &c.draw_state, 
                        c.transform, 
                        gl);*/


                }
            }

        }

        fn draw_pieces(c: Context, gl: &mut GlGraphics, size: f64, textures:&HashMap<i8, Texture>, board_state: [[i8; 8]; 8], reverse:bool){
            let square_size = size/8.0;

            for i in 0..8{
                for j in 0..8{

                    let i = if reverse {7-i} else {i};

                    let piece_code = board_state[j][i];
                    if piece_code != 0{

                        let image = Image::new().rect(square(square_size*(i as f64),square_size*(j as f64),square_size));
                        image.draw(textures.get(&piece_code).unwrap(), &c.draw_state, c.transform, gl);
                    }
                }
            }
        }


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BACKGROUND, gl);
            draw_board(c, gl, self.board_size, self.reverse_board);
            draw_pieces(c, gl, self.board_size, &self.piece_textures, self.board_graphic_state, self.reverse_board);


            let square_size = self.board_size/8.0;
            if self.piece_in_hand != 0{
                let image = Image::new().rect(square(self.mouse_position[0]-square_size/2.0,self.mouse_position[1]-square_size/2.0,square_size));
                image.draw(self.piece_textures.get(&self.piece_in_hand).unwrap(), &c.draw_state, c.transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.game_manager.update(){

            self.board_graphic_state = self.game_manager.get_board();
        }
    }

    fn cursor_board_coordinates(&mut self) -> (usize, usize){
        let x = (self.mouse_position[1]*8.0/self.board_size).floor() as usize;
        let y = (self.mouse_position[0]*8.0/self.board_size).floor() as usize;
        return (x, y)
    }

    fn pick_up_piece(&mut self){
        let (x, y) = self.cursor_board_coordinates();
        if(x < 8 && y < 8){
            self.piece_in_hand = self.board_graphic_state[x][y];
            self.board_graphic_state[x][y] = 0;
        }

        self.move_origin = (x, y);
    }

    fn put_down_piece(&mut self){
        let (x, y) = self.cursor_board_coordinates();
        if x < 8 && y < 8 {
            let origin = ((7-self.move_origin.0)*8 + self.move_origin.1) as u8;
            let target= ((7-x)*8 + y) as u8;
            self.game_manager.try_move(origin, target, self.promotion_piece.unwrap_or(0));
            
            
            self.piece_in_hand = 0;
            self.board_graphic_state = self.game_manager.get_board();
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl: OpenGL = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Bot", [800, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),

        piece_textures: HashMap::from([
                    (1, Texture::from_path(Path::new("assets/pieces-basic-png/white-pawn.png"), &TextureSettings::new()).unwrap()),
                    (2, Texture::from_path(Path::new("assets/pieces-basic-png/white-knight.png"), &TextureSettings::new()).unwrap()),
                    (3, Texture::from_path(Path::new("assets/pieces-basic-png/white-bishop.png"), &TextureSettings::new()).unwrap()),
                    (4, Texture::from_path(Path::new("assets/pieces-basic-png/white-rook.png"), &TextureSettings::new()).unwrap()),
                    (5, Texture::from_path(Path::new("assets/pieces-basic-png/white-queen.png"), &TextureSettings::new()).unwrap()),
                    (6, Texture::from_path(Path::new("assets/pieces-basic-png/white-king.png"), &TextureSettings::new()).unwrap()),
                    (-1, Texture::from_path(Path::new("assets/pieces-basic-png/black-pawn.png"), &TextureSettings::new()).unwrap()),
                    (-2, Texture::from_path(Path::new("assets/pieces-basic-png/black-knight.png"), &TextureSettings::new()).unwrap()),
                    (-3, Texture::from_path(Path::new("assets/pieces-basic-png/black-bishop.png"), &TextureSettings::new()).unwrap()),
                    (-4, Texture::from_path(Path::new("assets/pieces-basic-png/black-rook.png"), &TextureSettings::new()).unwrap()),
                    (-5, Texture::from_path(Path::new("assets/pieces-basic-png/black-queen.png"), &TextureSettings::new()).unwrap()),
                    (-6, Texture::from_path(Path::new("assets/pieces-basic-png/black-king.png"), &TextureSettings::new()).unwrap()),
                ]),
        

        board_graphic_state: [[0; 8];8],
        
        game_manager: GameManager::new_game(true, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        piece_in_hand: 0,
        mouse_position: [0.0, 0.0],
        board_size: 0.0,
        move_origin: (0, 0),
        promotion_piece: None,
        reverse_board: false
        
        
    };
    app.board_graphic_state = app.game_manager.get_board();


    let mut events = Events::new(EventSettings::new());


    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
        if let Some(args) = e.button_args(){
            if args.button == Button::Keyboard(piston::Key::Q){
                app.promotion_piece = Some(0b00000100);
            }
            if args.button == Button::Keyboard(piston::Key::R){
                app.promotion_piece = Some(0b00000011);
            }
            if args.button == Button::Keyboard(piston::Key::B){
                app.promotion_piece = Some(0b00000010);
            }
            if args.button == Button::Keyboard(piston::Key::N){
                app.promotion_piece = Some(0b00000001);
            }
            if args.button == Button::Keyboard(piston::Key::R) && args.state == ButtonState::Press{
                app.reverse_board = !app.reverse_board;
            }

        }
        if let Some(args) = e.mouse_cursor_args(){
            app.mouse_position = args;
        }
        if let Some(args) = e.button_args(){
            if args.button == Button::Mouse(piston::MouseButton::Left) && args.state == ButtonState::Press{
                if app.piece_in_hand == 0{
                    app.pick_up_piece();
                }else{
                    app.put_down_piece();
                }
            }
        }
        if let Some(args) = e.update_args(){
            app.update(&args);
        }
    }
}
