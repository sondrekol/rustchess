





//Piece codes

use super::state_bitboard::BitBoardState;



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


pub static VERTICAL_DISTANCE:[[u8; 64];64] = {
    let mut dis:[[u8; 64];64] = [[0; 64];64];
    let mut i:u8 = 0;
    
    while i < 64{
        let mut j:u8 = 0;
        while j < 64{
            dis[i as usize][j as usize] = BoardState::vertical_distance(i, j);
            j+=1;
        }
        i+=1;
    }
    dis
};
pub static HORIZONTAL_DISTANCE:[[u8; 64];64] = {
    let mut dis:[[u8; 64];64] = [[0; 64];64];
    let mut i:u8 = 0;
    
    while i < 64{
        let mut j:u8 = 0;
        while j < 64{
            dis[i as usize][j as usize] = BoardState::horizontal_distance(i, j);
            j+=1;
        }
        i+=1;
    }
    dis
};

#[derive(PartialEq)]
pub enum GameState{
    Playing,
    White,
    Black,
    Draw
}



pub struct ChessMove{
    move_data:u16
}

pub struct ChessMoveList{
    index: u8,
    chess_moves: [ChessMove; 218]
}


pub struct BoardState{
    pieces: [u8; 64],
    pieces_backup: [u8; 64],
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
    black_king: usize,
    move_history: Vec<ChessMove>,
    piece_history: Vec<[u8; 64]>

}



impl ChessMove{
    /*
    First 4 bits are flagss
    Next 6 bits are the index of the target square
    last 6 bits are the index of the origin square
     */
    pub fn new_empty() -> Self{
        Self { move_data: 0 }
    }

    pub fn new_exact(move_data:u16) -> Self{
        Self { move_data: move_data }
    }

