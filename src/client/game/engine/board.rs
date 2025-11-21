
use std::{hash::{Hash, Hasher}, fmt};

use futures::stream::futures_unordered::Iter;

use super::move_string::lan_move;



//Piece codes




const PIECE_PAWN: u8 = 0b00000001;
const PIECE_KNIGHT: u8 = 0b00000010;
const PIECE_BISHOP: u8 = 0b00000011;
const PIECE_ROOK: u8 = 0b00000100;
const PIECE_QUEEN: u8 = 0b00000101;
const PIECE_KING: u8 = 0b00000110;


const PIECE_WHITE: u8 = 0b10000000;
const PIECE_BLACK: u8 = 0b01000000;

//Castle Rights
const WHITE_CAN_CASTLE_KING: u8 = 0b00000001;
const WHITE_CAN_CASTLE_QUEEN: u8 = 0b00000010;
const BLACK_CAN_CASTLE_KING: u8 = 0b00000100;
const BLACK_CAN_CASTLE_QUEEN: u8 = 0b00001000;

//To move define
const WHITE_TO_MOVE: bool = true;
const BLACK_TO_MOVE: bool = false;

//en passant
const NO_EN_PASSANT_SQUARE:u8 = 0x80;




//Move flags:
pub const PROMOTE_TO_KNIGHT:u8 = 0b0000;
pub const PROMOTE_TO_BISHOP:u8 = 0b0001;
pub const PROMOTE_TO_ROOK:u8 = 0b0010;
pub const PROMOTE_TO_QUEEN:u8 = 0b0011;

pub const W_CASTLE_KING:u8 = 0b0100;
pub const W_CASTLE_QUEEN:u8 = 0b0101;
pub const B_CASTLE_KING:u8 = 0b0110;
pub const B_CASTLE_QUEEN:u8 = 0b0111;

pub const WHITE_EN_PASSANT:u8 = 0b1000;
pub const BLACK_EN_PASSANT:u8 = 0b1001;

pub const DOUBLE_PAWN_MOVE:u8 = 0b1010;

pub const NO_FLAG:u8 = 0b1111;



#[derive(PartialEq)]
pub enum GameState{
    Playing,
    White,
    Black,
    Draw
}


pub struct ChessMove{
    move_data:u16,
    pub promising_level:i16
}


impl Hash for ChessMove{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.move_data.hash(state);
    }
}
pub struct ChessMoveList{
    size: u8,
    chess_moves: [ChessMove; 218]
}


pub struct BoardState{
    pieces: [u8; 64],
    /*
    Indexes:
    a1 = 0
    a2 = 1
    a3 = 2
    .
    .
    .
    h8 = 63
    */


    white_to_move: bool,
    en_passant_square: u8,
    //index of en passant square


    castle_rights: u8,
    half_move_clock: u8,
    is_in_check: Option<bool>,
    legal_moves: Option<ChessMoveList>,
    white_king: usize,
    black_king: usize

}



