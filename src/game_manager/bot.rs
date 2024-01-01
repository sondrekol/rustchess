use crate::game_manager::board2::BoardState;
use super::board2::ChessMove;

pub trait Bot {
    fn new() -> Self;
    fn get_move(&self, board_state:BoardState) -> ChessMove;
}