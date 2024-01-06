use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::SystemTime;

use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;

use super::board2::{GameState, DOUBLE_PAWN_MOVE, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, BLACK_EN_PASSANT, PROMOTE_TO_KNIGHT, PROMOTE_TO_BISHOP, PROMOTE_TO_ROOK, PROMOTE_TO_QUEEN, NO_FLAG};
use super::bot::GetMoveResult;
use super::state_bitboard::bit_boards::{TOP_TIER_PAWN, SEC_TIER_PAWN, TOP_TIER_BISHOP, SEC_TIER_BISHOP, rank_of, pop_lsb, RookMoves, file_of, self};
use super::state_bitboard::{BitBoardState, BoardStateNumbers, PAWN, WHITE, BLACK, BISHOP, ROOK, KING};

extern crate fxhash;
use fxhash::FxHasher;

const TABLE_SIZE:usize = 1000000;
const SEARCH_DEPTH:i64 = 6;
const MAX_DEPTH:usize = 16;

pub struct Bot2{
    search_depth: i64,
    max_depth: usize,
    num_pos: usize,
    table: HashMap<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>,
    start_time: SystemTime,
    average_best_move_index: f64,
    average_best_move_index_count: u64,
    best_line: [ChessMove; MAX_DEPTH]

}


impl Bot for Bot2{
    fn new() -> Self{
        Self{
            search_depth: SEARCH_DEPTH,
            max_depth: MAX_DEPTH,
            num_pos: 0,
            table: HashMap::<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>::default(),
            start_time: SystemTime::now(),
            average_best_move_index: 0.0,
            average_best_move_index_count: 0,
            best_line: [ChessMove::new_empty(); MAX_DEPTH]
            
        }
    }

    fn get_move(&mut self, board_state:BoardState) -> GetMoveResult{
        self.start_time = SystemTime::now();
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.board_setup(&board_state);
        let mut best_line = self.best_line.clone();

        let mut best_move:ChessMove = ChessMove::new_empty();
        let mut best_eval:i32 = 0;
        for i in 2..self.search_depth+1{
            self.num_pos = 0;
            let search_result = self.search(&mut bit_board_state, i, i32::MIN, i32::MAX, 0, true, &mut best_line);
            self.table.clear();
            best_move = search_result.1;
            best_eval = search_result.0;
            self.best_line = best_line.clone();
        }
        return GetMoveResult::new(
            best_move,
            self.num_pos,
            best_eval,
            self.average_best_move_index);
    }
}


impl Bot2 {


