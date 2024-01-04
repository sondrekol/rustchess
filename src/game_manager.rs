use std::{thread::{self, JoinHandle}, ptr::null, time::SystemTime};

use self::{board2::{BoardState, ChessMove},
bot::{Bot, GetMoveResult}, 
bot2::Bot2, 
state_bitboard::bit_boards::{populate_rook_moves, populate_bishop_moves}};

mod board2;
mod board2tests;
mod bot;
mod bot2;
mod bot2bench;
mod state_bitboard;
mod state_bitboard_tests;
pub struct GameManager{
    player_color: bool,
    board_state: BoardState,
    turn: bool,
    bot: Bot2,
    bot_thread: Option<JoinHandle<GetMoveResult>>,
    bot_start_time: SystemTime

}

impl GameManager{

    pub fn new_game(color: bool, fen: &str) -> Self{
        //setup rook and bisshop moves
        state_bitboard::bit_boards::populate_rook_moves();
        state_bitboard::bit_boards::populate_bishop_moves();
        
        Self{
            player_color: color,
            turn: true,
            board_state: BoardState::new_from_fen(fen),
            bot: Bot2::new(),
            bot_thread: None,
            bot_start_time: SystemTime::now()
        }
    }

    pub fn try_move(&mut self, origin:u8, target:u8, promotion_piece:u8){
        if self.turn == self.player_color{
            if let Some(new_board_state) = self.board_state.perform_move_api(origin, target, promotion_piece){
                self.board_state = new_board_state;
                self.turn = !self.turn;
            }
        }
    }

    
    pub fn update(&mut self) -> bool{
        if self.turn != self.player_color {
            //start the bot if it is not running
            if self.bot_thread.is_none(){

                self.bot_start_time = SystemTime::now();
                let board_state = self.board_state.clone();
                let bot = self.bot;

                self.bot_thread = Some(thread::spawn(move ||{
                    return bot.clone().get_move(board_state);
                }));
                println!("Bot started");
                println!("----------------");



            }
            
            //check if bot has finished, and make the move
            if let Some(handle) = self.bot_thread.take() {
                if handle.is_finished(){
                    let bot_result = handle.join();
                    if bot_result.is_err(){
                        println!("Bot error")
                    }
                    let used_time =self.bot_start_time.elapsed().unwrap().as_millis();
                    let get_move_result = bot_result.unwrap();
                    println!("Bot finished with move: origin: {}, target: {}, flag:{}", get_move_result.chess_move().origin(), get_move_result.chess_move().target(), get_move_result.chess_move().flag());
                    println!("time elapsed: {}", used_time);
                    println!("evaluated {} positions", get_move_result.num_pos());
                    println!("eval: {}", get_move_result.eval());
                    println!("{} kN/s", get_move_result.num_pos()/used_time as usize);
                    println!();
                    self.board_state = self.board_state.perform_move(*get_move_result.chess_move());
                    self.turn = !self.turn;
                    self.bot_thread = None;
                    return true;
                }else{
                    self.bot_thread = Some(handle).take();
                }


            }
            
        }
        return false;
        
    }

    pub fn get_board(&self) -> [[i8; 8]; 8]{
        return self.board_state.get_board();
    }
}