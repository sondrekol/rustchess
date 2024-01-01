
//?

use std::arch::x86_64;



//Piece codes
const PIECE_PAWN: u8 = 0b00000001;
const PIECE_KNIGHT: u8 = 0b00000010;
const PIECE_BISHOP: u8 = 0b00000011;
const PIECE_ROOK: u8 = 0b00000100;
const PIECE_QUEEN: u8 = 0b00000101;
const PIECE_KING: u8 = 0b00000110;

const PIECE_WHITE: u8 = 0b10000000;
const PIECE_BLACK: u8 = 0b01000000;

const PIECE_TYPE_MASK: u8 = 0b00000111;
const PIECE_COLOR_MASK: u8 = 0b11000000;

//Castle Rights
const WHITE_CASTLE_KING: u8 = 0b00000001;
const WHITE_CASTLE_QUEEN: u8 = 0b00000010;
const BLACK_CASTLE_KING: u8 = 0b00000100;
const BLACK_CASTLE_QUEEN: u8 = 0b00001000;

//To move define
const WHITE_TO_MOVE: bool = true;
const BLACK_TO_MOVE: bool = false;

//en passant
const NO_EN_PASSANT_SQUARE:u8 = 0x80;

pub struct BoardState{
    pieces: [[u8;8];8], 
    castle_rights: u8, 
    to_move: bool,
    en_passant_square: u8,
    half_move_clock: usize,
    turns: usize

}


impl BoardState{
    pub fn new_from_fen(fen:&str) -> Self{
        let mut pieces:[[u8;8];8] = [[0;8];8];
        let mut to_move:bool = false;
        let mut en_passant_square:u8 = NO_EN_PASSANT_SQUARE;
        let mut castle_rights:u8 = 0x00;
        let mut half_move_clock: usize = 0;
        let mut turns:usize = 0;


        let mut index:i32 = -1;
        let mut state:usize = 0;
        for c in fen.chars(){

            //Piece part
            if state == 0{
                let mut to_add:u8 = 0;
                match c{
                    'k' => {to_add = PIECE_KING | PIECE_BLACK; index+=1;}
                    'q' => {to_add = PIECE_QUEEN | PIECE_BLACK; index+=1;}
                    'r' => {to_add = PIECE_ROOK | PIECE_BLACK; index+=1;}
                    'b' => {to_add = PIECE_BISHOP | PIECE_BLACK; index+=1;}
                    'n' => {to_add = PIECE_KNIGHT | PIECE_BLACK; index+=1;}
                    'p' => {to_add = PIECE_PAWN | PIECE_BLACK; index+=1;}
                    'K' => {to_add = PIECE_KING | PIECE_WHITE; index+=1;}
                    'Q' => {to_add = PIECE_QUEEN | PIECE_WHITE; index+=1;}
                    'R' => {to_add = PIECE_ROOK | PIECE_WHITE; index+=1;}
                    'B' => {to_add = PIECE_BISHOP | PIECE_WHITE; index+=1;}
                    'N' => {to_add = PIECE_KNIGHT | PIECE_WHITE; index+=1;}
                    'P' => {to_add = PIECE_PAWN | PIECE_WHITE; index+=1;}
                    '1' => {index+=1}
                    '2' => {index+=2}
                    '3' => {index+=3}
                    '4' => {index+=4}
                    '5' => {index+=5}
                    '6' => {index+=6}
                    '7' => {index+=7}
                    '8' => {index+=8}
                    '\\' => {continue}
                    ' ' => {state+=1; continue}
                    _ => {continue}
    
                }
                pieces[(index as usize)/8][(index as usize)%8] = to_add;
            }
            //to move
            else if state == 1{
                match c{
                    ' ' => {state+=1; continue}
                    'w' => {to_move = WHITE_TO_MOVE;}
                    'b' => {to_move = BLACK_TO_MOVE;}
                    _ => {continue}
                }
            }
            //castling rights
            else if state == 2{
                match c{
                    'q' => {castle_rights |= BLACK_CASTLE_QUEEN}
                    'k' => {castle_rights |= BLACK_CASTLE_KING}
                    'Q' => {castle_rights |= WHITE_CASTLE_QUEEN}
                    'K' => {castle_rights |= WHITE_CASTLE_KING}
                    ' ' => {state+=1; continue}
                    _ => {continue}
                }
            }
            //en passant square
            else if state == 3{
                match c{
                    'a' => {en_passant_square += 0}
                    'b' => {en_passant_square += 8}
                    'c' => {en_passant_square += 16}
                    'd' => {en_passant_square += 24}
                    'e' => {en_passant_square += 32}
                    'f' => {en_passant_square += 40}
                    'g' => {en_passant_square += 48}
                    'h' => {en_passant_square += 56}
                    '1' => {en_passant_square += 7}
                    '2' => {en_passant_square += 6}
                    '3' => {en_passant_square += 5}
                    '4' => {en_passant_square += 4}
                    '5' => {en_passant_square += 3}
                    '6' => {en_passant_square += 2}
                    '7' => {en_passant_square += 1}
                    '8' => {en_passant_square += 0}
                    '-' => {en_passant_square = 0x10}
                    ' ' => {state+=1; continue}
                    _ => {continue}
                }
            }

            //TODO handle halfmove clock
            else if state == 4{

            }
            //TODO handle turns
            else if state == 5{

            }


        }
        Self{
            pieces: pieces,
            castle_rights: castle_rights,
            to_move: to_move,
            en_passant_square:en_passant_square,
            half_move_clock:half_move_clock,
            turns:turns
        }
    }
    pub fn get_board(&self) -> [[i8; 8]; 8]{
        let mut result: [[i8; 8]; 8] = [[0; 8]; 8];
        for i in 0..8{
            for j in 0..8{
                let source_piece = self.pieces[i][j];

                result[i][j] =
                    if source_piece == PIECE_PAWN | PIECE_WHITE {1}
                    else if source_piece == PIECE_KNIGHT | PIECE_WHITE {2}
                    else if source_piece == PIECE_BISHOP | PIECE_WHITE {3}
                    else if source_piece == PIECE_ROOK | PIECE_WHITE {4}
                    else if source_piece == PIECE_QUEEN | PIECE_WHITE {5}
                    else if source_piece == PIECE_KING | PIECE_WHITE {6}
                    else if source_piece == PIECE_PAWN | PIECE_BLACK {-1}
                    else if source_piece == PIECE_KNIGHT | PIECE_BLACK {-2}
                    else if source_piece == PIECE_BISHOP | PIECE_BLACK {-3}
                    else if source_piece == PIECE_ROOK | PIECE_BLACK {-4}
                    else if source_piece == PIECE_QUEEN | PIECE_BLACK {-5}
                    else if source_piece == PIECE_KING | PIECE_BLACK {-6}
                    else {0};
            }
        }
        return result;
    }
}


