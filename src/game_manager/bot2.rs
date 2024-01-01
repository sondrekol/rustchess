use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;
use rand::Rng;

use super::board2::GameState;



pub struct Bot2{

}


impl Bot for Bot2{
    fn new() -> Self{
        Self{}
    }

    fn get_move(&self, board_state:BoardState) -> ChessMove{
        return self.search(&board_state, 5, f64::MIN, f64::MAX).1;
    }
}


impl Bot2 {

    fn evaluate(&self, board_state:&BoardState) -> f64{
        let game_state = board_state.game_state();
        match game_state{
            GameState::Black => {return f64::MIN}
            GameState::White => {return f64::MAX}
            GameState::Draw => {return 0.0}
            GameState::Playing => {}
        }
        let mut rng = rand::thread_rng();

        return board_state.piece_count() + rng.gen_range(-0.001..0.001);
    }

    fn search(&self, board_state:&BoardState, depth:usize, mut alpha:f64, mut beta:f64) -> (f64, ChessMove){
        if depth == 0 || board_state.has_ended(){
            return (self.evaluate(board_state), ChessMove::new_empty())
        }

        let moves = board_state.legal_moves().moves_vec();
        let mut min:f64 = f64::MAX;
        let mut max:f64 = f64::MIN;

        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        for chess_move in moves{
            if chess_move.is_null() {
                continue;
            }
            let result = self.search(&board_state.perform_move(chess_move), depth-1, alpha, beta);
            if result.0 >= max{
                max = result.0;
                max_move = chess_move;
            }
            if result.0 <= min{
                min = result.0;
                min_move = chess_move;
            }

            if(board_state.white_to_move()){
                if max > alpha {
                    alpha = max;
                }
            }else {
                if min < beta{
                    beta = min;
                }
            }
            if if board_state.white_to_move() {result.0 > beta} else {result.0 < alpha}{
                break;
            }
        }
        if board_state.white_to_move(){
            return (max, max_move);
        }else{
            return (min, min_move);
        }
    }
}


impl Clone for Bot2{
    fn clone(&self) -> Self {
        Self {  }
    }
}
impl Copy for Bot2{}