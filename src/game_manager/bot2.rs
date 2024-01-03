use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;
use graphics::color::WHITE;
use rand::Rng;

use super::board2::{GameState, DOUBLE_PAWN_MOVE, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, BLACK_EN_PASSANT, PROMOTE_TO_KNIGHT, PROMOTE_TO_BISHOP, PROMOTE_TO_ROOK, PROMOTE_TO_QUEEN, NO_FLAG};
use super::bot::GetMoveResult;
use super::move_gen::MoveGen;



pub struct Bot2{
    search_depth: i64,
    max_depth: usize,
    num_pos:usize,
    move_generator: MoveGen
}


impl Bot for Bot2{
    fn new() -> Self{
        Self{
            search_depth: 5,
            max_depth: 15,
            num_pos: 0,
            move_generator: MoveGen::new()
        }
    }

    fn get_move(&mut self, mut board_state:BoardState) -> GetMoveResult{
        let search_result = self.search(&mut board_state, self.search_depth, f64::MIN, f64::MAX, 0);
        return GetMoveResult::new(
            search_result.1,
            self.num_pos,
            search_result.0);
    }
}


impl Bot2 {

    fn promising_move(&self, board_state:&mut BoardState, chess_move: &ChessMove) -> f64{
        let mut promising_level = 0.0;

        let origin_value = board_state.piece_value(chess_move.origin());
        let target_value = board_state.piece_value(chess_move.target());

        let color_value = if origin_value < 0.0 {-1.0} else {1.0}; //Note that origin square is never 0

        match chess_move.flag(){
            NO_FLAG => {
                promising_level += (-target_value);
            }
            DOUBLE_PAWN_MOVE => {
                promising_level += 1.0*color_value;
            }
            W_CASTLE_KING | W_CASTLE_QUEEN => {
                promising_level += 1.0;
            }
            B_CASTLE_KING | B_CASTLE_QUEEN => {
                promising_level += -1.0;
            }
            WHITE_EN_PASSANT => {
                promising_level += 2.0;
            }
            BLACK_EN_PASSANT => {
                promising_level += -2.0;
            }
            PROMOTE_TO_KNIGHT => {
                promising_level += 3.0*color_value - target_value;
            }
            PROMOTE_TO_BISHOP => {
                promising_level += 3.0*color_value - target_value;
            }
            PROMOTE_TO_ROOK => {
                promising_level += 5.0*color_value - target_value;
            }
            PROMOTE_TO_QUEEN => {
                promising_level += 9.0*color_value - target_value;
            }
            _ => {println!("INVALID MOVE FLAG")}
        }

        return promising_level;
    }
    
    fn evaluate(&mut self, board_state:&mut BoardState) -> f64{
        self.move_generator.update_board(board_state);

        self.num_pos += 1;
        let mut rng = rand::thread_rng();
        let mut eval:f64 = 0.0;


        eval += rng.gen_range(-0.001..0.001);

        eval+=self.move_generator.piece_count();
        return eval;
    }

    fn search(&mut self, mut board_state:&mut BoardState, depth:i64, mut alpha:f64, mut beta:f64, true_depth:usize) -> (f64, ChessMove){

        let game_state = board_state.game_state();
        match game_state{
            GameState::Black => {return (f64::MIN, ChessMove::new_empty())}
            GameState::White => {return (f64::MAX, ChessMove::new_empty())}
            GameState::Draw => {return (0.0, ChessMove::new_empty())}
            GameState::Playing => {}
        }

        if depth <= 0 || true_depth >= self.max_depth{
            return (self.evaluate(board_state), ChessMove::new_empty());
        }

        let mut moves = self.move_generator.gen_moves_legal(board_state).moves_vec();

        
        let mut min:f64 = f64::MAX;
        let mut max:f64 = f64::MIN;

        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        if board_state.white_to_move() {
            moves.sort_by(|a, b| 
                self.promising_move(board_state, a)
                .partial_cmp(&self.promising_move(board_state, b))
                .unwrap().reverse()
                )
        }
        else {
            moves.sort_by(|a, b| 
                self.promising_move(board_state, a)
                .partial_cmp(&self.promising_move(board_state, b))
                .unwrap()
            )
        };
        for chess_move in moves{

            //Maybe maybe not
            let mut extension = 0;
            if board_state.piece_value(chess_move.target()) != 0.0 {
                extension = 0;
            }

            //Do the move
            let captured_piece = board_state.piece(chess_move.target() as usize);
            if captured_piece == 134 || captured_piece == 70 {
                println!("{}",self.num_pos);
            }
            

            

            let result = self.search(&mut board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1);


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
            if alpha >= beta{
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
        Self {  
            search_depth: self.search_depth,
            max_depth: self.max_depth,
            num_pos: self.num_pos,
            move_generator: MoveGen::new()
        }
    }
}
impl Copy for Bot2{}