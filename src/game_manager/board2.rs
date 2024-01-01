





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




//Move flags:
const PROMOTE_TO_KNIGHT:u8 = 0b0000;
const PROMOTE_TO_BISHOP:u8 = 0b0001;
const PROMOTE_TO_ROOK:u8 = 0b0010;
const PROMOTE_TO_QUEEN:u8 = 0b0011;

const W_CASTLE_KING:u8 = 0b0100;
const W_CASTLE_QUEEN:u8 = 0b0101;
const B_CASTLE_KING:u8 = 0b0110;
const B_CASTLE_QUEEN:u8 = 0b0111;

const WHITE_EN_PASSANT:u8 = 0b1000;
const BLACK_EN_PASSANT:u8 = 0b1001;

const DOUBLE_PAWN_MOVE:u8 = 0b1010;

const NO_FLAG:u8 = 0b1111;

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
    half_move_clock: u8

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
        Self { index: 0, chess_moves: [ChessMove{move_data: 0}; 218] }
    }

    pub fn add(&mut self, chess_move:ChessMove){
        self.chess_moves[self.index as usize] = chess_move;
        self.index += 1;
    }

    pub fn pop(&mut self) -> ChessMove{
        let result = self.chess_moves[self.index as usize];
        self.index -= 1;
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

    pub fn moves(&self) -> &[ChessMove; 218]{
        return &self.chess_moves;
    }
    pub fn moves_vec(&self) -> Vec<ChessMove>{
        let mut moves = Vec::<ChessMove>::new();
        for i in 0..218{
            if self.chess_moves[i].move_data != 0{
                moves.push(self.chess_moves[i]);
            }
        }
        return moves;
    }
}




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
                pieces[(rank*8 + file) as usize] = to_add;
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
            white_to_move: to_move,
            en_passant_square: en_passant_square, 
            castle_rights: castle_rights,
            half_move_clock: 0 }
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

    fn valid_origin(&self, origin: u8) -> bool{
        let white_on_origin = self.pieces[origin as usize] & PIECE_WHITE != 0; 
        return white_on_origin == self.white_to_move;
    }

    //compares square a and b and checks if they are the same color
    fn same_color(&self, a:u8, b:u8) -> bool{
        return self.pieces[a as usize] & PIECE_COLOR_MASK == self.pieces[b as usize] & PIECE_COLOR_MASK
    }

    fn horizontal_distance(a: u8, b:u8) -> u8{
        return ((a%8) as i8 - (b%8) as i8).abs() as u8;
    }
    fn vertical_distance(a: u8, b:u8) -> u8{
        return ((a/8) as i8 - (b/8) as i8).abs() as u8;
    }

    fn manhatten_distance(a:u8, b:u8) -> u8{
        return BoardState::horizontal_distance(a, b) + BoardState::vertical_distance(a, b);
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
                if self.pieces[double_move as usize] == 0 {
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
            if BoardState::manhatten_distance(origin, target) != 2{
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

            if BoardState::manhatten_distance(origin, target as u8) != 3{
                continue;
            }

            if target >= 64 || target < 0 {
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
                if BoardState::horizontal_distance(origin, target) != BoardState::vertical_distance(origin, target) {
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

                if BoardState::horizontal_distance(origin, target) != 0 && BoardState::vertical_distance(origin, target) != 0{
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

    fn legal_castle_moves(&self, chess_moves:&mut ChessMoveList){
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

    fn pseudo_legal_king_moves(&self, chess_moves:&mut ChessMoveList, origin: u8){
        for direction in [1, -1, 8, -8, 7, -7, 9, -9]{
            let target = origin as i8+direction;
            if target >= 64 || target < 0{
                continue;
            }
            let target = target as u8;
            if BoardState::manhatten_distance(origin, target) > 2 {
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

    fn pseudo_legal_moves(&self) -> ChessMoveList{
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
    //Missing "can not castle out of check rule"
    //?Implement in legal_moves() or pseudo_legal_king_moves() ?????
    pub fn legal_moves(&self) -> ChessMoveList{
        let mut moves = self.pseudo_legal_moves();
        for i in 0..218{
            if moves.chess_moves[i].move_data == 0 {
                continue;
            }
            let new_board_state = self.perform_move(moves.chess_moves[i]);
            if new_board_state.is_in_check(self.white_to_move){
                moves.chess_moves[i].move_data = 0;
            }
        }
        return moves;
    }
    
    //Blindly perform a move, and updates the board state. Does not check for legality
    pub fn perform_move(&self, chess_move:ChessMove) -> BoardState{

        let flag = chess_move.flag();
        let origin = chess_move.origin();
        let target = chess_move.target();

        let mut new_board_state = BoardState::clone(&self);


        match flag {
            NO_FLAG => {
                new_board_state.pieces[target as usize] = new_board_state.pieces[origin as usize];
                new_board_state.pieces[origin as usize] = 0;
            }
            DOUBLE_PAWN_MOVE => {
                new_board_state.pieces[target as usize] = new_board_state.pieces[origin as usize];
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.en_passant_square = if self.white_to_move {origin+8} else {origin-8};
            }
            W_CASTLE_KING => {
                new_board_state.pieces[4] = 0;
                new_board_state.pieces[7] = 0;
                new_board_state.pieces[5] = PIECE_WHITE | PIECE_ROOK;
                new_board_state.pieces[6] = PIECE_WHITE | PIECE_KING;
                new_board_state.castle_rights &= 0xFC;
            }
            W_CASTLE_QUEEN => {
                new_board_state.pieces[4] = 0;
                new_board_state.pieces[0] = 0;
                new_board_state.pieces[3] = PIECE_WHITE | PIECE_ROOK;
                new_board_state.pieces[2] = PIECE_WHITE | PIECE_KING;
                new_board_state.castle_rights &= 0xFC
            }
            B_CASTLE_KING => {
                new_board_state.pieces[60] = 0;
                new_board_state.pieces[63] = 0;
                new_board_state.pieces[61] = PIECE_BLACK | PIECE_ROOK;
                new_board_state.pieces[62] = PIECE_BLACK | PIECE_KING;
                new_board_state.castle_rights &= 0x03
            }
            B_CASTLE_QUEEN => {
                new_board_state.pieces[60] = 0;
                new_board_state.pieces[56] = 0;
                new_board_state.pieces[59] = PIECE_BLACK | PIECE_ROOK;
                new_board_state.pieces[58] = PIECE_BLACK | PIECE_KING;
                new_board_state.castle_rights &= 0x03;
            }
            PROMOTE_TO_BISHOP => {
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_BISHOP;
            }
            PROMOTE_TO_KNIGHT => {
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_KNIGHT;
            }
            PROMOTE_TO_QUEEN => {
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_QUEEN;
            }
            PROMOTE_TO_ROOK => {
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[target as usize] =  if self.white_to_move {PIECE_WHITE} else {PIECE_BLACK} | PIECE_ROOK;
            }
            BLACK_EN_PASSANT => {
                new_board_state.pieces[target as usize] = new_board_state.pieces[origin as usize];
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[(target + 8) as usize] = 0;

            }
            WHITE_EN_PASSANT => {
                new_board_state.pieces[target as usize] = new_board_state.pieces[origin as usize];
                new_board_state.pieces[origin as usize] = 0;
                new_board_state.pieces[(target - 8) as usize] = 0;

            }
            _ => {println!("INVALID MOVE FLAG")}
        }

        //remove castle rights if one of the involved pieces is 
        match target {
            0 => {new_board_state.castle_rights &= 0b1101}
            7 => {new_board_state.castle_rights &= 0b1110}
            54 => {new_board_state.castle_rights &= 0b0111}
            63 => {new_board_state.castle_rights &= 0b1011}
            4 => {new_board_state.castle_rights &= 0b1100}
            60 => {new_board_state.castle_rights &= 0b0011}
            _ => {}
        }
        match origin {
            0 => {new_board_state.castle_rights &= 0b1101}
            7 => {new_board_state.castle_rights &= 0b1110}
            54 => {new_board_state.castle_rights &= 0b0111}
            63 => {new_board_state.castle_rights &= 0b1011}
            4 => {new_board_state.castle_rights &= 0b1100}
            60 => {new_board_state.castle_rights &= 0b0011}
            _ => {}
        }

        //If the en passsant square was not set this halfmove, then we should remove it, as it is old
        if flag != DOUBLE_PAWN_MOVE {
            new_board_state.en_passant_square = NO_EN_PASSANT_SQUARE;
        }
        new_board_state.white_to_move = !new_board_state.white_to_move;
        new_board_state.half_move_clock += 1;


        return new_board_state;
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

            if BoardState::manhatten_distance(square, target as u8) != 3{
                continue;
            }

            if target >= 64 || target < 0 {
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
                if BoardState::horizontal_distance(square, target) != BoardState::vertical_distance(square, target) {
                    break;
                }

                if self.same_color(square, target){
                    break;
                }
                if self.pieces[target as usize] == attacker_color | PIECE_BISHOP ||
                    self.pieces[target as usize] == attacker_color | PIECE_QUEEN{
                    return true;
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

                if BoardState::horizontal_distance(square, target) != 0 && BoardState::vertical_distance(square, target) != 0{
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


        return false;
    }

    fn king_pos(&self, color:bool) -> usize{
        let mut king_pos = 0;
        let piece_color = if color {PIECE_WHITE} else {PIECE_BLACK};
        for i in 0..64{
            if self.pieces[i] == piece_color | PIECE_KING{
                return i;
            }
        }
        return 10000;
    }

    fn is_in_check(&self, color:bool) -> bool{
        return self.is_attacked(self.king_pos(color) as u8, !color);
    }

    pub fn perform_move_api(&self, mut origin:u8, mut target: u8, promote_to: u8) -> Option<BoardState>{

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
        let mut legal_moves_debug:[[u8; 3]; 218] = [[0; 3]; 218];
        for i in 0..218{
            legal_moves_debug[i][0] = legal_moves[i].flag();
            legal_moves_debug[i][1] = legal_moves[i].origin();
            legal_moves_debug[i][2] = legal_moves[i].target();
        }   
        // !DEBUG STUFF

        for legal_move in legal_moves{
            if chess_move == legal_move{
                return Some(self.perform_move(chess_move));
            }
        }
        return None;
    }

    pub fn legal_move_count(&self) -> usize{
        let moves = self.legal_moves();
        return moves.size();

    }

    pub fn game_state(&self) -> GameState{
        if self.legal_move_count() == 0{
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

    pub fn has_ended(&self) -> bool {
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

    pub fn white_to_move(&self) -> bool{self.white_to_move}
}

impl Clone for BoardState{
    fn clone(&self) -> Self {
        Self { 
            pieces: self.pieces, 
            white_to_move: self.white_to_move, 
            en_passant_square: self.en_passant_square, 
            castle_rights: self.castle_rights, 
            half_move_clock: self.half_move_clock }
    }
}

impl Copy for BoardState{}