    fn is_check(&self, bit_board_state:&BitBoardState, chess_move: &ChessMove) -> bool{
        /*
        summed up, if we give the moving side an extra tempo, can it capture the king?
         */



        let origin = chess_move.origin() as usize;
        let target = chess_move.target() as usize;
        let piece_mask = bit_board_state.piece_mask();
        let mut psuedo_legal_follow_up_captures:u64 = 0;
        match bit_board_state.piece_value(origin){
            10 => {
                psuedo_legal_follow_up_captures = bit_boards::PAWN_CAPTURES[1][target]
            }
            -10 => {
                psuedo_legal_follow_up_captures = bit_boards::PAWN_CAPTURES[0][target]
            }
            30 | -30 => {
                psuedo_legal_follow_up_captures = bit_boards::KNIGHT_MOVES[target]
            }
            35 | -35 => {
                psuedo_legal_follow_up_captures = bit_boards::BishopMoves::mov_map(target, piece_mask);
            }
            50 | -50 => {
                psuedo_legal_follow_up_captures = bit_boards::RookMoves::mov_map(target, piece_mask);
            }
            90 | -90 => {
                psuedo_legal_follow_up_captures = bit_boards::RookMoves::mov_map(target, piece_mask);
                psuedo_legal_follow_up_captures = bit_boards::BishopMoves::mov_map(target, piece_mask);
            }
            _ => {}
        }
        let other = if bit_board_state.white_to_move() {0} else {1};
        if psuedo_legal_follow_up_captures & bit_board_state.piece_bb()[other][KING] != 0 {
            return true;
        }else{
            return false;
        }
    }
    fn promising_move(&self, bit_board_state:&mut BitBoardState, chess_move: &mut ChessMove, ply: usize){

        let mut promising_level = 0;
        
        let origin = chess_move.origin() as usize;
        let target = chess_move.target() as usize;
        
        let origin_value = bit_board_state.piece_value(origin);
        let target_value = bit_board_state.piece_value(target);
        
        let to_move = if bit_board_state.white_to_move() {1} else {0};
        let other = if to_move == 1 {0} else {1};
        let color_value = if origin_value < 0 {-1} else {1}; //Note that origin square is never no piece
        let other_value = if color_value == 1 {-1} else {1};

        if self.best_line[ply] == *chess_move{
            let promising_level_ref = chess_move.promising_level_mut();
            *promising_level_ref = 30000*color_value as i16;
            return;
        }

        match chess_move.flag(){
            NO_FLAG => {
                //let least_valuable_defender = bit_board_state.least_valuable_controller(chess_move.target() as usize, other)*other_value;
                promising_level += -target_value*10; //add value of captured piece

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

        if self.is_check(bit_board_state, chess_move){
            promising_level+=1000*color_value;
        }

        let promising_level_ref = chess_move.promising_level_mut();
        *promising_level_ref = promising_level as i16;
    }

    fn pawn_placement_score(pawns:u64, color:usize) -> i32{
        let mut score:i32 = 0;
        score += u64::count_ones(pawns & TOP_TIER_PAWN[color]) as i32 * 2;
        score += u64::count_ones(pawns & SEC_TIER_PAWN[color]) as i32;
        return score;
    }

    fn bishop_placement_score(bishops:u64, color:usize) -> i32{
        let mut score:i32 = 0;
        score += u64::count_ones(bishops & TOP_TIER_BISHOP[color]) as i32 * 2;
        score += u64::count_ones(bishops & SEC_TIER_BISHOP[color]) as i32;
        return score;
    }

    fn rook_score(rooks:u64, pawns:u64, blockers:u64) -> i32{
        let mut score:i32 = 0;
        let mut rooks_mut:u64 = rooks;
        let mut attacked_by_more_rooks:u64 = u64::MAX;
        while rooks_mut != 0{
            let rook = pop_lsb(&mut rooks_mut);
            if u64::count_ones(rooks) == 2{
                attacked_by_more_rooks &= file_of(rook) | rank_of(rook);
            }else{
                attacked_by_more_rooks = 0;
            }
            if file_of(rook) & pawns == 0 {
                score+=1;
            }

        }

        if u64::count_ones(attacked_by_more_rooks) > 2 { //Rooks share either a file or rank
            score += 1;
        }

        return score;
    }

    fn evaluate(&mut self, bit_board_state:&mut BitBoardState) -> i32{
        self.num_pos += 1;

        

        let pieces = bit_board_state.piece_bb();
        let piece_mask:u64 = bit_board_state.piece_mask();
        let to_move = if bit_board_state.white_to_move() {1} else {0};
        let other = if to_move == 1 {0} else {1};

        let mut eval:i32 = 0;
        //eval += fastrand::i32(-5..5);

        eval+=bit_board_state.piece_count()*10;


        eval+=((bit_board_state.knights_in_center(1)-bit_board_state.knights_in_center(0))) * 30; // ! move this function outside of bit_board_state

        eval += (Bot2::pawn_placement_score(pieces[WHITE][PAWN], WHITE) - 
                Bot2::pawn_placement_score(pieces[BLACK][PAWN], BLACK))
                *3;
        eval += (Bot2::bishop_placement_score(pieces[WHITE][BISHOP], WHITE) -
                Bot2::bishop_placement_score(pieces[BLACK][BISHOP], BLACK))
                *15;
        eval += (Bot2::rook_score(pieces[WHITE][ROOK], pieces[WHITE][PAWN], piece_mask) -
                Bot2::rook_score(pieces[BLACK][ROOK], pieces[BLACK][PAWN], piece_mask)
                )*8;


        return eval;
    }

    fn search(&mut self, mut bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, first: bool, best_line:& mut [ChessMove; MAX_DEPTH]) -> (i32, ChessMove){

        if !first { //No transpositions are possible for the first move
            if let Some(eval) = self.table.get(&(bit_board_state.board_state_numbers(), true_depth)){
                return (*eval, ChessMove::new_empty());
            }
        }



        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return (-1000000, ChessMove::new_empty())}
            GameState::White => {return (1000000, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }

        if depth <= 0 || true_depth >= self.max_depth{
            return (self.evaluate(bit_board_state), ChessMove::new_empty());
        }

        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;

        for i in 0..moves.len(){
            self.promising_move(bit_board_state, &mut moves[i], true_depth);
        }


        if bit_board_state.white_to_move() {
            moves.sort_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
                .reverse()
                )
        }
        else {
            moves.sort_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
                )
        };

        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        let mut move_index = 0;
        let mut best_move_index = 0;

        let mut best_line_clone = best_line.clone();

        for chess_move in moves{

            //Maybe maybe not
            let mut extension = 0;

            if move_index > 20{
                extension -=1;
            }

            if bit_board_state.piece_value(chess_move.target() as usize) != 0 && depth == 1{
                extension = std::cmp::max(1, extension);
            }

            

            
            let result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, &mut best_line_clone);


            if result.0 >= max{
                max = result.0;
                max_move = chess_move;
                best_move_index = move_index;
                *best_line = best_line_clone; // set up all follow up moves
                best_line[true_depth] = chess_move;

            }
            if result.0 <= min{
                min = result.0;
                min_move = chess_move;
                best_move_index = move_index;
                *best_line = best_line_clone; // set up all follow up moves
                best_line[true_depth] = chess_move;

            }

            if(bit_board_state.white_to_move()){
                if max > alpha {
                    alpha = max;
                }
                if max == 1000000{//if white has found forced mate
                    break;
                }
            }else {
                if min < beta{
                    beta = min;
                }
                if min == -1000000{//if black has found forced mate
                    break;
                }
            }
            if alpha > beta{
                //Killer move! opponent does not want to see this move be played
                break;
            }

            /*if self.start_time.elapsed().unwrap().as_millis() > 1000{
                break;
            }*/

            move_index += 1;
        }
        self.average_best_move_index_count += 1;
        self.average_best_move_index += (best_move_index as f64 - self.average_best_move_index)/self.average_best_move_index_count as f64;

        let eval = if bit_board_state.white_to_move() {max} else {min};

        if depth >= 1{
            self.table.insert((bit_board_state.board_state_numbers(), true_depth), eval);
            self.table.shrink_to(TABLE_SIZE);
        }

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
            table: HashMap::<(BoardStateNumbers, usize), i32, BuildHasherDefault<FxHasher>>::default(),
            start_time: SystemTime::now(),
            average_best_move_index: 0.0,
            average_best_move_index_count: 0,
            best_line: [ChessMove::new_empty(); MAX_DEPTH]
        }
    }
}
//impl Copy for Bot2{}