use crate::client::game::engine::board;
/**
 * eval contains all functions meant to statically evaluate a function, mainly trough the function "evaluate"
 * all static evaluation should remain stateless
 * 
 */
use crate::client::game::engine::state_bitboard::{BISHOP, KNIGHT, PAWN, QUEEN, ROOK};

use super::board::{ChessMove, BLACK_EN_PASSANT, B_CASTLE_KING, B_CASTLE_QUEEN, WHITE_EN_PASSANT, W_CASTLE_KING, W_CASTLE_QUEEN};
use super::state_bitboard::bit_boards::{file_of, pop_lsb, rank_of, BOARD_CENTER, KING_PAWNS_OPTIMAL, NEIGHBOUR_FILES, RANKS, RANK_1, RANK_8, SEC_TIER_BISHOP, SEC_TIER_PAWN, TOP_TIER_BISHOP, TOP_TIER_PAWN};
use super::state_bitboard::{bit_boards, BitBoardState, BLACK, KING, WHITE};


fn is_check(bit_board_state:&BitBoardState, chess_move: &ChessMove) -> bool{
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

/**
 * PUBLIC FUNCTIONS
 */

pub fn capture_score(bit_board_state:&mut BitBoardState, capture: &ChessMove) -> i32{
    let origin_value = bit_board_state.piece_value(capture.origin()as usize).abs();
    let target_value = bit_board_state.piece_value(capture.target()as usize).abs();
    return target_value - origin_value/10;
}

pub fn is_capture(bit_board_state:&mut BitBoardState, m: &ChessMove) -> bool{
    match m.flag(){
        B_CASTLE_KING | B_CASTLE_QUEEN | W_CASTLE_KING | W_CASTLE_QUEEN => {return false;}//castle is never a capture
        WHITE_EN_PASSANT | BLACK_EN_PASSANT => {return true;}//en passant is allways a capture
        _ => {
            return bit_board_state.piece_value(m.target() as usize) != 0;
        }
    }
}

//attempts a very rough estimate on how good a move is
pub fn promising_move(bit_board_state:&mut BitBoardState, chess_move: &mut ChessMove, best_moves_option:Option<&Vec<(ChessMove, i32)>>){
    
    
    let mut promising_level = 0;
    
    let origin = chess_move.origin() as usize;
    let target = chess_move.target() as usize;
    
    let origin_value = bit_board_state.piece_value(origin);
    let target_value = bit_board_state.piece_value(target);
    
    let color_value = if origin_value < 0 {-1} else {1};


    if let Some(best_moves) = best_moves_option{
        for good_move in best_moves {
            if *chess_move == good_move.0 {
                let promising_level_ref = chess_move.promising_level_mut();
                *promising_level_ref = 3000*color_value as i16 + good_move.1 as i16;
                return;
            }
        }
    }

    


    //?I dont understand the unsused variable warnings here, TODO: fix later
    match chess_move.flag(){
        board::NO_FLAG => {
            if target_value != 0{ //is a capture
                /*
                capture of a rook will always come before capture of a knight,
                but capturing the rook with a pawn will come before capturing it with the queen
                
                
                 */
                promising_level += -target_value*10; //add value of captured piece
                promising_level -= origin_value //subtract value/10 of capturing piece
            }else{ // for non captures

                if origin_value == 10{
                    promising_level += pawn_placement_score(1 << target, WHITE);
                    promising_level -= pawn_placement_score(1 << origin, WHITE);
                }else if origin_value == -10{
                    promising_level += pawn_placement_score(1 << target, BLACK);
                    promising_level -= pawn_placement_score(1 << origin, BLACK);
                }else if origin_value == 30 || origin_value == -30{
                    promising_level += knight_placement_score(1 << target);
                    promising_level -= knight_placement_score(1 << origin);
                }
                else if origin_value == 35 || origin_value == -35{
                    promising_level += bishop_placement_score(1 << target, 0);
                    promising_level -= bishop_placement_score(1 << origin, 0);
                }
            }

        }
        board::DOUBLE_PAWN_MOVE => {
            promising_level += 10*color_value;
        }
        board::W_CASTLE_KING | board::W_CASTLE_QUEEN => {
            promising_level += 10;
        }
        board::B_CASTLE_KING | board::B_CASTLE_QUEEN => {
            promising_level += -10;
        }
        board::WHITE_EN_PASSANT => {
            promising_level += 20;
        }
        board::BLACK_EN_PASSANT => {
            promising_level += -20;
        }
        board::PROMOTE_TO_KNIGHT => {
            promising_level += 10*color_value - target_value;
        }
        board::PROMOTE_TO_BISHOP => {
            promising_level += 2*color_value - target_value;
        }
        board::PROMOTE_TO_ROOK => {
            promising_level += 2*color_value - target_value;
        }
        board::PROMOTE_TO_QUEEN => {
            promising_level += 90*color_value - target_value;
        }
        _ => {println!("INVALID MOVE FLAG")}
    }

    if is_check(bit_board_state, chess_move){
        promising_level+=1000*color_value;
    }

    let promising_level_ref = chess_move.promising_level_mut();
    *promising_level_ref = promising_level as i16;
}


pub fn evaluate(bit_board_state:&BitBoardState) -> i32{
    let pieces = bit_board_state.piece_bb();
    let piece_mask:u64 = bit_board_state.piece_mask();


    let endgame_factor = endgame_factor(&pieces);

    let mut eval:i32 = 0;
    //eval += fastrand::i32(-5..5);

    eval += dynamic_piece_count(&pieces[WHITE], &pieces[BLACK]) -
            dynamic_piece_count(&pieces[BLACK], &pieces[WHITE]);

    eval += (pawn_placement_score(pieces[WHITE][PAWN], WHITE) - 
            pawn_placement_score(pieces[BLACK][PAWN], BLACK))
            *3;
    
    eval += ((pawn_promotion_score(pieces[WHITE][PAWN], WHITE) - 
            pawn_promotion_score(pieces[BLACK][PAWN], BLACK))
            *endgame_factor)/10;

    eval += (pawn_structure_score(pieces[WHITE][PAWN]) -
            pawn_structure_score(pieces[BLACK][PAWN]))
            *35;

    eval += (knight_placement_score(pieces[WHITE][KNIGHT]) -
            knight_placement_score(pieces[BLACK][KNIGHT]))
            *5;

    eval += (bishop_placement_score(pieces[WHITE][BISHOP], WHITE) -
            bishop_placement_score(pieces[BLACK][BISHOP], BLACK))
            *15;

    eval += (rook_score(pieces[WHITE][ROOK], pieces[WHITE][PAWN], piece_mask) -
            rook_score(pieces[BLACK][ROOK], pieces[BLACK][PAWN], piece_mask)
            )*20;
    eval += (king_safety(&pieces[WHITE], &pieces[BLACK], WHITE) -
            king_safety(&pieces[BLACK], &pieces[WHITE], BLACK))
            *35;

    return eval;
}
