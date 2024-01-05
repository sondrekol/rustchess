use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;

use super::board2::{GameState, DOUBLE_PAWN_MOVE, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, BLACK_EN_PASSANT, PROMOTE_TO_KNIGHT, PROMOTE_TO_BISHOP, PROMOTE_TO_ROOK, PROMOTE_TO_QUEEN, NO_FLAG};
use super::bot::GetMoveResult;
use super::state_bitboard::{BitBoardState, BoardStateNumbers};

extern crate fxhash;
use fxhash::FxHashMap;
use fxhash::FxBuildHasher;
use fxhash::FxHasher;
const TABLE_SIZE:usize = 1000000;



pub struct Bot2{
    search_depth: i64,
    max_depth: usize,
    num_pos: usize,
    table: HashMap<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>
}


impl Bot for Bot2{
    fn new() -> Self{
        Self{
            search_depth: 6,
            max_depth: 20,
            num_pos: 0,
            table: HashMap::<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>::default()
        }
    }

    fn get_move(&mut self, mut board_state:BoardState) -> GetMoveResult{
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.board_setup(&board_state);
        let search_result = self.search(&mut bit_board_state, self.search_depth, i32::MIN, i32::MAX, 0, true);
        return GetMoveResult::new(
            search_result.1,
            self.num_pos,
            search_result.0);
    }
}


impl Bot2 {

    fn promising_move(&self, bit_board_state:&mut BitBoardState, chess_move: &ChessMove) -> i32{
        let mut promising_level = 0;

        let origin_value = bit_board_state.piece_value(chess_move.origin() as usize);
        let target_value = bit_board_state.piece_value(chess_move.target() as usize);

        let color_value = if origin_value < 0 {-1} else {1}; //Note that origin square is never no piece

        match chess_move.flag(){
            NO_FLAG => {
                promising_level += -target_value*10;
            }
            DOUBLE_PAWN_MOVE => {
                promising_level += 10*color_value;
            }
            W_CASTLE_KING | W_CASTLE_QUEEN => {
                promising_level += 10;
            }
            B_CASTLE_KING | B_CASTLE_QUEEN => {
                promising_level += -10;
            }
            WHITE_EN_PASSANT => {
                promising_level += 20;
            }
            BLACK_EN_PASSANT => {
                promising_level += -20;
            }
            PROMOTE_TO_KNIGHT => {
                promising_level += 4*color_value - target_value;
            }
            PROMOTE_TO_BISHOP => {
                promising_level += 2*color_value - target_value;
            }
            PROMOTE_TO_ROOK => {
                promising_level += 2*color_value - target_value;
            }
            PROMOTE_TO_QUEEN => {
                promising_level += 90*color_value - target_value;
            }
            _ => {println!("INVALID MOVE FLAG")}
        }

        return promising_level;
    }
    
    fn evaluate(&mut self, bit_board_state:&mut BitBoardState) -> i32{

        self.num_pos += 1;
        let mut eval:i32 = 0;


        eval += fastrand::i32(-5..5);

        let to_move = if bit_board_state.white_to_move() {1} else {0};
        let other = if to_move == 1 {0} else {1};

        eval+=((bit_board_state.knights_in_center(1)-bit_board_state.knights_in_center(0))) * 30;

        eval+=bit_board_state.piece_count()*100;


        return eval;
    }

    fn search(&mut self, mut bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, first: bool) -> (i32, ChessMove){

        if !first { //No transpositions are possible for the first move
            if let Some(eval) = self.table.get(&(bit_board_state.board_state_numbers(), true_depth)){
                return (*eval, ChessMove::new_empty());
            }
        }



        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return (i32::MIN, ChessMove::new_empty())}
            GameState::White => {return (i32::MAX, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }

        if depth <= 0 || true_depth >= self.max_depth{
            return (self.evaluate(bit_board_state), ChessMove::new_empty());
        }

        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;


        if bit_board_state.white_to_move() {
            moves.sort_by(|a, b| 
                self.promising_move(bit_board_state, a)
                .partial_cmp(&self.promising_move(bit_board_state, b))
                .unwrap().reverse()
                )
        }
        else {
            moves.sort_by(|a, b| 
                self.promising_move(bit_board_state, a)
                .partial_cmp(&self.promising_move(bit_board_state, b))
                .unwrap()
            )
        };

        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        for chess_move in moves{

            //Maybe maybe not
            let mut extension = 0;

            if bit_board_state.piece_value(chess_move.target() as usize) != 0 && depth == 1{
                extension = std::cmp::max(1, extension);
            }

            //check captured piece //?Implement later if needed
            /*let captured_piece = bit_board_state.piece(chess_move.target() as usize);
            if captured_piece == 134 || captured_piece == 70 {
                println!("{}",self.num_pos);
            }
            */
            

            

            let result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false);


            if result.0 >= max{
                max = result.0;
                max_move = chess_move;
            }
            if result.0 <= min{
                min = result.0;
                min_move = chess_move;
            }

            if(bit_board_state.white_to_move()){
                if max > alpha {
                    alpha = max;
                }
            }else {
                if min < beta{
                    beta = min;
                }
            }
            if alpha > beta{
                break;
            }

        }

        let eval = if bit_board_state.white_to_move() {max} else {min};

        self.table.insert((bit_board_state.board_state_numbers(), true_depth), eval);
        self.table.shrink_to(TABLE_SIZE);

        if bit_board_state.white_to_move(){
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
            table: HashMap::<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>::default()
        }
    }
}
//impl Copy for Bot2{}