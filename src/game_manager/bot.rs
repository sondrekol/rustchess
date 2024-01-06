use crate::game_manager::board2::BoardState;
use super::board2::ChessMove;


pub struct GetMoveResult{
    chess_move: ChessMove,
    searched_positions: usize,
    eval: i32,
    average_best_move_index: f64
}

impl GetMoveResult{
    pub fn new(chess_move:ChessMove, searched_positions:usize, eval:i32, avg_best_move_i:f64) -> Self{
        Self { chess_move: chess_move, searched_positions: searched_positions, eval: eval, average_best_move_index: avg_best_move_i }
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
}

pub trait Bot {
    fn new() -> Self;
    fn get_move(&mut self, board_state:BoardState) -> GetMoveResult;
}