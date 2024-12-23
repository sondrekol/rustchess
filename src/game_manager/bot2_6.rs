use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::SystemTime;

use crate::game_manager::board2::{BoardState, ChessMove};
use crate::game_manager::bot::Bot;
use crate::game_manager::state_bitboard::QUEEN;

use super::board2::{GameState, DOUBLE_PAWN_MOVE, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, BLACK_EN_PASSANT, PROMOTE_TO_KNIGHT, PROMOTE_TO_BISHOP, PROMOTE_TO_ROOK, PROMOTE_TO_QUEEN, NO_FLAG};
use super::bot::GetMoveResult;
use super::state_bitboard::bit_boards::{TOP_TIER_PAWN, SEC_TIER_PAWN, TOP_TIER_BISHOP, SEC_TIER_BISHOP, rank_of, pop_lsb, file_of, self, BOARD_CENTER, NEIGHBOUR_FILES, RANKS, RANK_1, KING_PAWNS_OPTIMAL, RANK_8};
use super::state_bitboard::{BitBoardState, BoardStateNumbers, PAWN, WHITE, BLACK, BISHOP, ROOK, KING, KNIGHT};

extern crate fxhash;
use fxhash::FxHasher;

const DEFAULT_TABLE_SIZE:usize = 1000000;
const DEFAULT_SEARCH_DEPTH:i64 = 6;
const DEFAULT_MAX_DEPTH:usize = 20;
const DEFAULT_MAX_TIME:Option<u128> = None;