    pub fn from_indices(flags: u8, origin:u8, target: u8) -> Self{
        Self { 
            move_data:  (((flags as u16) & 0x0F) << 12) | 
                        (((target as u16) & 0x3F) << 6) | 
                        (((origin as u16) & 0x3F))
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

    pub fn is_null(&self) -> bool{
        return self.move_data == 0;
    }

    pub fn move_data(&self) -> u16{
        return self.move_data;
    }

}

impl Clone for ChessMove{
    fn clone(&self) -> Self {
        Self { move_data: self.move_data }
    }
}

impl Copy for ChessMove{}

impl PartialEq for ChessMove {
    fn eq(&self, other: &Self) -> bool {
        self.move_data == other.move_data
    }
}

impl ChessMoveList{
    pub fn new() -> Self{
        Self { 
            index: 0, 
            chess_moves: [ChessMove{move_data: 0}; 218]
        }
    }

    pub fn add(&mut self, chess_move:ChessMove){
        self.chess_moves[self.index as usize] = chess_move;
        self.index += 1;
    }

    pub fn add_no_alloc(&mut self, origin:u8, target:u8, flag:u8){
        self.chess_moves[self.index as usize].move_data = origin as u16 | ((target as u16) << 6) | ((flag as u16) << 12);
        self.index+= 1;
    }

    pub fn pop(&mut self) -> ChessMove{
        self.index -= 1;
        let result = self.chess_moves[self.index as usize];
        return result;
    }

    pub fn size(&self) -> usize{
        let mut size = 0;
        for i in 0..218{
            if self.chess_moves[i].move_data != 0{
                size+=1;
            }
        }
        return size;
    }

    //NOTE: this does not work if the moves are mutated
    pub fn size_fast(&self) -> usize{
        return self.index as usize;
    }

    pub fn moves(&self) -> &[ChessMove; 218]{
        return &self.chess_moves;
    }
    
    pub fn moves_vec(&self) -> Vec<ChessMove>{
        let mut moves = Vec::<ChessMove>::with_capacity(218);
        for i in 0..self.index as usize{
            if self.chess_moves[i].move_data != 0{
                moves.push(self.chess_moves[i]);
            }
        }
        return moves;
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl Clone for ChessMoveList{
    fn clone(&self) -> Self {
        Self { index: self.index.clone(), chess_moves: self.chess_moves.clone() }
    }
}

impl Copy for ChessMoveList{}


impl BoardState{

    pub fn new_from_fen(fen:&str) -> Self{
        let mut pieces:[u8; 64] = [0; 64];
        let mut to_move:bool = false;
        let mut en_passant_square:u8 = NO_EN_PASSANT_SQUARE;
        let mut castle_rights:u8 = 0x00;
        let mut half_move_clock: usize = 0;
        let mut turns:usize = 0;


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
            pieces_backup: pieces,
            white_to_move: to_move,
            en_passant_square: en_passant_square, 
            castle_rights: castle_rights,
            half_move_clock: 0,
            is_in_check: None,
            legal_moves: None,
            white_king: white_king,
            black_king: black_king,
            move_history: Vec::<ChessMove>::new(),
            piece_history: Vec::<[u8; 64]>::new()
        }
    }

    pub fn get_board(&self) -> [[i8; 8]; 8]{
        let mut graphic_board = [[0_i8; 8]; 8];
        for i in 0..64{
            let graphic_rank = 7-i/8;
            let graphic_file = i%8;
            let mut piece_type = 0;
            match self.pieces[i as usize] & PIECE_TYPE_MASK{
                PIECE_PAWN => {
                    piece_type = 1;
                }
                PIECE_KNIGHT => {
                    piece_type = 2;
                }
                PIECE_BISHOP => {
                    piece_type = 3;
                }
                PIECE_ROOK => {
                    piece_type = 4;
                }
                PIECE_QUEEN => {
                    piece_type = 5;
                }
                PIECE_KING => {
                    piece_type = 6;
                }
                _ => {
                    piece_type = 0;
                }
            }
            if self.pieces[i as usize] & PIECE_COLOR_MASK == PIECE_BLACK {
                piece_type*= -1;
            }
            graphic_board[graphic_rank][graphic_file] = piece_type;
        }
        return graphic_board;
    }

    pub fn equal_game_state(a:&BoardState, b:&BoardState) -> bool {
        if a.pieces != b.pieces {
            return false;
        }
        if a.en_passant_square != b.en_passant_square {
            return false;
        }
        if a.castle_rights != b.castle_rights {
            return false;
        }
        if a.half_move_clock != b.half_move_clock{
            return false;
        }
        if a.white_to_move != b.white_to_move{
            return false;
        }
        return true;
    }
    pub fn piece(&self, index: usize) -> u8{
        return self.pieces[index];
    }

    pub fn castle_rights(&self) -> u8{
        return self.castle_rights;
    }

    pub fn set_piece(&mut self, index: usize, piece: u8) {
        self.pieces[index] = piece;
    }

    pub fn set_castle_rights(&mut self, castle_rights:u8){
        self.castle_rights = castle_rights;
    }

    fn valid_origin(&self, origin: u8) -> bool{
        let white_on_origin = self.pieces[origin as usize] & PIECE_WHITE != 0; 
        return white_on_origin == self.white_to_move;
    }

    //compares square a and b and checks if they are the same color
    fn same_color(&self, a:u8, b:u8) -> bool{
        return self.pieces[a as usize] & PIECE_COLOR_MASK == self.pieces[b as usize] & PIECE_COLOR_MASK
    }

    pub const fn horizontal_distance(a: u8, b:u8) -> u8{
        return ((a%8) as i8 - (b%8) as i8).abs() as u8;
    }
    pub const fn vertical_distance(a: u8, b:u8) -> u8{
        return ((a/8) as i8 - (b/8) as i8).abs() as u8;
    }

    fn manhatten_distance(a:&u8, b:&u8) -> u8{
        return (HORIZONTAL_DISTANCE[*a as usize][*b as usize] + VERTICAL_DISTANCE[*a as usize][*b as usize]) as u8;
    }


    fn pseudo_legal_pawn_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){
        let direction:i8 = if self.white_to_move {8} else {-8};
        /*
        normal move and double move
        en passant square is set upon performing the move
         */
        let regular_move = origin as i8 + direction; //target square on regular move
        let double_move = origin as i8 + direction*2; //target square on double move


        fn add_pawn_move_promotion_checked(chess_moves:&mut ChessMoveList, origin:u8, target:u8, white_to_move:bool){
            if white_to_move && target >= 56 || !white_to_move && target < 8{
                for promote_flag in [PROMOTE_TO_BISHOP, PROMOTE_TO_KNIGHT, PROMOTE_TO_QUEEN, PROMOTE_TO_ROOK]{
                    chess_moves.add(
                        ChessMove::from_indices(
                            promote_flag, 
                            origin, 
                            target)
                    );
                }
            }else{
                chess_moves.add(
                    ChessMove::from_indices(
                        0b1111, 
                        origin, 
                        target)
                );
            }
        }

        if self.pieces[regular_move as usize] == 0 {
            add_pawn_move_promotion_checked(chess_moves, origin, regular_move as u8, self.white_to_move);
            if double_move >= 0 && double_move < 64{
                if self.pieces[double_move as usize] == 0 && origin/8 == if self.white_to_move {1} else {6}{
                    //?Note that this move would create a 
                    chess_moves.add(
                        ChessMove::from_indices(
                            DOUBLE_PAWN_MOVE,
                            origin, 
                            double_move as u8)
                    );
                }
            }
        }

        //check for captures
        for i in [-1, 1]{
            let target = origin as i8 + direction + i;
            if target >= 64 || target < 0{
                continue;
            }
            let target = target as u8;
            if BoardState::manhatten_distance(&origin, &target) != 2{
                continue;
            }

            if !self.same_color(origin, target) && self.pieces[target as usize] != 0{
                add_pawn_move_promotion_checked(chess_moves, origin, target, self.white_to_move)
            }
            else if target == self.en_passant_square{
                chess_moves.add(
                    ChessMove::from_indices(
                        if self.white_to_move {0b1000} else {0b1001}, 
                        origin, 
                        target)
                )
            }
        }
    }

    fn pseudo_legal_knight_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){
        for direction in [-17, -15, -10, -6, 6, 10, 15, 17]{
            let target = origin as i8+direction;

            if target >= 64 || target < 0 {
                continue;
            }

            if BoardState::manhatten_distance(&origin, &(target as u8)) != 3{
                continue;
            }

            
            if self.same_color(origin, target as u8) {
                continue;
            }
            chess_moves.add(
                ChessMove::from_indices(
                    0b1111, 
                    origin, 
                    target as u8)
            )
        }
    }

    fn pseudo_legal_bishop_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){
        for direction in [-7, 7, -9, 9]{
            for i in 1..8{
                let target = origin as i8+direction*i;
                if target >= 64 || target < 0{
                    break;
                }
                let target = target as u8;
                if HORIZONTAL_DISTANCE[origin as usize][target as usize] != VERTICAL_DISTANCE[origin as usize][target as usize] {
                    break;
                }

                if self.same_color(origin, target){
                    break;
                }
                else if self.pieces[target as usize] == 0{
                    chess_moves.add(
                        ChessMove::from_indices(
                            0b1111,
                            origin,
                            target

                    ));
                }else {
                    chess_moves.add(
                        ChessMove::from_indices(
                            0b1111,
                            origin,
                            target

                    ));
                    break;
                }
            }
        }
    }