impl ChessMove{
    /*
    First 4 bits are flagss
    Next 6 bits are the index of the target square
    last 6 bits are the index of the origin square
     */
    pub fn move_data(&self) -> u16{
        return self.move_data;
    }


    
    pub fn from_uci(uci_move:&str, board_state:&BoardState) -> Self{
        match uci_move {
            "e1g1" => {
                if board_state.castle_rights & WHITE_CAN_CASTLE_KING != 0{
                    return Self::from_indices(W_CASTLE_KING, 0, 0);
                }},
            "e1c1" => {
                if board_state.castle_rights & WHITE_CAN_CASTLE_QUEEN != 0{
                    return Self::from_indices(W_CASTLE_QUEEN, 0, 0);
                }},
            "e8g8" => {
                if board_state.castle_rights & BLACK_CAN_CASTLE_KING != 0{
                    return Self::from_indices(B_CASTLE_KING, 0, 0);
                }},
            "e8c8" => {
                if board_state.castle_rights & BLACK_CAN_CASTLE_QUEEN != 0{
                    return Self::from_indices(B_CASTLE_QUEEN, 0, 0);
                }},
            _ => {
                
            }
        }

        let origin_file = (uci_move.chars().nth(0).unwrap() as u8) - ('a' as u8);
        let origin_rank = (uci_move.chars().nth(1).unwrap() as u8) - ('1' as u8);
        let target_file = (uci_move.chars().nth(2).unwrap() as u8) - ('a' as u8);
        let target_rank = (uci_move.chars().nth(3).unwrap() as u8) - ('1' as u8);
        let origin = origin_rank * 8 + origin_file;
        let target = target_rank * 8 + target_file;


        if uci_move.len() == 5{
            let promote_char = uci_move.chars().nth(4).unwrap();
            let promote_flag = match promote_char {
                'n' => PROMOTE_TO_KNIGHT,
                'b' => PROMOTE_TO_BISHOP,
                'r' => PROMOTE_TO_ROOK,
                'q' => PROMOTE_TO_QUEEN,
                _ => NO_FLAG
            };
            return Self::from_indices(promote_flag, origin, target);
        }

        if board_state.piece(origin as usize) == PIECE_PAWN {
            if BoardState::vertical_distance(origin, target) == 2 {
                return Self::from_indices(DOUBLE_PAWN_MOVE, origin, target);
            }
            else if target == board_state.en_passant_square {
                if board_state.white_to_move {
                    return Self::from_indices(WHITE_EN_PASSANT, origin, target);
                }
                else{
                    return Self::from_indices(BLACK_EN_PASSANT, origin, target);
                }
            }
        }
        return Self::from_indices(NO_FLAG, origin, target);

    }
    pub fn new_empty() -> Self{
        Self { 
            move_data: 0,
            promising_level: 0
        }
    }

    pub fn from_indices(flags: u8, origin:u8, target: u8) -> Self{
        Self { 
            move_data:  (((flags as u16) & 0x0F) << 12) | 
                        (((target as u16) & 0x3F) << 6) | 
                        (((origin as u16) & 0x3F)),
            promising_level: 0
        }
    }

    //Get index of origin square
    pub fn origin(&self) -> u8{
        return (self.move_data & 0x003F) as u8;
    }

    //Get index of target square
    pub fn target(&self) -> u8{
        return ((self.move_data & 0x0FC0) >> 6) as u8;
    }

    pub fn flag(&self) -> u8{
        return ((self.move_data & 0xF000) >> 12) as u8;
    }

    pub fn promising_level_mut(&mut self) -> &mut i16{
        return &mut self.promising_level;
    }

    pub fn promising_level(&self) -> &i16{
        return &self.promising_level;
    }

}

impl Clone for ChessMove{
    fn clone(&self) -> Self {
        Self { move_data: self.move_data , promising_level: self.promising_level}
    }
}

impl Copy for ChessMove{}

impl fmt::Debug for ChessMove{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChessMove")
        .field("move", &lan_move(*self))
        .field("flag", &self.flag())
        .finish()
    }
}

impl PartialEq for ChessMove {
    fn eq(&self, other: &Self) -> bool {
        self.move_data == other.move_data
    }
}
impl Eq for ChessMove {
}

impl ChessMoveList{
    pub fn new() -> Self{
        Self {
            size: 0, 
            chess_moves: [ChessMove{move_data: 0, promising_level: 0}; 218]
        }
    }
    pub fn add_no_alloc(&mut self, origin:u8, target:u8, flag:u8){
        self.chess_moves[self.size as usize].move_data = origin as u16 | ((target as u16) << 6) | ((flag as u16) << 12);
        self.size+= 1;
    }

    pub fn add(&mut self, chess_move:ChessMove) {
        self.chess_moves[self.size as usize] = chess_move;
        self.size+=1;
    }

    //NOTE: this does not work if the moves are mutated
    pub fn size(&self) -> usize{
        return self.size as usize;
    }

    pub fn get_mut(&self, index:usize) -> &ChessMove{
        return &self.chess_moves[index];
    }


    pub fn sort<F>(&mut self, score:F) 
    where 
    F: Fn(&ChessMove) -> i32
    {
        let n = self.size as usize;
        if n <= 1 { return; }

        // Insertion sort is good enough for now because of small size
        for i in 1..n {
            let key = self.chess_moves[i];
            let mut j = i;
            // Sort by promising_level descending (higher is better)
            while j > 0 && score(&self.chess_moves[j - 1]) < score(&key) {
                self.chess_moves[j] = self.chess_moves[j - 1];
                j -= 1;
            }
            self.chess_moves[j] = key;
        }
    }