//?Maybe delete
pub struct ChessMove{
    origin_square: u8,
    target_square: u8,
    flag: u8,
}

impl ChessMove{
    pub fn from_coordinates(origin:(u8, u8), target:(u8, u8)) -> Self{
        Self{
            origin_square: origin.0*8+origin.1,
            target_square: target.0*8+target.1,
            flag: 0 //TODO
        }
    }
}

fn king_position(board_state:&BoardState, color: bool) -> (usize, usize){
    let this_color = if color == true {PIECE_WHITE} else {PIECE_BLACK};
    let mut king_pos:(usize, usize) = (0, 0);

    for i in 0..8{
        for j in 0..8{
            if board_state.pieces[i][j] == this_color | PIECE_KING{
                king_pos = (i, j);
            }
        }
    }
    return king_pos;
}

fn is_attacked(board_state:&BoardState, color: bool, target_square: (usize, usize)) -> bool{

    let pawn_direction:i8 = if color == true {-1} else {1};

    let opposite_color = if color == true {PIECE_BLACK} else {PIECE_WHITE};
    let this_color = if color == true {PIECE_WHITE} else {PIECE_BLACK};

    //check pawn
    for i in [target_square.1-1, target_square.1+1]{
        let x = (target_square.0 as i8+pawn_direction) as usize;
        if x > 8 || i > 8 {
            continue;
        }
        if board_state.pieces[x][i] == opposite_color | PIECE_PAWN{
            return true;
        }
    }
    //check diagonal
    for dir_x in [-1, 1]{
        for dir_y in [-1, 1]{
            for i in 1..8{
                let x = target_square.0 as i8+dir_x*i;
                let y = target_square.1 as i8+dir_y*i;
                if x < 0 || x >= 8 || y < 0 || y >= 8 {
                    break;
                }
                if board_state.pieces[x as usize][y as usize] != 0{
                    if 
                        board_state.pieces[x as usize][y as usize] == opposite_color | PIECE_BISHOP ||
                        board_state.pieces[x as usize][y as usize] == opposite_color | PIECE_QUEEN 
                    {
                        return true;
                    }
                    break;
                }
            }
        }
    }
    //check rook
    for (dir_x, dir_y) in [
        (1, 0),(-1, 0),
        (0, 1),(0, -1)]
    {
        for i in 1..8{

            let x = target_square.0 as i8+dir_x*i;
            let y = target_square.1 as i8+dir_y*i;
            if x < 0 || x >= 8 || y < 0 || y >= 8 {
                break;
            }
            if 
                board_state.pieces[x as usize][y as usize] != 0
            {

                if 
                    board_state.pieces[x as usize][y as usize] == opposite_color | PIECE_ROOK || 
                    board_state.pieces[x as usize][y as usize] == opposite_color | PIECE_QUEEN
                {
                    return true;
                }else{
                    break;
                }
            }
        }
    }

    //check knight
    for (dir_x, dir_y) in [
        (1, 2),(1, -2),
        (-1, 2),(-1, -2),
        (2, 1),(2, -1),
        (-2, 1),(-2, -1)]
    {
        let x = target_square.0 as i8+dir_x;
        let y = target_square.1 as i8+dir_y;
        if x < 0 || x >= 8 || y < 0 || y >= 8 {
            continue;
        }
        if board_state.pieces[x as usize][y as usize] == opposite_color | PIECE_KNIGHT{
            return true;
        }
    }
    return false;
}

