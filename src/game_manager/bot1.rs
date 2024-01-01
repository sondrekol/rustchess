use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;
use rand::Rng;



pub struct Bot1{

}


impl Bot for Bot1{
    fn new() -> Self{
        Self{}
    }

    fn get_move(&self, board_state:BoardState) -> ChessMove{
        let moves = board_state.legal_moves();
        let mut picked_move:ChessMove = ChessMove::new_empty();
        let mut rng = rand::thread_rng();

        while picked_move.is_null(){
            let n1: usize = rng.gen_range(0..218);
            picked_move = moves.moves()[n1];
        }
        return picked_move;
    }
}