    pub fn retain<F>(&self, pred:F) -> ChessMoveList
    where F: Fn(&ChessMove) -> bool,
    {
        let mut retained:ChessMoveList = ChessMoveList::new();

        for i in 0..self.size {
            if pred(&self.chess_moves[i as usize]) {
                retained.add(self.chess_moves[i as usize]);
            }
        }

        return retained;
    }

    
    pub fn moves_vec(&self) -> Vec<ChessMove>{
        let mut moves = Vec::<ChessMove>::with_capacity(218);
        for i in 0..self.size as usize{
            if self.chess_moves[i].move_data != 0{
                moves.push(self.chess_moves[i]);
            }
        }
        return moves;
    }

    pub fn reset(&mut self) {
        self.size = 0;
    }

}


impl Clone for ChessMoveList{
    fn clone(&self) -> Self {
        Self { size: self.size.clone(), chess_moves: self.chess_moves.clone() }
    }
}

impl Copy for ChessMoveList{}


impl BoardState{

    pub fn new_from_fen(fen:&str) -> Self{
        let mut pieces:[u8; 64] = [0; 64];
        let mut to_move:bool = false;
        let mut en_passant_square:u8 = NO_EN_PASSANT_SQUARE;
        let mut castle_rights:u8 = 0x00;


        let mut index:i32 = -1;
        let mut state:usize = 0;
        let mut white_king = 0;
        let mut black_king = 0;
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
                //TODO flip index horizontaly
                let rank = 7-index/8;
                let file = index%8;
                let piece_index = (rank*8 + file) as usize;
                pieces[piece_index] = to_add;
                if to_add == PIECE_BLACK | PIECE_KING {
                    black_king = piece_index;
                }
                else if to_add == PIECE_WHITE | PIECE_KING {
                    white_king = piece_index;
                }
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
                    'q' => {castle_rights |= BLACK_CAN_CASTLE_QUEEN}
                    'k' => {castle_rights |= BLACK_CAN_CASTLE_KING}
                    'Q' => {castle_rights |= WHITE_CAN_CASTLE_QUEEN}
                    'K' => {castle_rights |= WHITE_CAN_CASTLE_KING}
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
                    '1' => {en_passant_square += 0}
                    '2' => {en_passant_square += 1}
                    '3' => {en_passant_square += 2}
                    '4' => {en_passant_square += 3}
                    '5' => {en_passant_square += 4}
                    '6' => {en_passant_square += 5}
                    '7' => {en_passant_square += 6}
                    '8' => {en_passant_square += 7}
                    '-' => {en_passant_square = NO_EN_PASSANT_SQUARE}
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
        return Self { 
            pieces: pieces,
            white_to_move: to_move,
            en_passant_square: en_passant_square, 
            castle_rights: castle_rights,
            half_move_clock: 0,
            is_in_check: None,
            legal_moves: None,
            white_king: white_king,
            black_king: black_king
        }
    }

    pub fn piece(&self, index: usize) -> u8{
        return self.pieces[index];
    }

    pub fn castle_rights(&self) -> u8{
        return self.castle_rights;
    }

    pub const fn vertical_distance(a: u8, b:u8) -> u8{
        return ((a/8) as i8 - (b/8) as i8).abs() as u8;
    }