fn check_pawn_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize), double_pawn_move:&mut bool, promote:&mut bool)->bool{


    if(target.0 == 0 && board_state.to_move==WHITE_TO_MOVE || target.0 == 7 && board_state.to_move==BLACK_TO_MOVE){
        *promote = true;
    }

    //Normal move
    if target.0 == if board_state.to_move {origin.0-1} else {origin.0+1} && target.1 == origin.1{
        if board_state.pieces[target.0][target.1] != 0{
            return false;
        }
        return true;
    }

    

    //Capture move
    if target.0 == if board_state.to_move {origin.0-1} else {origin.0+1} && (target.1 == origin.1+1 || target.1 == origin.1-1){
        if board_state.to_move && board_state.pieces[target.0][target.1] & PIECE_COLOR_MASK == PIECE_BLACK{
            return true;
        }else if !board_state.to_move && board_state.pieces[target.0][target.1] & PIECE_COLOR_MASK == PIECE_WHITE{
            return true;
        }
        if board_state.en_passant_square != NO_EN_PASSANT_SQUARE {
            let en_pas_square = ((board_state.en_passant_square>>4) as usize, (board_state.en_passant_square%8) as usize);
            if (board_state.en_passant_square>>4) as usize == target.0 && (board_state.en_passant_square%8) as usize == target.1{
                board_state.pieces[if board_state.to_move == WHITE_TO_MOVE {3} else {4}][(board_state.en_passant_square%8) as usize] = 0;
                board_state.en_passant_square = NO_EN_PASSANT_SQUARE;
                return true;
            }
        }
    }

    //Double move
    if board_state.to_move == WHITE_TO_MOVE && target.1 == origin.1{
        if target.0 == 4 && origin.0 == 6{
            if board_state.pieces[5][target.1] != 0{
                return false;
            }
            if board_state.pieces[4][target.1] != 0{
                return false;
            }
            board_state.en_passant_square = 0x50 | target.1 as u8;
            *double_pawn_move = true;
            return true;
        }
    }else if board_state.to_move == BLACK_TO_MOVE && target.1 == origin.1{
        if target.0 == 3 && origin.0 == 1 {
            if board_state.pieces[2][target.1] != 0{
                return false;
            }
            if board_state.pieces[3][target.1] != 0{
                return false;
            }
            board_state.en_passant_square = 0x20 | target.1 as u8;
            *double_pawn_move = true;
            return true;
        }
    }
    //TODO: promotion

    return false;
}

fn check_knight_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize))->bool{
    let length_x = (origin.0 as i8-target.0 as i8).abs();
    let length_y = (origin.1 as i8-target.1 as i8).abs();

    if(length_x == 2 && length_y == 1)||(length_x == 1 && length_y == 2){
        return true;
    }
    return false;
}

