use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::SystemTime;

use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;

use super::board2::{GameState, DOUBLE_PAWN_MOVE, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, BLACK_EN_PASSANT, PROMOTE_TO_KNIGHT, PROMOTE_TO_BISHOP, PROMOTE_TO_ROOK, PROMOTE_TO_QUEEN, NO_FLAG};
use super::bot::GetMoveResult;
use super::state_bitboard::bit_boards::{TOP_TIER_PAWN, SEC_TIER_PAWN, TOP_TIER_BISHOP, SEC_TIER_BISHOP, rank_of, pop_lsb, RookMoves, file_of, self, BOARD_CENTER};
use super::state_bitboard::{BitBoardState, BoardStateNumbers, PAWN, WHITE, BLACK, BISHOP, ROOK, KING, KNIGHT};

extern crate fxhash;
use fxhash::FxHasher;

const DEFAULT_TABLE_SIZE:usize = 1000000;
const DEFAULT_SEARCH_DEPTH:i64 = 7;
const DEFAULT_MAX_DEPTH:usize = 8;

pub struct Bot2{
    search_depth: i64,
    max_depth: usize,
    num_pos: usize,
    table: HashMap<BoardStateNumbers, ChessMove, BuildHasherDefault<FxHasher>>,
    table_size: usize,
    start_time: SystemTime,
    average_best_move_placement: f64,
    average_best_move_index_placement: u64,
    search_stopped: bool


}


impl Bot for Bot2{
    fn default() -> Self{
        return Bot2::new(DEFAULT_SEARCH_DEPTH, DEFAULT_MAX_DEPTH, DEFAULT_TABLE_SIZE);
    }

    fn new(search_depth: i64, max_depth: usize, table_size: usize) -> Self{
        Self{
            search_depth: search_depth,
            max_depth: max_depth,
            num_pos: 0,
            table: HashMap::<BoardStateNumbers, ChessMove, BuildHasherDefault<FxHasher>>::default(),
            table_size: table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false
            
        }
    }

