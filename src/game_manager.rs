use std::{thread::{self, JoinHandle}, ptr::null, time::SystemTime};

use crate::game_manager::{state_bitboard::BoardStateNumbers, move_string::lan_move};

use self::{board2::{BoardState, ChessMove},
bot::{Bot, GetMoveResult}, 
bot2::Bot2, 
state_bitboard::bit_boards::{populate_rook_moves, populate_bishop_moves}, bot2_2::Bot2_2, bot2_3::Bot2_3, bot2_4::Bot2_4, bot2_5::Bot2_5, bot2_6::Bot2_6};

mod board2;
mod board2tests;
mod bot;
mod bot2;
mod bot2_2;
mod bot2_3;
mod bot2_4;
mod bot2_5;
mod bot2_6;
mod bot2bench;
mod state_bitboard;
mod state_bitboard_tests;
mod bot_evaluater;
mod transposition_table;
mod move_string;

pub struct GameManager{
    player_color: bool,
    board_state: BoardState,
    turn: bool,
    bot: Bot2_6,
    bot_thread: Option<JoinHandle<GetMoveResult>>,
    bot_start_time: SystemTime

}

impl GameManager{

    pub fn new_game(color: bool, fen: &str) -> Self{
        //setup rook and bisshop moves
        state_bitboard::bit_boards::populate_rook_moves();
        state_bitboard::bit_boards::populate_bishop_moves();
        let board_state = BoardState::new_from_fen(fen);

        Self{
            player_color: color,
            turn: board_state.white_to_move(),
            board_state: board_state,
            bot: Bot2_6::new(15, 25, 1000000, Some(15000)),
            bot_thread: None,
            bot_start_time: SystemTime::now()
        }
    }

    pub fn try_move(&mut self, origin:u8, target:u8, promotion_piece:u8){
        if self.turn == self.player_color{
            if let Some(new_board_state) = self.board_state.perform_move_api(origin, target, promotion_piece){
                self.board_state = new_board_state;
                self.turn = !self.turn;
                println!("player played: {} to {}", origin, target);
            }
        }
    }

    
    pub fn update(&mut self) -> bool{
        if self.turn != self.player_color {
            //start the bot if it is not running
            if self.bot_thread.is_none(){

                self.bot_start_time = SystemTime::now();
                let board_state = self.board_state.clone();
                let bot = self.bot.clone();

                self.bot_thread = Some(thread::spawn(move ||{
                    return bot.clone().get_move(board_state, &mut Vec::<BoardStateNumbers>::new());
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
                    println!("Bot finished with move: {}", lan_move(*get_move_result.chess_move()));
                    println!("time elapsed: {}", used_time);
                    println!("evaluated {} positions", get_move_result.num_pos());
                    println!("eval: {}", (get_move_result.eval() as f64)/100.0);
                    if used_time != 0 {

                        println!("{} kN/s", get_move_result.num_pos()/used_time as usize);
                    }
                    println!("average index of best move: {}", get_move_result.avg_best_move_i());
                    println!("depth reached: {}", get_move_result.depth_reached());
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