fn check_bishop_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize))->bool{
    
    let mut direction_x = target.0 as i8 - origin.0 as i8;
    let mut direction_y = target.1 as i8 - origin.1 as i8;

    //check that move is diagonal
    if direction_x.abs() != direction_y.abs() {
        return false;
    }


    //check that move is not blocked
    let num_steps = direction_x.abs();
    direction_x = if direction_x < 0 {-1} else {1};
    direction_y = if direction_y < 0 {-1} else {1};
    for i in 1..num_steps{
        let x_index = ((origin.0 as i8)+(direction_x*i)) as usize;
        let y_index = ((origin.1 as i8)+(direction_y*i)) as usize;
        if board_state.pieces[x_index][y_index] != 0{
            return false;
        }
    }


    return true;
}

fn check_rook_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize))->bool{
    if origin.0 == target.0 {
        let smallest = std::cmp::min(origin.1, target.1);
        let biggest = std::cmp::max(origin.1, target.1);
        for i in smallest+1..biggest{
            if board_state.pieces[origin.0][i] != 0{
                return false;
            }
        }
        return true;
    } else if origin.1 == target.1 {
        let smallest = std::cmp::min(origin.0, target.0);
        let biggest = std::cmp::max(origin.0, target.0);
        for i in smallest+1..biggest{
            if board_state.pieces[i][origin.1] != 0{
                return false;
            }
        }
        return true;
    }
    return false;
}

fn check_queen_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize))->bool{
    if check_rook_move(board_state, origin, target) || check_bishop_move(board_state, origin, target) {
        return true;
    }
    return false;
}



fn check_king_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize))->bool{
    //Attempt white kingside Castle //!DRY
    if(board_state.to_move == WHITE_TO_MOVE){
        if origin.0 == 7 && origin.1 == 4{
            if target.0 == 7 && target.1 == 6 {
                if board_state.castle_rights & WHITE_CASTLE_KING != 0{
                    if board_state.pieces[7][5] == 0 && board_state.pieces[7][6] == 0{
                        board_state.pieces[7][7] = 0;
                        board_state.pieces[7][5] = PIECE_WHITE | PIECE_ROOK;

                        board_state.castle_rights &= !(WHITE_CASTLE_KING | WHITE_CASTLE_QUEEN);
                        return true;
                    }
                }
            }
            if target.0 == 7 && target.1 == 2 {
                if board_state.castle_rights & WHITE_CASTLE_QUEEN != 0{
                    if board_state.pieces[7][1] == 0 && board_state.pieces[7][2] == 0 && board_state.pieces[7][3] == 0{
                        board_state.pieces[7][0] = 0;
                        board_state.pieces[7][3] = PIECE_WHITE | PIECE_ROOK;

                        board_state.castle_rights &= !(WHITE_CASTLE_KING | WHITE_CASTLE_QUEEN);
                        return true;
                    }
                }
            }
        }
    }
    if(board_state.to_move == BLACK_TO_MOVE){
        if origin.0 == 0 && origin.1 == 4{
            if target.0 == 0 && target.1 == 6 {
                if board_state.castle_rights & BLACK_CASTLE_KING != 0{
                    if board_state.pieces[0][5] == 0 && board_state.pieces[0][6] == 0{
                        board_state.pieces[0][7] = 0;
                        board_state.pieces[0][5] = PIECE_BLACK | PIECE_ROOK;

                        board_state.castle_rights &= !(BLACK_CASTLE_KING | BLACK_CASTLE_QUEEN);
                        return true;
                    }
                }
            }
            if target.0 == 0 && target.1 == 2 {
                if board_state.castle_rights & BLACK_CASTLE_QUEEN != 0{
                    if board_state.pieces[0][1] == 0 && board_state.pieces[0][2] == 0 && board_state.pieces[0][3] == 0{
                        board_state.pieces[0][0] = 0;
                        board_state.pieces[0][3] = PIECE_WHITE | PIECE_ROOK;

                        board_state.castle_rights &= !(WHITE_CASTLE_KING | WHITE_CASTLE_QUEEN);
                        return true;
                    }
                }
            }
        }
    }

    if (origin.0 as i8 - target.0 as i8).abs() > 1{
        return false;
    }
    if (origin.1 as i8 - target.1 as i8).abs() > 1{
        return false;
    }
    


    return true;
}


pub enum PerformMoveResult{
    Success,
    MoveOutOfBounds,
    InvalidMove,
    ResultsInCheck,
    NoPieceOnOrigin,
    WrongColor,
    NoPromotionPiece,

}

