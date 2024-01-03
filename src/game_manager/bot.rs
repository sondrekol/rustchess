use crate::game_manager::board2::BoardState;
use super::board2::ChessMove;


pub struct GetMoveResult{
    chess_move: ChessMove,
    searched_positions: usize,
    eval: f64
}

impl GetMoveResult{
    pub fn new(chess_move:ChessMove, searched_positions:usize, eval:f64) -> Self{
        Self { chess_move: chess_move, searched_positions: searched_positions, eval: eval}
    }

    pub fn chess_move(&self) -> &ChessMove{
        &self.chess_move
    }

    pub fn num_pos(&self) -> usize{
        self.searched_positions
    }
    pub fn eval(&self) -> f64{
        self.eval
    }
}

pub trait Bot {
    fn new() -> Self;
    fn get_move(&mut self, board_state:BoardState) -> GetMoveResult;
}