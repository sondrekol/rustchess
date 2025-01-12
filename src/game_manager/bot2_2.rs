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
const DEFAULT_SEARCH_DEPTH:i64 = 6;
const DEFAULT_MAX_DEPTH:usize = 20;
const DEFAULT_MAX_TIME:Option<u128> = None;

pub struct Bot2_2{
    search_depth: i64,
    max_depth: usize,
    num_pos: usize,
    table: HashMap<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>,
    table_size: usize,
    start_time: SystemTime,
    average_best_move_placement: f64,
    average_best_move_index_placement: u64,
    search_stopped: bool,
    max_time: Option<u128>


}


impl Bot for Bot2_2{
    fn default() -> Self{
        return Bot2_2::new(DEFAULT_SEARCH_DEPTH, DEFAULT_MAX_DEPTH, DEFAULT_TABLE_SIZE, DEFAULT_MAX_TIME);
    }

    fn new(search_depth: i64, max_depth: usize, table_size: usize, max_time: Option<u128>) -> Self{
        Self{
            search_depth: search_depth,
            max_depth: max_depth,
            num_pos: 0,
            table: HashMap::<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>::default(),
            table_size: table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false,
            max_time: max_time
            
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
        let mut depth = 0;
        for i in 2..self.search_depth+1{
            depth = i as u32;
            self.num_pos = 0;
            let search_result = self.search(&mut bit_board_state, i, i32::MIN, i32::MAX, 0, true, match_history);
            //self.table.clear();
            if self.search_stopped {
                break;
            }
            if best_move != search_result.1{
                best_move = search_result.1;
                if best_eval < 30000 && best_eval > -30000 {
                    best_eval = search_result.0;
                }
            }
        }

        return GetMoveResult::new(
            best_move,
            self.num_pos,
            best_eval,
            self.average_best_move_placement,
            depth
        );
    }
}


impl Bot2_2 {


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
                psuedo_legal_follow_up_captures |= bit_boards::BishopMoves::mov_map(target, piece_mask);
            }
            _ => {}
        }
        let other = if bit_board_state.white_to_move() {0} else {1};
        return psuedo_legal_follow_up_captures & bit_board_state.piece_bb()[other][KING] != 0;
    }

    fn promising_move(&self, bit_board_state:&mut BitBoardState, chess_move: &mut ChessMove, ply: usize, best_moves_option:Option<&Vec<(ChessMove, i32)>>){

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

        if let Some(best_moves) = best_moves_option{
            for good_move in best_moves {
                if *chess_move == good_move.0 {
                    let promising_level_ref = chess_move.promising_level_mut();
                    *promising_level_ref = 3000*color_value as i16 + good_move.1 as i16;
                    return;
                }
            }
        }
        //If there is allready a calculated best move for this position, one should probaly search that first
        /*if let Some(best_move) = previous_best{
            if *chess_move == *best_move {
                let promising_level_ref = chess_move.promising_level_mut();
                *promising_level_ref = 30000*color_value as i16;
                return;
            }
        }*/

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
                        promising_level += Bot2_2::pawn_placement_score(1 << target, WHITE);
                        promising_level -= Bot2_2::pawn_placement_score(1 << origin, WHITE);
                    }else if origin_value == -10{
                        promising_level += Bot2_2::pawn_placement_score(1 << target, BLACK);
                        promising_level -= Bot2_2::pawn_placement_score(1 << origin, BLACK);
                    }else if origin_value == 30 || origin_value == -30{
                        promising_level += Bot2_2::knight_placement_score(1 << target);
                        promising_level -= Bot2_2::knight_placement_score(1 << origin);
                    }
                    else if origin_value == 35 || origin_value == -35{
                        promising_level += Bot2_2::bishop_placement_score(1 << target, 0);
                        promising_level -= Bot2_2::bishop_placement_score(1 << origin, 0);
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

        //eval += bit_board_state.piece_count()*10;
        eval += self.capture_search(bit_board_state, i32::MIN, i32::MAX, 0, None)*10;


        eval += (Bot2_2::pawn_placement_score(pieces[WHITE][PAWN], WHITE) - 
                Bot2_2::pawn_placement_score(pieces[BLACK][PAWN], BLACK))
                *3;
        eval += (Bot2_2::knight_placement_score(pieces[WHITE][KNIGHT]) -
                Bot2_2::knight_placement_score(pieces[BLACK][KNIGHT]))
                *5;
        eval += (Bot2_2::bishop_placement_score(pieces[WHITE][BISHOP], WHITE) -
                Bot2_2::bishop_placement_score(pieces[BLACK][BISHOP], BLACK))
                *15;
        eval += (Bot2_2::rook_score(pieces[WHITE][ROOK], pieces[WHITE][PAWN], piece_mask) -
                Bot2_2::rook_score(pieces[BLACK][ROOK], pieces[BLACK][PAWN], piece_mask)
                )*20;


        return eval;
    }


    fn capture_score(&self, bit_board_state:&mut BitBoardState, capture: &ChessMove) -> i32{
        let origin_value = bit_board_state.piece_value(capture.origin()as usize).abs();
        let target_value = bit_board_state.piece_value(capture.target()as usize).abs();
        return target_value - origin_value/10;
    }

    fn is_capture(bit_board_state:&mut BitBoardState, m: &ChessMove) -> bool{
        match m.flag(){
            B_CASTLE_KING | B_CASTLE_QUEEN | W_CASTLE_KING | W_CASTLE_QUEEN => {return false;}//castle is never a capture
            WHITE_EN_PASSANT | BLACK_EN_PASSANT => {return true;}//en passant is allways a capture
            _ => {
                return bit_board_state.piece_value(m.target() as usize) != 0;
            }
        }
    }

    //returns the piece count after a series of best captures
    //for now very basic implementation
    fn capture_search(&mut self, bit_board_state:&mut BitBoardState, mut alpha:i32, mut beta:i32, capture_depth:usize, opt_capture_square:Option<u8>) -> i32{

        //Not directly related to piece count but should work
        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return -1000}
            GameState::White => {return 1000}
            GameState::Draw => {return 0}
            GameState::Playing => {}
        }

        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        moves.retain(|m|{
            Bot2_2::is_capture(bit_board_state, m)
        });
        //moves should only contain captures at this point


        //after initial capture, only check if can capture back, dont check any potential "danger level captures"
        if let Some(capture_square) = opt_capture_square{
            moves.retain(|m|{
                m.target() == capture_square
            });
        }

        //if there are no more captures available, return the piece count
        if moves.len() == 0 {
            return bit_board_state.piece_count();
        }

        moves.sort_by(|a, b| 
            self.capture_score(bit_board_state, a)
            .cmp(&self.capture_score(bit_board_state, b))
            .reverse()
            );

        //at worst either player can choose to not capture
        let mut min = bit_board_state.piece_count();
        let mut max = bit_board_state.piece_count();


        for capture in moves{

            let mut result = self.capture_search(&mut bit_board_state.perform_move(capture), alpha, beta, capture_depth+1, Some(capture.target()));

            if result >= 900 {
                result -= 1;
            }
            if result <= -900{
                result += 1;
            }
            if result > max {
                max = result;
            }
            if result < min{
                min = result;
            }
            if max > alpha{
                alpha = max;
            }
            if min < beta{
                beta = min;
            }
            if alpha > beta{
                break;
            }
        }

        if bit_board_state.white_to_move(){
            return max;
        }else{
            return min;
        }
    }

    fn search(&mut self, mut bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, first: bool, match_history:&mut Vec<BoardStateNumbers>) -> (i32, ChessMove){




        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return (-10000, ChessMove::new_empty())}
            GameState::White => {return (10000, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }
        let board_state_numbers = bit_board_state.board_state_numbers();
        if match_history.iter().filter(|&n| *n == board_state_numbers).count() == 2{
            return (0, ChessMove::new_empty()); 
        }
        match_history.push(board_state_numbers);
        
        if depth <= 0 || true_depth >= self.max_depth{
            match_history.pop();
            return (self.evaluate(bit_board_state), ChessMove::new_empty());
        }
        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        

        let previous_best_moves = self.table.get(&board_state_numbers);

        //add promising level to the moves for later sorting
        for i in 0..moves.len(){
            self.promising_move(bit_board_state, &mut moves[i], true_depth, previous_best_moves);
        }

        self.table.insert(board_state_numbers, Vec::<(ChessMove, i32)>::new());
        
        //at this point previous_best_moves_mut should contain an empty vec

        //Sort moves by how promising they are
        if bit_board_state.white_to_move() {
            moves.sort_unstable_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
                .reverse()
                )
        }else {
            moves.sort_unstable_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
                )
        };
        
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;
        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        let mut move_placement = 0;
        let mut best_move_placement: f64 = 0.0;

        let move_count = moves.len() as f64;
        for chess_move in moves{

            //Maybe maybe not
            let mut extension = 0;


            /*if self.is_check(bit_board_state, &chess_move) && depth == 1{
                extension += 1;
            }*/

            

            
            let mut result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history);

            if let Some(max_time) = self.max_time{
                if self.start_time.elapsed().unwrap().as_millis() > max_time{
                    self.search_stopped = true;
                    break;
                }
            }

            if result.0 >= 1000 {
                result.0 -= 1;
            }else if result.0 <= -1000{
                result.0 += 1;
            }

            if result.0 >= max{
                
                if !(result.0 == 0 && max > -30){//dont go for draw in a roughly equal position
                    max = result.0;
                    max_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                    
                    //replace or add best move
                    let best_moves = self.table.get_mut(&board_state_numbers).unwrap();
                    let mut found_move: bool = false;
                    for i in 0..best_moves.len(){
                        if max_move == best_moves[i].0{
                            best_moves[i].1 = max;
                            found_move = true;
                            break;
                        }
                    }
                    if !found_move{
                        self.table.get_mut(&board_state_numbers).unwrap().push((max_move, max));
                    }
                }
                
            }
            if result.0 <= min{
                if !(result.0 == 0 && min < 30){//dont go for draw in a roughly equal position
                    min = result.0;
                    min_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;

                    //replace or add best move
                    let best_moves = self.table.get_mut(&board_state_numbers).unwrap();
                    let mut found_move: bool = false;
                    for i in 0..best_moves.len(){
                        if min_move == best_moves[i].0{
                            best_moves[i].1 = min;
                            found_move = true;
                            break;
                        }
                    }
                    if !found_move{
                        self.table.get_mut(&board_state_numbers).unwrap().push((min_move, min));
                    }
                }

            }

            if bit_board_state.white_to_move() {
                if max > alpha {
                    alpha = max;
                }
            }else {
                if min < beta{
                    beta = min;
                }
            }
            if alpha > beta{
                //Killer move! opponent does not want to see this move be played
                break;
            }

            move_placement += 1;
        }
        self.average_best_move_index_placement += 1;
        self.average_best_move_placement += (best_move_placement as f64 - self.average_best_move_placement)/self.average_best_move_index_placement as f64;



        match_history.pop();
        if bit_board_state.white_to_move(){
            return (max, max_move);
        }else{
            return (min, min_move);
        }
    }
}


impl Clone for Bot2_2{
    fn clone(&self) -> Self {
        Self {  
            search_depth: self.search_depth,
            max_depth: self.max_depth,
            num_pos: self.num_pos,
            table: HashMap::<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>::default(),
            table_size: self.table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false,
            max_time: self.max_time
        }
    }
}
//impl Copy for Bot2_2{}