    fn pseudo_legal_rook_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){

        for direction in [-8, 8, -1, 1]{
            for i in 1..8{
                let target = origin as i8+direction*i;
                if target >= 64 || target < 0{
                    break;
                }
                let target = target as u8;

                if HORIZONTAL_DISTANCE[origin as usize][target as usize] != 0 && VERTICAL_DISTANCE[origin as usize][target as usize] != 0{
                    break;
                }

                if self.same_color(origin, target){
                    break;
                }
                else if self.pieces[target as usize] == 0{
                    chess_moves.add(
                        ChessMove::from_indices(
                            0b1111,
                            origin,
                            target

                    ));
                }else {
                    chess_moves.add(
                        ChessMove::from_indices(
                            0b1111,
                            origin,
                            target

                    ));
                    break;
                }
            }
        }
    }  

    fn pseudo_legal_queen_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){
        self.pseudo_legal_bishop_moves(chess_moves, origin);
        self.pseudo_legal_rook_moves(chess_moves, origin);
    }

    fn can_castle(&self, king_pos:u8, rook_pos:u8) -> bool{
        //At this point one allready knows that king and rook is positioned correctly
        let min = std::cmp::min(king_pos, rook_pos);
        let max = std::cmp::max(king_pos, rook_pos);
        for i in min+1..max{
            if self.pieces[i as usize] != 0{
                return false;
            }
        }
        return true;
    }

    fn legal_castle_moves(&mut self, chess_moves:&mut ChessMoveList){
        let back_rank = if self.white_to_move {0} else {56};
        if self.is_in_check(self.white_to_move) {
            return;
        }
        if self.white_to_move {
            if self.castle_rights & WHITE_CASTLE_KING != 0 && self.can_castle(4, 7){
                chess_moves.add(ChessMove::from_indices(
                    W_CASTLE_KING, 
                    0, 
                    0))
            }
            if self.castle_rights & WHITE_CASTLE_QUEEN != 0 && self.can_castle(4, 0){
                chess_moves.add(ChessMove::from_indices(
                    W_CASTLE_QUEEN, 
                    0, 
                    0))
            }
        }else{
            if self.castle_rights & BLACK_CASTLE_KING != 0 && self.can_castle(60, 63){
                chess_moves.add(ChessMove::from_indices(
                    B_CASTLE_KING, 
                    0, 
                    0))
            }
            if self.castle_rights & BLACK_CASTLE_QUEEN != 0 && self.can_castle(60, 56){
                chess_moves.add(ChessMove::from_indices(
                    B_CASTLE_QUEEN, 
                    0, 
                    0))                
            }
        }
    }

    fn pseudo_legal_king_moves(&mut self, chess_moves:&mut ChessMoveList, origin: u8){
        for direction in [1, -1, 8, -8, 7, -7, 9, -9]{
            let target = origin as i8+direction;
            if target >= 64 || target < 0{
                continue;
            }
            let target = target as u8;
            if BoardState::manhatten_distance(&origin, &target) > 2 {
                continue;
            }
            if self.same_color(origin, target){
                continue;
            }
            else {
                chess_moves.add(
                    ChessMove::from_indices(
                        0b1111,
                        origin,
                        target

                ));
            }
            
        }
        self.legal_castle_moves(chess_moves);
    }

    fn pseudo_legal_moves(&mut self) -> ChessMoveList{
        let mut chess_moves = ChessMoveList::new();
        for i in 0..64{
            let i = i;
            if (self.pieces[i] & PIECE_COLOR_MASK == PIECE_WHITE) != self.white_to_move{
                continue;
            }
            match self.pieces[i] & PIECE_TYPE_MASK{
                0 => {continue;}
                PIECE_PAWN => {self.pseudo_legal_pawn_moves(&mut chess_moves, i as u8)}
                PIECE_KNIGHT => {self.pseudo_legal_knight_moves(&mut chess_moves, i as u8)}
                PIECE_BISHOP => {self.pseudo_legal_bishop_moves(&mut chess_moves, i as u8)}
                PIECE_ROOK => {self.pseudo_legal_rook_moves(&mut chess_moves, i as u8)}
                PIECE_QUEEN => {self.pseudo_legal_queen_moves(&mut chess_moves, i as u8)}
                PIECE_KING => {self.pseudo_legal_king_moves(&mut chess_moves, i as u8)}
                _ => {println!("INVALID PIECE")}
            }
        }
        return chess_moves;
    }
    pub fn is_illegal(&mut self, chess_move:ChessMove) -> bool{
        let mut is_illegal = false;
        //Make move
        let captured_piece = self.pieces[chess_move.target() as usize];
        let castle_rights = self.castle_rights;
        self.perform_move_mutable(chess_move);

        if self.is_in_check(!self.white_to_move){
            is_illegal = true;
        }
        self.undo_move_mutable(chess_move);
        self.pieces[chess_move.target() as usize] = captured_piece;
        self.castle_rights = castle_rights;
        return is_illegal;
    }

    pub fn legal_moves_gen(&mut self) -> ChessMoveList{
        let mut moves = self.pseudo_legal_moves();
        for i in 0..218{
            if moves.chess_moves[i].move_data == 0 {
                continue;
            }
            let chess_move = moves.chess_moves[i];

            if self.is_illegal(chess_move){moves.chess_moves[i].move_data = 0;}
        }
        return moves;
    }

    pub fn no_legal_moves(&mut self) -> bool{
        self.legal_moves = Some(ChessMoveList::new());
        let mut chess_moves = self.legal_moves.unwrap();
        for i in 0..64{
            let i = i;
            if (self.pieces[i] & PIECE_COLOR_MASK == PIECE_WHITE) != self.white_to_move{
                continue;
            }
            match self.pieces[i] & PIECE_TYPE_MASK{
                0 => {continue;}
                PIECE_PAWN => {self.pseudo_legal_pawn_moves(&mut chess_moves, i as u8)}
                PIECE_KNIGHT => {self.pseudo_legal_knight_moves(&mut chess_moves, i as u8)}
                PIECE_BISHOP => {self.pseudo_legal_bishop_moves(&mut chess_moves, i as u8)}
                PIECE_ROOK => {self.pseudo_legal_rook_moves(&mut chess_moves, i as u8)}
                PIECE_QUEEN => {self.pseudo_legal_queen_moves(&mut chess_moves, i as u8)}
                PIECE_KING => {self.pseudo_legal_king_moves(&mut chess_moves, i as u8)}
                _ => {println!("INVALID PIECE")}
            }
            while chess_moves.index > 0 {
                let chess_move = chess_moves.pop();
                if self.is_illegal(chess_move) {
                    continue;
                }else{
                    return false;
                }
            }
        }
        return true;
    }

    pub fn legal_moves(&mut self) -> ChessMoveList{
        if self.legal_moves.is_none(){
            self.legal_moves = Some(self.legal_moves_gen());
        }
        return self.legal_moves.unwrap();
    }
    
    //Blindly perform a move, and updates the board state. Does not check for legality
    pub fn perform_move(&self, chess_move:ChessMove) -> BoardState{
        let mut new_board_state = self.clone();
        new_board_state.perform_move_mutable(chess_move);
        return new_board_state;
    }
    pub fn perform_move_mutable(&mut self, chess_move:ChessMove){
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

    pub fn undo_move_mutable(&mut self, chess_move:ChessMove){
        //self.move_history.pop();
        //self.piece_history.pop();
        let flag = chess_move.flag();
        let origin = chess_move.origin();
        let target = chess_move.target();

        self.legal_moves = None;
        self.is_in_check = None;

        self.white_to_move = !self.white_to_move;
        self.half_move_clock -= 1;

        match flag {
            NO_FLAG => {
                self.pieces[origin as usize] = self.pieces[target as usize];
                if self.pieces[origin as usize] == PIECE_WHITE | PIECE_KING{
                    self.white_king = origin as usize;
                }else if self.pieces[origin as usize] == PIECE_BLACK | PIECE_KING{
                    self.black_king = origin as usize;
                }
            }
            DOUBLE_PAWN_MOVE => {
                self.pieces[origin as usize] = self.pieces[target as usize];
                self.en_passant_square = NO_EN_PASSANT_SQUARE;
            }
            W_CASTLE_KING => {
                self.pieces[6] = 0;
                self.pieces[5] = 0;
                self.pieces[7] = PIECE_WHITE | PIECE_ROOK;
                self.pieces[4] = PIECE_WHITE | PIECE_KING;
                self.white_king = 4;
            }
            W_CASTLE_QUEEN => {
                self.pieces[2] = 0;
                self.pieces[3] = 0;
                self.pieces[0] = PIECE_WHITE | PIECE_ROOK;
                self.pieces[4] = PIECE_WHITE | PIECE_KING;
                self.white_king = 4;
            }
            B_CASTLE_KING => {
                self.pieces[62] = 0;
                self.pieces[61] = 0;
                self.pieces[63] = PIECE_BLACK | PIECE_ROOK;
                self.pieces[60] = PIECE_BLACK | PIECE_KING;
                self.black_king = 60;
            }
            B_CASTLE_QUEEN => {
                self.pieces[58] = 0;
                self.pieces[59] = 0;
                self.pieces[56] = PIECE_BLACK | PIECE_ROOK;
                self.pieces[60] = PIECE_BLACK | PIECE_KING;
                self.black_king = 60;
            }
            BLACK_EN_PASSANT => {
                self.pieces[origin as usize] = self.pieces[target as usize];
                self.pieces[target as usize] = 0;
                self.pieces[(target + 8) as usize] = PIECE_WHITE | PIECE_PAWN;

            }
            WHITE_EN_PASSANT => {
                self.pieces[origin as usize] = self.pieces[target as usize];
                self.pieces[target as usize] = 0;
                self.pieces[(target - 8) as usize] = PIECE_BLACK | PIECE_PAWN;

            }
            PROMOTE_TO_BISHOP | PROMOTE_TO_KNIGHT | PROMOTE_TO_QUEEN | PROMOTE_TO_ROOK => {
                self.pieces[origin as usize] = if self.white_to_move {PIECE_WHITE | PIECE_PAWN} else {PIECE_BLACK | PIECE_PAWN};
            }
            _ => {println!("INVALID MOVE FLAG")}
        }

    }

    fn is_attacked(&self, square:u8, attacked_by:bool) -> bool{
        let attacker_color = if attacked_by {PIECE_WHITE} else {PIECE_BLACK};
        let attacker_pawn_direction = if attacked_by {8} else {-8};


        //Check for pawns
        let pawn_attack_1 = square as i8- attacker_pawn_direction + 1;
        let pawn_attack_2 = square as i8- attacker_pawn_direction - 1;

        for pawn_attack in [pawn_attack_1, pawn_attack_2]{
            if pawn_attack >= 64 || pawn_attack < 0{
                continue;
            }
            if (pawn_attack%8 - (square as i8)%8).abs() != 1{
                continue;
            }
            if self.pieces[pawn_attack as usize] == attacker_color | PIECE_PAWN {
                return true;
            }

        }

        //check for knights
        for direction in [-17, -15, -10, -6, 6, 10, 15, 17]{
            let target = square as i8+direction;
            if target >= 64 || target < 0 {
                continue;
            }

            if BoardState::manhatten_distance(&square, &(target as u8)) != 3{
                continue;
            }

            
            if self.pieces[target as usize] == attacker_color | PIECE_KNIGHT{
                return true;
            }

        }

        //check diagonal
        for direction in [-7, 7, -9, 9]{
            for i in 1..8{
                let target = square as i8+direction*i;
                if target >= 64 || target < 0{
                    break;
                }
                let target = target as u8;
                if HORIZONTAL_DISTANCE[square as usize][target as usize] != VERTICAL_DISTANCE[square as usize][target as usize] {
                    break;
                }

                if self.same_color(square, target){
                    break;
                }

                if self.pieces[target as usize] == attacker_color | PIECE_BISHOP ||
                    self.pieces[target as usize] == attacker_color | PIECE_QUEEN{
                    return true;
                }
                if self.pieces[target as usize] != 0{
                    break;
                }
            }
        }

        //check rook
        for direction in [-8, 8, -1, 1]{
            for i in 1..8{
                let target = square as i8+direction*i;
                if target >= 64 || target < 0{
                    break;
                }
                let target = target as u8;

                if HORIZONTAL_DISTANCE[square as usize][target as usize] != 0 && VERTICAL_DISTANCE[square as usize][target as usize] != 0{
                    break;
                }

                if self.same_color(square, target){
                    break;
                }
                else {
                    if self.pieces[target as usize] == attacker_color | PIECE_ROOK ||
                        self.pieces[target as usize] == attacker_color | PIECE_QUEEN{
                        return true;
                    }
                    if self.pieces[target as usize] != 0 {
                        break;
                    }
                }
            }
        }

        //check king
        for direction in [1, -1, 7, 8, 9, -7, -8, -9]{
            let target = square as i8+direction;
            if target >= 64 || target < 0{
                continue;
            }
            let target = target as u8;

            if HORIZONTAL_DISTANCE[square as usize][target as usize] > 1 || VERTICAL_DISTANCE[square as usize][target as usize] > 1{
                continue;
            }

            if self.same_color(square, target){
                continue;
            }
            else {
                if self.pieces[target as usize] == attacker_color | PIECE_KING{
                    return true;
                }
            }
            
        }


        return false;
    }

    fn king_pos(&self, color:bool) -> usize{
        return if color {self.white_king} else {self.black_king}
    }

    fn is_in_check(&mut self, color:bool) -> bool{
        if self.is_in_check.is_none(){
            self.is_in_check = Some(self.is_attacked(self.king_pos(color) as u8, !color));
        }
        return self.is_in_check.unwrap();
    }

    pub fn perform_move_api(&mut self, mut origin:u8, mut target: u8, promote_to: u8) -> Option<BoardState>{

        //if pawn move:
            //check for en passant
            //check for promotion
            //check for double move
        //if king move:
            //check for castle

        let mut flag:u8 = 0b1111;

        // !PAWN STUFF
        if self.pieces[origin as usize] & PIECE_TYPE_MASK == PIECE_PAWN{
            let pawn_direction = if self.pieces[origin as usize] & PIECE_COLOR_MASK == PIECE_WHITE {8} else {-8};
            if target as i8 == origin as i8 + pawn_direction + 1 || target as i8 == origin as i8 + pawn_direction - 1{
                if self.en_passant_square == target{
                    flag = if self.white_to_move {WHITE_EN_PASSANT} else {BLACK_EN_PASSANT};
                }
            }
            if target/8 == if self.white_to_move {7} else {0}{
                flag = promote_to;
            }
            if target/8 == origin/8 + 2 || (target/8) as i8 == ((origin/8) as i8) - 2{
                flag = DOUBLE_PAWN_MOVE;
            }

        }

        if self.pieces[origin as usize] & PIECE_TYPE_MASK == PIECE_KING{
            if origin == 4 && target == 6{
                flag = W_CASTLE_KING;
                origin = 0;
                target = 0;
            }
            else if origin == 4 && target == 2{
                flag = W_CASTLE_QUEEN;
                origin = 0;
                target = 0;
            }
            else if origin == 60 && target == 62{
                flag = B_CASTLE_KING;
                origin = 0;
                target = 0;
            }
            else if origin == 60 && target == 58{
                flag = B_CASTLE_QUEEN;
                origin = 0;
                target = 0;
            }
        }
        




        let mut chess_move:ChessMove;
        //TODO: create ChessMove
        chess_move = ChessMove{
            move_data: (flag as u16) << 12 | origin as u16 | (target as u16) << 6
        };
        let legal_moves = self.legal_moves().chess_moves;


        // !DEBUG STUFF
        let mut legal_moves_debug = self.legal_moves_debug();  
        // !DEBUG STUFF

        for legal_move in legal_moves{
            if chess_move == legal_move{
                return Some(self.perform_move(chess_move));
            }
        }
        return None;
    }

    pub fn legal_moves_debug(&mut self) -> [[u8; 3]; 218]{
        let legal_moves = self.legal_moves().chess_moves;
        let mut legal_moves_debug:[[u8; 3]; 218] = [[0; 3]; 218];
        for i in 0..218{
            legal_moves_debug[i][0] = legal_moves[i].flag();
            legal_moves_debug[i][1] = legal_moves[i].origin();
            legal_moves_debug[i][2] = legal_moves[i].target();
        }  
        return legal_moves_debug;
    }

    pub fn legal_move_count(&mut self) -> usize{
        let moves = self.legal_moves();
        return moves.size();

    }

    pub fn game_state(&mut self) -> GameState{
        if self.no_legal_moves(){
            if self.white_to_move && self.is_in_check(self.white_to_move){
                return GameState::Black;
            }else if  self.is_in_check(self.white_to_move){
                return GameState::White;
            }else{
                return GameState::Draw;
            }
        }
        return GameState::Playing;
    }

    pub fn has_ended(&mut self) -> bool {
        return self.game_state() != GameState::Playing;
    }

    pub fn piece_count(&self) -> f64{
        let mut piece_count:f64 = 0.0;
        for i in 0..64{
            if self.pieces[i] == 0{
                continue;
            }

            if self.pieces[i] & PIECE_COLOR_MASK == PIECE_WHITE{
                match self.pieces[i] & PIECE_TYPE_MASK{
                    PIECE_PAWN => {piece_count+=1.0}
                    PIECE_KNIGHT => {piece_count+=3.0}
                    PIECE_BISHOP => {piece_count+=3.0}
                    PIECE_ROOK => {piece_count+=5.0}
                    PIECE_QUEEN => {piece_count+=9.0}
                    _ => {}
                }
            }else {
                match self.pieces[i] & PIECE_TYPE_MASK{
                    PIECE_PAWN => {piece_count-=1.0}
                    PIECE_KNIGHT => {piece_count-=3.0}
                    PIECE_BISHOP => {piece_count-=3.0}
                    PIECE_ROOK => {piece_count-=5.0}
                    PIECE_QUEEN => {piece_count-=9.0}
                    _ => {}
                }
            }
        }
        return piece_count;
    }

    pub fn piece_value(&self, square: u8) -> f64{
        let mut value:f64 = 0.0;
        match self.pieces[square as usize] & PIECE_TYPE_MASK{
            PIECE_PAWN => {value = 1.0}
            PIECE_KNIGHT => {value = 3.0}
            PIECE_BISHOP => {value = 3.0}
            PIECE_ROOK => {value = 5.0}
            PIECE_QUEEN => {value = 9.0}
            PIECE_KING => {value = 1.0}
            0 => {value = 0.0}
            _ => {println!("INVALID PIECE AT {}, PIECE: {}", square, self.pieces[square as usize])}
        }
        if self.pieces[square as usize] | PIECE_COLOR_MASK == PIECE_BLACK {
            value *= -1.0;
        }
        return value;
    }

    pub fn pieces(&self) -> &[u8; 64]{
        return &self.pieces;
    }

    pub fn white_to_move(&self) -> bool{self.white_to_move}
    pub fn en_passant_square(&self) -> u8{self.en_passant_square}
    pub fn casstle_rights(&self) -> u8{self.castle_rights}
}

impl Clone for BoardState{
    fn clone(&self) -> Self {
        Self { 
            pieces: self.pieces, 
            pieces_backup: self.pieces,
            white_to_move: self.white_to_move, 
            en_passant_square: self.en_passant_square, 
            castle_rights: self.castle_rights, 
            half_move_clock: self.half_move_clock,
            is_in_check: self.is_in_check,
            legal_moves: self.legal_moves,
            white_king: self.white_king,
            black_king: self.black_king,
            move_history: Vec::<ChessMove>::new(),
            piece_history: Vec::<[u8; 64]>::new()
        }
    }
}