    fn get_move(&mut self, board_state:BoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult{
        self.start_time = SystemTime::now();
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.board_setup(&board_state);

        return self.get_move_bb(bit_board_state, match_history);
    }
    fn get_move_bb(&mut self, board_state:BitBoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult{
        self.start_time = SystemTime::now();

        let mut bit_board_state = board_state;
        let mut best_move:ChessMove = ChessMove::new_empty();
        let mut best_eval:i32 = 0;
        for i in 2..self.search_depth+1{
            self.num_pos = 0;
            let search_result = self.search(&mut bit_board_state, i, i32::MIN, i32::MAX, 0, true, match_history);
            //self.table.clear();
            if self.search_stopped {
                println!("stopped at depth: {i}");
                break;
            }
            best_move = search_result.1;
            best_eval = search_result.0;
        }

        return GetMoveResult::new(
            best_move,
            self.num_pos,
            best_eval,
            self.average_best_move_placement);
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
    fn promising_move(&self, bit_board_state:&mut BitBoardState, chess_move: &mut ChessMove, ply: usize, previous_best:Option<&ChessMove>){

        let mut promising_level = 0;
        
        let origin = chess_move.origin() as usize;
        let target = chess_move.target() as usize;
        
        let origin_value = bit_board_state.piece_value(origin);
        let target_value = bit_board_state.piece_value(target);
        
        let to_move = if bit_board_state.white_to_move() {1} else {0};
        let other = if to_move == 1 {0} else {1};
        let color_value = if origin_value < 0 {-1} else {1}; //Note that origin square is never no piece
        let other_value = if color_value == 1 {-1} else {1};

        /*if self.best_line[ply] == *chess_move{
            let promising_level_ref = chess_move.promising_level_mut();
            *promising_level_ref = 30000*color_value as i16;
            return;
        }*/

        //If there is allready a calculated best move for this position, one should probaly search that first
        if let Some(best_move) = previous_best{
            if *chess_move == *best_move {
                let promising_level_ref = chess_move.promising_level_mut();
                *promising_level_ref = 30000*color_value as i16;
                return;
            }
        }

        match chess_move.flag(){
            NO_FLAG => {
                if target_value != 0{ //is a capture
                    /*
                    capture of a rook will always come before capture of a knight,
                    but capturing the rook with a pawn will come before capturing it with the queen
                    
                    
                     */
                    promising_level += -target_value*10; //add value of captured piece
                    promising_level -= origin_value //subtract value/10 of capturing piece
                }else{ // for non captures

                    if origin_value == 10{
                        promising_level += Bot2::pawn_placement_score(1 << target, WHITE);
                        promising_level -= Bot2::pawn_placement_score(1 << origin, WHITE);
                    }else if origin_value == -10{
                        promising_level += Bot2::pawn_placement_score(1 << target, BLACK);
                        promising_level -= Bot2::pawn_placement_score(1 << origin, BLACK);
                    }else if origin_value == 30 || origin_value == -30{
                        promising_level += Bot2::knight_placement_score(1 << target);
                        promising_level -= Bot2::knight_placement_score(1 << origin);
                    }
                    else if origin_value == 35 || origin_value == -35{
                        promising_level += Bot2::bishop_placement_score(1 << target, 0);
                        promising_level -= Bot2::bishop_placement_score(1 << origin, 0);
                    }
                }

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
                promising_level += 10*color_value - target_value;
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

    fn knight_placement_score(knights:u64) -> i32{
        return u64::count_ones(knights & BOARD_CENTER) as i32;
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


        eval += (Bot2::pawn_placement_score(pieces[WHITE][PAWN], WHITE) - 
                Bot2::pawn_placement_score(pieces[BLACK][PAWN], BLACK))
                *3;
        eval += (Bot2::knight_placement_score(pieces[WHITE][KNIGHT]) -
                Bot2::knight_placement_score(pieces[BLACK][KNIGHT]))
                *5;
        eval += (Bot2::bishop_placement_score(pieces[WHITE][BISHOP], WHITE) -
                Bot2::bishop_placement_score(pieces[BLACK][BISHOP], BLACK))
                *15;
        eval += (Bot2::rook_score(pieces[WHITE][ROOK], pieces[WHITE][PAWN], piece_mask) -
                Bot2::rook_score(pieces[BLACK][ROOK], pieces[BLACK][PAWN], piece_mask)
                )*20;


        return eval;
    }

    fn search(&mut self, mut bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, first: bool, match_history:&mut Vec<BoardStateNumbers>) -> (i32, ChessMove){




        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return (-1000000, ChessMove::new_empty())}
            GameState::White => {return (1000000, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }

        if match_history.iter().filter(|&n| *n == bit_board_state.board_state_numbers()).count() == 2{
            return (0, ChessMove::new_empty()); 
        }
        match_history.push(bit_board_state.board_state_numbers());
        
        if depth <= 0 || true_depth >= self.max_depth{
            match_history.pop();
            return (self.evaluate(bit_board_state), ChessMove::new_empty());
        }
        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;

        let previous_best = self.table.get(&bit_board_state.board_state_numbers());

        for i in 0..moves.len(){
            self.promising_move(bit_board_state, &mut moves[i], true_depth, previous_best);
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

        let mut move_placement = 0;
        let mut best_move_placement: f64 = 0.0;

        let move_count = moves.len() as f64;
        for chess_move in moves{

            //Maybe maybe not
            let mut extension = 0;
            let mut lazy = false;

            if (move_placement as f64)/move_count > 0.3 && move_count > 9.0 && depth > 3{
                extension -=1;
                lazy = true;

            }
            if (move_placement as f64)/move_count > 0.6 && move_count > 9.0 && depth > 3{
                extension -=1;
                lazy = true;

            }

            if bit_board_state.piece_value(chess_move.target() as usize) != 0 && depth == 1{
                extension = std::cmp::max(1, extension);
            }

            

            
            let mut result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history);

            if lazy && result.0 >= max{
                extension = 0;
                if bit_board_state.piece_value(chess_move.target() as usize) != 0 && depth == 1{
                    extension = std::cmp::max(1, extension);
                }
                result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history);
            }

            if result.0 >= max{
                
                if !(result.0 == 0 && max > -30){//dont go for draw in a roughly equal position
                    max = result.0;
                    max_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                }
            }

            if lazy && result.0 <= min{
                extension = 0;
                if bit_board_state.piece_value(chess_move.target() as usize) != 0 && depth == 1{
                    extension = std::cmp::max(1, extension);
                }
                result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history);
            }
            
            if result.0 <= min{
                if !(result.0 == 0 && min < 30){//dont go for draw in a roughly equal position
                    min = result.0;
                    min_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                }

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

            if self.start_time.elapsed().unwrap().as_millis() > 2000{
                self.search_stopped = true;
                break;
            }

            move_placement += 1;
        }
        self.average_best_move_index_placement += 1;
        self.average_best_move_placement += (best_move_placement as f64 - self.average_best_move_placement)/self.average_best_move_index_placement as f64;



        match_history.pop();
        if bit_board_state.white_to_move(){
            if depth >= 1{
                if self.table.len() < self.table_size {
                    self.table.insert(bit_board_state.board_state_numbers(), max_move);
                }
            }
            return (max, max_move);
        }else{
            if depth >= 1{
                if self.table.len() < self.table_size {
                    self.table.insert(bit_board_state.board_state_numbers(), min_move);
                }
            }
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
            table: HashMap::<BoardStateNumbers, ChessMove, BuildHasherDefault<FxHasher>>::default(),
            table_size: self.table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false
        }
    }
}
//impl Copy for Bot2{}