pub struct Bot2_6{
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


impl Bot for Bot2_6{
    fn default() -> Self{
        return Bot2_6::new(DEFAULT_SEARCH_DEPTH, DEFAULT_MAX_DEPTH, DEFAULT_TABLE_SIZE, DEFAULT_MAX_TIME);
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
            best_move = search_result.1;
            if search_result.0 < 30000 && search_result.0 > -30000 {//if depth stopped before calculating the evaluation of the best move, use the previous
                best_eval = search_result.0;
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

impl Bot2_6 {

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
                psuedo_legal_follow_up_captures += bit_boards::BishopMoves::mov_map(target, piece_mask);
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

    fn promising_move(&self, bit_board_state:&mut BitBoardState, chess_move: &mut ChessMove, best_moves_option:Option<&Vec<(ChessMove, i32)>>){

        let mut promising_level = 0;
        
        let origin = chess_move.origin() as usize;
        let target = chess_move.target() as usize;
        
        let origin_value = bit_board_state.piece_value(origin);
        let target_value = bit_board_state.piece_value(target);
        
        //let to_move = if bit_board_state.white_to_move() {1} else {0}; //?commented because of warning
        //let other = if to_move == 1 {0} else {1}; //?commented because of warning
        let color_value = if origin_value < 0 {-1} else {1}; //Note that origin square is never no piece
        //let other_value = if color_value == 1 {-1} else {1}; //?commented because of warning

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
                        promising_level += Bot2_6::pawn_placement_score(1 << target, WHITE);
                        promising_level -= Bot2_6::pawn_placement_score(1 << origin, WHITE);
                    }else if origin_value == -10{
                        promising_level += Bot2_6::pawn_placement_score(1 << target, BLACK);
                        promising_level -= Bot2_6::pawn_placement_score(1 << origin, BLACK);
                    }else if origin_value == 30 || origin_value == -30{
                        promising_level += Bot2_6::knight_placement_score(1 << target);
                        promising_level -= Bot2_6::knight_placement_score(1 << origin);
                    }
                    else if origin_value == 35 || origin_value == -35{
                        promising_level += Bot2_6::bishop_placement_score(1 << target, 0);
                        promising_level -= Bot2_6::bishop_placement_score(1 << origin, 0);
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

    fn pawn_structure_score(pawns:u64) -> i32{
        let mut pawn_structure_score:i32 = 0;
        
        for i in 0..8{

            if file_of(i) & pawns == 0 {//no pawns on file, no penalty
                continue;
            }else if NEIGHBOUR_FILES[i] & pawns == 0 {//no neighbour pawns -> isolated pawn -> penalty
                pawn_structure_score -= 1;
            }
            let pawns_on_file = u64::count_ones(file_of(i) & pawns);
            pawn_structure_score += 1 - pawns_on_file as i32;
        }


        return pawn_structure_score;
    }

    fn pawn_promotion_score(pawns: u64, color: usize) -> i32{

        // ! does not take into account if a pawn is passed or not.
        let mut pawn_promotion_score = 0;
        let dir = -1 + 2*color as i32;//-1 for black, 1 for white 
        pawn_promotion_score += u64::count_ones(RANKS[((dir*3).rem_euclid(8)) as usize] & pawns) as i32 * 1;
        pawn_promotion_score += u64::count_ones(RANKS[((dir*4).rem_euclid(8)) as usize] & pawns) as i32 * 2;
        pawn_promotion_score += u64::count_ones(RANKS[((dir*5).rem_euclid(8)) as usize] & pawns) as i32 * 3;
        pawn_promotion_score += u64::count_ones(RANKS[((dir*6).rem_euclid(8)) as usize] & pawns) as i32 * 4;
        pawn_promotion_score += u64::count_ones(RANKS[((dir*7).rem_euclid(8)) as usize] & pawns) as i32 * 5;

        return pawn_promotion_score;
    }

    fn bishop_placement_score(bishops:u64, color:usize) -> i32{
        let mut score:i32 = 0;
        score += u64::count_ones(bishops & TOP_TIER_BISHOP[color]) as i32 * 2;
        score += u64::count_ones(bishops & SEC_TIER_BISHOP[color]) as i32;
        return score;
    }

    fn rook_score(rooks:u64, pawns:u64, _blockers:u64) -> i32{

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

    //a slightly less static way of counting material
    //knights are worth more in closed position
    //bishops are worth more in open positions
    //one rook is worth 500, two are 900
    fn dynamic_piece_count(pieces:&[u64; 6], other_pieces:&[u64; 6]) -> i32{
        let mut piece_count:i32 = 0;

        //default value for pieces
        const VALUE_PAWN:i32 = 100;
        const VALUE_KNIGHT:i32 = 300;
        const VALUE_BISHOP:i32 = 340;
        const VALUES_ROOK:[i32; 10] = [500, 900, 1300, 1700, 2100, 2500, 2900, 3300, 3600, 3900];
        const VALUE_QUEEN:i32 = 900;

        //number of piece type for self color
        let num_pawns = u64::count_ones(pieces[PAWN]) as i32;
        let num_knights = u64::count_ones(pieces[KNIGHT]) as i32;
        let num_bishops = u64::count_ones(pieces[BISHOP]) as i32;
        let num_rooks = u64::count_ones(pieces[ROOK]) as i32;
        let num_queens = u64::count_ones(pieces[QUEEN]) as i32;

        //number of total pawns, both black and white
        let total_pawns = u64::count_ones(other_pieces[PAWN])as i32+num_pawns;

        //pawns
        piece_count += num_pawns*VALUE_PAWN;


        //knights are worth more in a closed position(more pawns)
        piece_count += num_knights*(VALUE_KNIGHT+total_pawns);


        //bishops are worth more in an endgame(less pawns)
        piece_count += num_bishops*(VALUE_BISHOP-total_pawns);


        //rooks: rooks decrease in value when having more, two are worth as much as a queen
        piece_count += VALUES_ROOK[num_rooks as usize];

        
        //queens
        piece_count += num_queens*VALUE_QUEEN;

        return piece_count;
    }

    //returns a king safety score
    //this score is only a penalty
    fn king_safety(pieces:&[u64; 6], _other_pieces:&[u64; 6], color:usize) -> i32{
        let mut king_safety_score:i32 = 0;

        let king_pos = u64::trailing_zeros(pieces[KING]) as usize;

        // !nasty code
        //? find a better way of achieving this, right now this does not really play along with concept of open files

        if rank_of(king_pos) == RANK_1 && color == WHITE{
            king_safety_score += (u64::count_ones(KING_PAWNS_OPTIMAL[WHITE][king_pos] & pieces[PAWN])*2) as i32;
            king_safety_score += u64::count_ones((KING_PAWNS_OPTIMAL[WHITE][king_pos] << 8) & pieces[PAWN]) as i32;
        }else if rank_of(king_pos) == RANK_8 && color == BLACK{
            king_safety_score += (u64::count_ones(KING_PAWNS_OPTIMAL[BLACK][king_pos - 56] & pieces[PAWN])*2) as i32;
            king_safety_score += u64::count_ones((KING_PAWNS_OPTIMAL[BLACK][king_pos - 56] >> 8) & pieces[PAWN]) as i32;
        }

        //TODO: penalty for enemy pieces attacking nearby squares

        //TODO: penalty for pawn storm, if enemy pawns are close. Note that this penalty should be strictly less than that for a open file near the king

        

        return king_safety_score;
    }

    //0 -> all pieces are on the board
    //256 -> all pieces are on the board
    fn endgame_factor(pieces:&[[u64; 6]; 2]) -> i32{

        let num_pawns = u64::count_ones(pieces[WHITE][PAWN] | pieces[BLACK][PAWN]) as i32;
        let num_knights = u64::count_ones(pieces[WHITE][KNIGHT] | pieces[BLACK][KNIGHT]) as i32;
        let num_bishops = u64::count_ones(pieces[WHITE][BISHOP] | pieces[BLACK][BISHOP]) as i32;
        let num_rooks = u64::count_ones(pieces[WHITE][ROOK] | pieces[BLACK][ROOK]) as i32;
        let num_queens = u64::count_ones(pieces[WHITE][QUEEN] | pieces[BLACK][QUEEN]) as i32;

        const PAWN_PHASE:i32 = 0;
        const KNIGHT_PHASE:i32 = 1;
        const BISHOP_PHASE:i32 = 1;
        const ROOK_PHASE:i32 = 2;
        const QUEEN_PHASE:i32 = 4;

        let mut phase = PAWN_PHASE*16 + KNIGHT_PHASE*4 + BISHOP_PHASE*4 + ROOK_PHASE*4 + QUEEN_PHASE*4;
        phase -= num_pawns*PAWN_PHASE;
        phase -= num_knights*KNIGHT_PHASE;
        phase -= num_bishops*BISHOP_PHASE;
        phase -= num_rooks*ROOK_PHASE;
        phase -= num_queens*QUEEN_PHASE;

        return phase;
    }

    fn evaluate(&mut self, bit_board_state:&BitBoardState) -> i32{
        self.num_pos += 1;

        //calculate end game factor(but how??)

        let pieces = bit_board_state.piece_bb();
        let piece_mask:u64 = bit_board_state.piece_mask();


        let endgame_factor = Bot2_6::endgame_factor(&pieces);

        let mut eval:i32 = 0;
        //eval += fastrand::i32(-5..5);

        eval += Bot2_6::dynamic_piece_count(&pieces[WHITE], &pieces[BLACK]) -
                Bot2_6::dynamic_piece_count(&pieces[BLACK], &pieces[WHITE]);

        eval += (Bot2_6::pawn_placement_score(pieces[WHITE][PAWN], WHITE) - 
                Bot2_6::pawn_placement_score(pieces[BLACK][PAWN], BLACK))
                *3;
        
        eval += ((Bot2_6::pawn_promotion_score(pieces[WHITE][PAWN], WHITE) - 
                Bot2_6::pawn_promotion_score(pieces[BLACK][PAWN], BLACK))
                *endgame_factor)/10;

        eval += (Bot2_6::pawn_structure_score(pieces[WHITE][PAWN]) -
                Bot2_6::pawn_structure_score(pieces[BLACK][PAWN]))
                *35;

        eval += (Bot2_6::knight_placement_score(pieces[WHITE][KNIGHT]) -
                Bot2_6::knight_placement_score(pieces[BLACK][KNIGHT]))
                *5;

        eval += (Bot2_6::bishop_placement_score(pieces[WHITE][BISHOP], WHITE) -
                Bot2_6::bishop_placement_score(pieces[BLACK][BISHOP], BLACK))
                *15;

        eval += (Bot2_6::rook_score(pieces[WHITE][ROOK], pieces[WHITE][PAWN], piece_mask) -
                Bot2_6::rook_score(pieces[BLACK][ROOK], pieces[BLACK][PAWN], piece_mask)
                )*20;
        eval += (Bot2_6::king_safety(&pieces[WHITE], &pieces[BLACK], WHITE) -
                Bot2_6::king_safety(&pieces[BLACK], &pieces[WHITE], BLACK))
                *35;

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

    //finishes the search by looking at any captures in a position, and subsequent "capture-backs" on the same square
    //all nodes are evaluated, a node is evaluated as the min/max of its children and itself (works on the assumption that there is a non capturing move)
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
            Bot2_6::is_capture(bit_board_state, m)
        });
        //moves should only contain captures at this point


        //after initial capture, only check if can capture back, dont check any potential "danger level captures"
        if let Some(capture_square) = opt_capture_square{
            moves.retain(|m|{
                m.target() == capture_square
            });
        }
        let this_eval = self.evaluate(bit_board_state);
        //if there are no more captures available, return the piece count
        if moves.len() == 0 {
            return this_eval;
        }

        moves.sort_by(|a, b| 
            self.capture_score(bit_board_state, a)
            .cmp(&self.capture_score(bit_board_state, b))
            .reverse()
            );

        //at worst either player can choose to not capture
        let mut min = this_eval;
        let mut max = this_eval;


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

    fn search(&mut self, bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, _first: bool, match_history:&mut Vec<BoardStateNumbers>) -> (i32, ChessMove){



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
            return (self.capture_search(bit_board_state, alpha, beta, 0, None), ChessMove::new_empty());
        }
        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        

        let previous_best_moves = self.table.get(&board_state_numbers);

        //add promising level to the moves for later sorting
        for i in 0..moves.len(){
            self.promising_move(bit_board_state, &mut moves[i], previous_best_moves);
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
            let extension = 0;


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


impl Clone for Bot2_6{
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
//impl Copy for Bot2_6{}