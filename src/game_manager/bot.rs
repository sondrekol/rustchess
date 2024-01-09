use crate::game_manager::board2::BoardState;
use super::{board2::ChessMove, state_bitboard::{BitBoardState, BoardStateNumbers}};


pub struct GetMoveResult{
    chess_move: ChessMove,
    searched_positions: usize,
    eval: i32,
    average_best_move_index: f64,
    depth_reached: u32,
}

impl GetMoveResult{
    pub fn new(chess_move:ChessMove, searched_positions:usize, eval:i32, avg_best_move_i:f64, depth_reached: u32) -> Self{
        Self { chess_move: chess_move, searched_positions: searched_positions, eval: eval, average_best_move_index: avg_best_move_i, depth_reached: depth_reached}
    }

    pub fn chess_move(&self) -> &ChessMove{
        &self.chess_move
    }

    pub fn num_pos(&self) -> usize{
        self.searched_positions
    }
    pub fn eval(&self) -> i32{
        self.eval
    }
    pub fn avg_best_move_i(&self) -> f64 {
        return self.average_best_move_index;
    }

    pub fn depth_reached(&self) -> u32{
        return self.depth_reached;
    }
}

pub trait Bot {
    fn default() -> Self;
    fn new(search_depth: i64, max_depth: usize, table_size: usize, max_time: Option<u128>) -> Self;
    fn get_move(&mut self, board_state:BoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult;
    fn get_move_bb(&mut self, board_state:BitBoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult;

}