    pub fn perform_move(&mut self, chess_move:ChessMove){
        //self.move_history.push(chess_move);
        //self.piece_history.push(self.pieces);

        let flag = chess_move.flag();
        let origin = chess_move.origin();
        let target = chess_move.target();

        self.legal_moves = None;
        self.is_in_check = None;


        match flag {
            NO_FLAG => {
                self.pieces[target as usize] = self.pieces[origin as usize];
                self.pieces[origin as usize] = 0;
                if self.pieces[target as usize] == PIECE_WHITE | PIECE_KING{
                    self.white_king = target as usize;
                }else if self.pieces[target as usize] == PIECE_BLACK | PIECE_KING{
                    self.black_king = target as usize;
                }
            }
            DOUBLE_PAWN_MOVE => {
                self.pieces[target as usize] = self.pieces[origin as usize];
                self.pieces[origin as usize] = 0;
                self.en_passant_square = if self.white_to_move {origin+8} else {origin-8};
            }
            W_CASTLE_KING => {
                self.pieces[4] = 0;
                self.pieces[7] = 0;
                self.pieces[5] = PIECE_WHITE | PIECE_ROOK;
                self.pieces[6] = PIECE_WHITE | PIECE_KING;
                self.castle_rights &= 0xFC;
                self.white_king = 6;
            }
            W_CASTLE_QUEEN => {
                self.pieces[4] = 0;
                self.pieces[0] = 0;
                self.pieces[3] = PIECE_WHITE | PIECE_ROOK;
                self.pieces[2] = PIECE_WHITE | PIECE_KING;
                self.castle_rights &= 0xFC;
                self.white_king = 2;
            }
            B_CASTLE_KING => {
                self.pieces[60] = 0;
                self.pieces[63] = 0;
                self.pieces[61] = PIECE_BLACK | PIECE_ROOK;
                self.pieces[62] = PIECE_BLACK | PIECE_KING;
                self.castle_rights &= 0x03;
                self.black_king = 62;
            }
            B_CASTLE_QUEEN => {
                self.pieces[60] = 0;
                self.pieces[56] = 0;
                self.pieces[59] = PIECE_BLACK | PIECE_ROOK;
                self.pieces[58] = PIECE_BLACK | PIECE_KING;
                self.castle_rights &= 0x03;
                self.black_king = 58;
            }
            PROMOTE_TO_BISHOP => {
                self.pieces[origin as usize] = 0;
                self.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_BISHOP;
            }
            PROMOTE_TO_KNIGHT => {
                self.pieces[origin as usize] = 0;
                self.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_KNIGHT;
            }
            PROMOTE_TO_QUEEN => {
                self.pieces[origin as usize] = 0;
                self.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_QUEEN;
            }
            PROMOTE_TO_ROOK => {
                self.pieces[origin as usize] = 0;
                self.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_ROOK;
            }
            BLACK_EN_PASSANT => {
                self.pieces[target as usize] = self.pieces[origin as usize];
                self.pieces[origin as usize] = 0;
                self.pieces[(target + 8) as usize] = 0;

            }
            WHITE_EN_PASSANT => {
                self.pieces[target as usize] = self.pieces[origin as usize];
                self.pieces[origin as usize] = 0;
                self.pieces[(target - 8) as usize] = 0;

            }
            _ => {println!("INVALID MOVE FLAG")}
        }

        //remove castle rights if one of the involved pieces is 
        match target {
            0 => {self.castle_rights &= 0b1101}
            7 => {self.castle_rights &= 0b1110}
            54 => {self.castle_rights &= 0b0111}
            63 => {self.castle_rights &= 0b1011}
            4 => {self.castle_rights &= 0b1100}
            60 => {self.castle_rights &= 0b0011}
            _ => {}
        }
        match origin {
            0 => {self.castle_rights &= 0b1101}
            7 => {self.castle_rights &= 0b1110}
            54 => {self.castle_rights &= 0b0111}
            63 => {self.castle_rights &= 0b1011}
            4 => {self.castle_rights &= 0b1100}
            60 => {self.castle_rights &= 0b0011}
            _ => {}
        }

        //If the en passsant square was not set this halfmove, then we should remove it, as it is old
        if flag != DOUBLE_PAWN_MOVE {
            self.en_passant_square = NO_EN_PASSANT_SQUARE;
        }
        self.white_to_move = !self.white_to_move;
        self.half_move_clock += 1;

    }


    pub fn pieces(&self) -> &[u8; 64]{
        return &self.pieces;
    }

    pub fn white_to_move(&self) -> bool{self.white_to_move}
    pub fn en_passant_square(&self) -> u8{self.en_passant_square}
}

impl Clone for BoardState{
    fn clone(&self) -> Self {
        Self { 
            pieces: self.pieces,
            white_to_move: self.white_to_move, 
            en_passant_square: self.en_passant_square, 
            castle_rights: self.castle_rights, 
            half_move_clock: self.half_move_clock,
            is_in_check: self.is_in_check,
            legal_moves: self.legal_moves,
            white_king: self.white_king,
            black_king: self.black_king
        }
    }
}