pub fn perform_move(board_state:&mut BoardState, origin:(usize, usize), target:(usize, usize), promote_to: Option<u8>) -> PerformMoveResult{

    let mut double_pawn_move: bool = false;
    let mut promote: bool = false;

    let mut backup = BoardState{
        pieces: board_state.pieces,
        castle_rights: board_state.castle_rights,
        to_move: board_state.to_move,
        en_passant_square: board_state.en_passant_square,
        half_move_clock: board_state.half_move_clock,
        turns: board_state.turns,
        
    };
    //check that all coordinates are valid
    if
        origin.0 >= 8 &&
        origin.1 >= 8 &&
        target.0 >= 8 &&
        target.1 >= 8
    {
        return PerformMoveResult::MoveOutOfBounds;
    }
    
    //Assert that correct colored piece is being moved
    if
        board_state.to_move == WHITE_TO_MOVE && board_state.pieces[origin.0][origin.1] & PIECE_BLACK != 0 ||
        board_state.to_move == BLACK_TO_MOVE && board_state.pieces[origin.0][origin.1] & PIECE_WHITE != 0
    {
        return PerformMoveResult::WrongColor;
    }

    //Assert that same colored piece is not captured
    if
        board_state.to_move == WHITE_TO_MOVE && board_state.pieces[target.0][target.1] & PIECE_WHITE != 0 ||
        board_state.to_move == BLACK_TO_MOVE && board_state.pieces[target.0][target.1] & PIECE_BLACK != 0
    {
        return PerformMoveResult::WrongColor;
    }

    match board_state.pieces[origin.0][ origin.1] & PIECE_TYPE_MASK{
        PIECE_PAWN => {
            if !check_pawn_move(board_state, origin, target, &mut double_pawn_move, &mut promote){
                return PerformMoveResult::InvalidMove;
            }
        }
        PIECE_KNIGHT => {
            if !check_knight_move(board_state, origin, target){
                return PerformMoveResult::InvalidMove;
            }
        }
        PIECE_BISHOP => {
            if !check_bishop_move(board_state, origin, target){
                return PerformMoveResult::InvalidMove;
            }
        }
        PIECE_ROOK => {
            if !check_rook_move(board_state, origin, target){
                return PerformMoveResult::InvalidMove;
            }
        }
        PIECE_QUEEN => {
            if !check_queen_move(board_state, origin, target){
                return PerformMoveResult::InvalidMove;
            }
        }
        PIECE_KING => {
            if !check_king_move(board_state, origin, target){
                return PerformMoveResult::InvalidMove;
            }
        }
        _ => {
            return PerformMoveResult::NoPieceOnOrigin;
        }
    }

    
    

    
    board_state.pieces[target.0][target.1] = board_state.pieces[origin.0][origin.1];
    board_state.pieces[origin.0][origin.1] = 0;


    if promote {
        if let Some(promotionPiece) = promote_to{
            board_state.pieces[target.0][target.1] = (board_state.pieces[target.0][target.1] & 0xF8) | promotionPiece & 0x07;
        }else{
            board_state.pieces = backup.pieces;
            board_state.castle_rights = backup.castle_rights;
            board_state.en_passant_square = backup.en_passant_square;
            board_state.to_move = backup.to_move;
            board_state.half_move_clock = backup.half_move_clock;
            board_state.turns = backup.turns;
            return PerformMoveResult::NoPromotionPiece;
        }
    }

    

    if is_attacked(board_state, board_state.to_move, king_position(board_state, board_state.to_move)){
        board_state.pieces = backup.pieces;
        board_state.castle_rights = backup.castle_rights;
        board_state.en_passant_square = backup.en_passant_square;
        board_state.to_move = backup.to_move;
        board_state.half_move_clock = backup.half_move_clock;
        board_state.turns = backup.turns;
        return PerformMoveResult::ResultsInCheck;
    }

    /*s
    Move is legal from this point
     */

    //Disable castling
    if origin == (7,7) || target == (7,7){
        board_state.castle_rights &= !WHITE_CASTLE_KING
    }
    if origin == (7,0) || target == (7,0){
        board_state.castle_rights &= !WHITE_CASTLE_QUEEN
    }
    if origin == (0,7) || target == (0,7){
        board_state.castle_rights &= !BLACK_CASTLE_KING
    }
    if origin == (0,0) || target == (0,0){
        board_state.castle_rights &= !BLACK_CASTLE_QUEEN
    }

    //Disable en_passant_square, exception being if the en_passant_square was set on this move
    if !double_pawn_move {
        board_state.en_passant_square = NO_EN_PASSANT_SQUARE;
    }

    board_state.to_move = !board_state.to_move;

    return PerformMoveResult::Success;

}


