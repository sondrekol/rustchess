

use self::bit_boards::BOARD_CENTER;

use super::board2::{BoardState, ChessMoveList, ChessMove, GameState};

pub mod bit_boards;
mod bit_boards_tests;

//Piece indexes
pub const PAWN:usize = 0;
pub const KNIGHT:usize = 1;
pub const BISHOP:usize = 2;
pub const ROOK:usize = 3;
pub const QUEEN:usize = 4;
pub const KING:usize = 5;

//Castle Rights
const WHITE_CASTLE_KING: u8 = 0b00000001;
const WHITE_CASTLE_QUEEN: u8 = 0b00000010;
const BLACK_CASTLE_KING: u8 = 0b00000100;
const BLACK_CASTLE_QUEEN: u8 = 0b00001000;

//Color define
pub const WHITE: usize = 1;
pub const BLACK: usize = 0;

//en passant
const NO_EN_PASSANT_SQUARE:usize = 0x80;




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


//smaller state used for transposition table
#[derive(Hash, PartialEq, Eq)]
pub struct BoardStateNumbers{
    piece_bb: [u64; 6],
    piece_bb_color: [u64; 2],
    data: u8,
}


/*
    NOTE:
    the state of the position should not be mutated,



*/
pub struct BitBoardState{
    //Board state
    piece_bb: [[u64; 6];2],
    color_mask: [u64; 2], //indexed by color, return a bitboard of pieces with said color
    to_move: usize,
    other: usize,
    en_passant_square: usize,
    en_passant_possible: bool,
    castle_w_k: bool,
    castle_w_q: bool,
    castle_b_k: bool,
    castle_b_q: bool,

    //move lists
    legal_moves: ChessMoveList,
    legal_moves_calculated: bool,

    //helpers
    checkers: u64, //bitboard containing the current checkers
    check_line: u64, //line bitboard of checkline if king is checked by sliding piece
    pinned_pieces: u64, //bitboard containing all pinned pieces
    pinned_pieces_indicies: [usize; 8], //list of indicies of pinned pieces
    pin_lines: [u64; 8], //8 line bitboards corresponding to pinned pieces
    rank_2th: [u64; 2] //bit board for each color of corresponding 7th rank
}

impl BitBoardState{
    pub fn new() -> Self{
        Self {
            piece_bb: [[0; 6]; 2],
            to_move: 2,
            other: 2,
            en_passant_square: 0,
            en_passant_possible: false,
            castle_w_k: false,
            castle_w_q: false,
            castle_b_k: false,
            castle_b_q: false,

            legal_moves: ChessMoveList::new(), //move lists should only be instansiated here
            legal_moves_calculated: false,

            color_mask: [0; 2],
            checkers: 0,
            check_line: 0,
            pinned_pieces: 0,
            pinned_pieces_indicies: [64; 8], //value 64 represents no pinned piece
            pin_lines: [0; 8],
            rank_2th: [bit_boards::RANK_7, bit_boards::RANK_2]
        }
    }

    fn piece_bb_from_board_state(&mut self, board_state_pieces:&[u8; 64]){
        self.piece_bb = [[0; 6]; 2];
        for i in 0..64{
            match board_state_pieces[i]{
                0b10000001 => {self.piece_bb[WHITE][PAWN] |= 1 << i}
                0b10000010 => {self.piece_bb[WHITE][KNIGHT] |= 1 << i}
                0b10000011 => {self.piece_bb[WHITE][BISHOP] |= 1 << i}
                0b10000100 => {self.piece_bb[WHITE][ROOK] |= 1 << i}
                0b10000101 => {self.piece_bb[WHITE][QUEEN] |= 1 << i}
                0b10000110 => {self.piece_bb[WHITE][KING] |= 1 << i}
                0b01000001 => {self.piece_bb[BLACK][PAWN] |= 1 << i}
                0b01000010 => {self.piece_bb[BLACK][KNIGHT] |= 1 << i}
                0b01000011 => {self.piece_bb[BLACK][BISHOP] |= 1 << i}
                0b01000100 => {self.piece_bb[BLACK][ROOK] |= 1 << i}
                0b01000101 => {self.piece_bb[BLACK][QUEEN] |= 1 << i}
                0b01000110 => {self.piece_bb[BLACK][KING] |= 1 << i}
                _ => {}
            }
        }
        self.color_mask[BLACK] = 
                                self.piece_bb[BLACK][PAWN] |
                                self.piece_bb[BLACK][KNIGHT] |
                                self.piece_bb[BLACK][BISHOP] |
                                self.piece_bb[BLACK][ROOK] |
                                self.piece_bb[BLACK][QUEEN] |
                                self.piece_bb[BLACK][KING];
        self.color_mask[WHITE] = 
                                self.piece_bb[WHITE][PAWN] |
                                self.piece_bb[WHITE][KNIGHT] |
                                self.piece_bb[WHITE][BISHOP] |
                                self.piece_bb[WHITE][ROOK] |
                                self.piece_bb[WHITE][QUEEN] |
                                self.piece_bb[WHITE][KING];
    }

    fn setup_state(&mut self, board_state:&BoardState){

        self.piece_bb_from_board_state(board_state.pieces());

        self.to_move = if board_state.white_to_move() {1} else {0};
        self.other = if board_state.white_to_move() {0} else {1};

        self.en_passant_square = board_state.en_passant_square() as usize;

        self.en_passant_possible = if self.en_passant_square >= 64 {false} else {true};

        let castle_rights = board_state.castle_rights();
        self.castle_w_k = if castle_rights & 0b0001 != 0 {true} else {false};
        self.castle_w_q = if castle_rights & 0b0010 != 0 {true} else {false};
        self.castle_b_k = if castle_rights & 0b0100 != 0 {true} else {false};
        self.castle_b_q = if castle_rights & 0b1000 != 0 {true} else {false};

        //TODO: move clock

        self.checkers = 0;
        self.check_line = 0;
        self.pinned_pieces = 0;
        self.pinned_pieces_indicies = [64; 8];
        self.pin_lines = [0; 8];
        
    }


    //does not add moves that move into mask (mask=1 represents a square that should no be move to)
    //otherwise adds moves defined by target bb
    fn generate_moves_target_masked(&mut self, origin_square: usize, mut targets: u64, flag: u8, mask: u64) {
        targets &= !mask;
        while targets != 0 {
            let target = bit_boards::pop_lsb(&mut targets);
            self.legal_moves.add_no_alloc(origin_square as u8, target as u8, flag);
        }
    }

    //like generate_moves but does not add moves moving into attacked square
    fn generate_moves_for_king(&mut self, origin_square: usize, mut targets: u64, flag: u8) {
        while targets != 0 {
            let target = bit_boards::pop_lsb(&mut targets);
            if self.attackers(target) != 0 {
                continue;
            }
            self.legal_moves.add_no_alloc(origin_square as u8, target as u8, flag);
        }
    }

    fn generate_en_passant(&mut self, pawn:usize, en_passant_square:u64){
        if en_passant_square == 0 {
            return;
        }


        //check if results in check
        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        let captured_by_enpassant = if self.to_move == WHITE {en_passant_square >> 8} else {en_passant_square << 8};
        if pawn/8 == king_pos/8 {//if pawn and king are on same file
            let rook_moves_bb = bit_boards::RookMoves::mov_map(
                king_pos, 
                (self.color_mask[WHITE] | self.color_mask[BLACK]) & !(captured_by_enpassant|(1 << pawn)));

            let king_rook_bb:u64;
            if pawn%8 < king_pos%8 {//never equal, as it would imply pawn and king occupy the same square
                king_rook_bb = rook_moves_bb & bit_boards::west(king_pos);

            }else{
                king_rook_bb = rook_moves_bb & bit_boards::east(king_pos);

            }
            if king_rook_bb & (self.piece_bb[self.other][ROOK] | self.piece_bb[self.other][QUEEN]) != 0{
                return; //This en passant move results in check
            }
        }

        if self.to_move == WHITE{
            self.legal_moves.add_no_alloc(
                pawn as u8, 
                u64::trailing_zeros(en_passant_square) as u8, 
                WHITE_EN_PASSANT)
        }else{
            self.legal_moves.add_no_alloc(
                pawn as u8, 
                u64::trailing_zeros(en_passant_square) as u8, 
                BLACK_EN_PASSANT)
        }
        
    }

    //generates legal pawn moves for pawns on 2nd rank
    fn legal_pawn_on_2nd(&mut self, mut pawns: u64, mask:u64){
        while pawns != 0 {
            let pawn = bit_boards::pop_lsb(&mut pawns);
            let normal_captures = bit_boards::PAWN_CAPTURES[self.to_move][pawn] & self.color_mask[self.other];
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
            let normal_moves = normal_captures | ((1 << normal_move) & !(self.color_mask[self.to_move] | self.color_mask[self.other]));

            self.generate_moves_target_masked(pawn, normal_moves, 0b1111, mask);
            if (1 << normal_move) & (self.color_mask[self.to_move] | self.color_mask[self.other]) == 0{
                let double_pawn_square = if self.to_move == WHITE {pawn + 16} else {pawn-16}; //Hacky as fuck, but may help branch predictor
                let double_move = 1 << (double_pawn_square) & !(self.color_mask[self.to_move] | self.color_mask[self.other]);
                self.generate_moves_target_masked(pawn, double_move, 0b1010, mask);
            }
        }
    }

    //generates legal pawn moves for pawns on 7th rank
    fn legal_pawn_on_7th(&mut self, mut pawns: u64, mask:u64){
        while pawns != 0 {
            let pawn = bit_boards::pop_lsb(&mut pawns);
            let normal_captures = bit_boards::PAWN_CAPTURES[self.to_move][pawn] & self.color_mask[self.other];
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
            let normal_moves = normal_captures | ((1 << normal_move) & !(self.color_mask[self.to_move] | self.color_mask[self.other]));
            self.generate_moves_target_masked(pawn, normal_moves, 0b0000, mask);
            self.generate_moves_target_masked(pawn, normal_moves, 0b0001, mask);
            self.generate_moves_target_masked(pawn, normal_moves, 0b0010, mask);
            self.generate_moves_target_masked(pawn, normal_moves, 0b0011, mask);
        }
    }

    //generates legal pawn moves for pawns neither on 7th nor 2nd rank
    fn legal_pawn_mid_board(&mut self, mut pawns: u64, mask:u64) {
        while pawns != 0 {
            let pawn = bit_boards::pop_lsb(&mut pawns);
            let capture_targets = bit_boards::PAWN_CAPTURES[self.to_move][pawn];
            let normal_captures = capture_targets & self.color_mask[self.other];
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
            let normal_moves = normal_captures | ((1 << normal_move) & !(self.color_mask[self.to_move] | self.color_mask[self.other]));
            self.generate_moves_target_masked(pawn, normal_moves, 0b1111, mask);


            if self.en_passant_possible{
                let en_passant_capture = capture_targets & (1 << self.en_passant_square) & !mask;
                self.generate_en_passant(pawn, en_passant_capture);
            }
        }
    }

    fn legal_pawn_moves(&mut self, mut pawns: u64, mask:u64){
        let rank_2nd_pawns = pawns & self.rank_2th[self.to_move];
        let rank_7th_pawns = pawns & self.rank_2th[self.other];
        self.legal_pawn_on_2nd(rank_2nd_pawns, mask);
        self.legal_pawn_on_7th(rank_7th_pawns, mask);
        self.legal_pawn_mid_board(pawns & !(rank_2nd_pawns | rank_7th_pawns), mask);
    }

    fn legal_knight_moves(&mut self, mut knights: u64, mask:u64){
        while knights != 0 {

            let knight = bit_boards::pop_lsb(&mut knights);

            let moves_bb = bit_boards::KNIGHT_MOVES[knight] & !(self.color_mask[self.to_move]);

            self.generate_moves_target_masked( knight, moves_bb, 0b1111, mask);
        }
    }

    fn legal_king_moves(&mut self){

        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;

        let moves_bb = bit_boards::KING_MOVES[king_pos] & !(self.color_mask[self.to_move]);

        self.generate_moves_for_king(king_pos, moves_bb, 0b1111);
    }

    fn legal_evading_king_moves(&mut self){
        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        let moves_bb = bit_boards::KING_MOVES[king_pos] & !(self.color_mask[self.to_move]);

        let mut towards_check = moves_bb & self.check_line;
        let offset1 = bit_boards::pop_lsb(&mut towards_check) as i32 - king_pos as i32;
        let opposite_offset1 = king_pos as i32 - offset1;

        
        let mut away_from_check:u64 = if opposite_offset1 >= 0 && opposite_offset1 < 64 {
                                        1 << opposite_offset1
                                    }else {0}; //add first opposite offset
                                
        if towards_check != 0 {
            let offset2 = bit_boards::pop_lsb(&mut towards_check) as i32 - king_pos as i32;
            let opposite_offset2 = king_pos as i32 - offset2; //in the case of single checking sliding piece this will be 0
            away_from_check |= if opposite_offset2 >= 0 && opposite_offset2 < 64 {
                                        1 << opposite_offset2
                                    }else {0}; //add second opposite offset
        }
        
        self.generate_moves_for_king(king_pos, moves_bb & !away_from_check, 0b1111);


        /*NOTE: 
        no check for validity of away_from_check as a king move,
        if it is not valid, then no such move exists and the map has no effect on the moves
        */
    }

    //Calculate all pseudo legal rook moves, on squares provided by "rooks"
    fn legal_rook_moves(&mut self, mut rooks:u64, mask:u64){
        while rooks != 0 {
            let rook_pos =bit_boards::pop_lsb(&mut rooks);

            let moves_bb = bit_boards::RookMoves::mov_map(
                rook_pos, 
                self.color_mask[self.to_move] |
                        self.color_mask[self.other]) & !(self.color_mask[self.to_move]);
            self.generate_moves_target_masked(rook_pos, moves_bb, 0b1111, mask);
        }
    }

    //Calculate all pseudo legal bishop moves, on squares provided by "bishops"
    fn legal_bishop_moves(&mut self, mut bishops:u64, mask:u64){
        while bishops != 0 {
            let bishop_pos =bit_boards::pop_lsb(&mut bishops);

            let moves_bb = bit_boards::BishopMoves::mov_map(
                bishop_pos, 
                self.color_mask[self.to_move] |
                        self.color_mask[self.other]) & !(self.color_mask[self.to_move]);
            self.generate_moves_target_masked(bishop_pos, moves_bb, 0b1111, mask);
        }
    }

    //Calculate all pseudo legal queen moves, on squares provided by "queens"
    fn legal_queen_moves(&mut self, mut queens:u64, mask:u64){
        while queens != 0 {
            let queen_pos =bit_boards::pop_lsb(&mut queens);

            let moves_bb = (bit_boards::RookMoves::mov_map(
                         queen_pos, 
                self.color_mask[self.to_move] |
                        self.color_mask[self.other]) |
                        bit_boards::BishopMoves::mov_map(
                            queen_pos, 
                   self.color_mask[self.to_move] |
                           self.color_mask[self.other])) & !(self.color_mask[self.to_move]);

            self.generate_moves_target_masked(queen_pos, moves_bb, 0b1111, mask);
        }
    }

    //generate legal castle_moves
    //this function assumes that "to move side" is not in check
    fn legal_castles(&mut self){
        //check for castle rights -> then neither king or rook has moved
        //check for pieces blocking the castle
        //check for attacked squares on king_path
        if self.to_move == WHITE{
            if self.castle_w_k {
                if (self.color_mask[BLACK] | self.color_mask[WHITE]) & bit_boards::CASTLE_W_K_LINE == 0 {
                    if self.attackers(5) == 0 && self.attackers(6) == 0{
                        self.legal_moves.add_no_alloc(0, 0, 0b0100);
                    }
                }
            }
            if self.castle_w_q {
                if (self.color_mask[BLACK] | self.color_mask[WHITE]) & bit_boards::CASTLE_W_Q_LINE == 0 {
                    if self.attackers(2) == 0 && self.attackers(3) == 0{
                        self.legal_moves.add_no_alloc(0, 0, 0b0101);
                    }
                }
            }
        }else{
            if self.castle_b_k {
                if (self.color_mask[BLACK] | self.color_mask[WHITE]) & bit_boards::CASTLE_B_K_LINE == 0 {
                    if self.attackers(61) == 0 && self.attackers(62) == 0{
                        self.legal_moves.add_no_alloc(0, 0, 0b0110);
                    }
                }
            }
            if self.castle_b_q {
                if (self.color_mask[BLACK] | self.color_mask[WHITE]) & bit_boards::CASTLE_B_Q_LINE == 0 {
                    if self.attackers(59) == 0 && self.attackers(58) == 0{
                        self.legal_moves.add_no_alloc(0, 0, 0b0111);
                    }
                }
            }
        }
    }

    //generate all legal moves except for castles
    //this function assumes "to move side" is not in check
    //a move is then legal if and only if it is
        //pseudo legal +
        //for king: does not move into check
        //for pinned pieces: does not move out of pin line
    fn legal_moves(&mut self, mask:u64){
        //generate normal moves for king

        //generate for non pinned rook, bishop, queen, knight:
        self.legal_knight_moves(self.piece_bb[self.to_move][KNIGHT] & !self.pinned_pieces, mask);
        self.legal_bishop_moves(self.piece_bb[self.to_move][BISHOP] & !self.pinned_pieces, mask);
        self.legal_rook_moves(self.piece_bb[self.to_move][ROOK] & !self.pinned_pieces, mask);
        self.legal_queen_moves(self.piece_bb[self.to_move][QUEEN] & !self.pinned_pieces, mask);
        self.legal_pawn_moves(self.piece_bb[self.to_move][PAWN] & !self.pinned_pieces, mask);

        //generate moves for pinned pieces
        for i in 0..8{
            if self.pinned_pieces_indicies[i] != 64{
                let pinned_piece_map = 1 << self.pinned_pieces_indicies[i];
                self.legal_knight_moves(self.piece_bb[self.to_move][KNIGHT] & pinned_piece_map, !self.pin_lines[i] | mask);
                self.legal_bishop_moves(self.piece_bb[self.to_move][BISHOP] & pinned_piece_map, !self.pin_lines[i] | mask);
                self.legal_rook_moves(self.piece_bb[self.to_move][ROOK] & pinned_piece_map, !self.pin_lines[i] | mask);
                self.legal_queen_moves(self.piece_bb[self.to_move][QUEEN] & pinned_piece_map, !self.pin_lines[i] | mask);
                self.legal_pawn_moves(self.piece_bb[self.to_move][PAWN] & pinned_piece_map, !self.pin_lines[i] | mask);
            }
        }
    }

    fn update_pinned_pieces_and_check_line(&mut self){
        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;

        //generate move bb, note that these "moves" pass right trough the kings own pieces
        let rook_moves_bb = bit_boards::RookMoves::mov_map(
            king_pos, 
        self.color_mask[self.other]);
        let bishop_moves_bb = bit_boards::BishopMoves::mov_map(
            king_pos, 
        self.color_mask[self.other]);
        
        
        let north = rook_moves_bb & bit_boards::north(king_pos);
        let south = rook_moves_bb & bit_boards::south(king_pos);
        let east = rook_moves_bb & bit_boards::east(king_pos);
        let west = rook_moves_bb & bit_boards::west(king_pos);
        let northeast = bishop_moves_bb & bit_boards::north_of(king_pos) & bit_boards::east_of(king_pos);
        let northwest = bishop_moves_bb & bit_boards::north_of(king_pos) & bit_boards::west_of(king_pos);
        let southeast = bishop_moves_bb & bit_boards::south_of(king_pos) & bit_boards::east_of(king_pos);
        let southwest = bishop_moves_bb & bit_boards::south_of(king_pos) & bit_boards::west_of (king_pos);
        
        let mut pinned_piece_index:usize = 0; //this index corresponds to the pinned_piece_indicies lisst, not a square on the board
        for rookmoves in [north, south, east, west]{
            if rookmoves & (self.piece_bb[self.other][ROOK] | self.piece_bb[self.other][QUEEN]) != 0{
                let own_pieces_on_path = rookmoves & self.color_mask[self.to_move];
                if u64::count_ones(own_pieces_on_path) == 1 { 
                    //pinned piece detected
                    self.pinned_pieces |= own_pieces_on_path; //update the bb to include pinned piece
                    self.pinned_pieces_indicies[pinned_piece_index] = u64::trailing_zeros(own_pieces_on_path) as usize;
                    self.pin_lines[pinned_piece_index] = rookmoves;
                    pinned_piece_index+=1;

                }
                //if there is no piece between king and attack, then king is checked trough this line
                else if u64::count_ones(own_pieces_on_path) == 0{
                    self.check_line |= rookmoves;
                }
            }
        }
        for bishopmoves in [northeast, northwest, southeast, southwest]{
            if bishopmoves & (self.piece_bb[self.other][BISHOP] | self.piece_bb[self.other][QUEEN]) != 0{
                let own_pieces_on_path = bishopmoves & self.color_mask[self.to_move];
                if u64::count_ones(own_pieces_on_path) == 1 { 
                    //pinned piece detected
                    self.pinned_pieces |= own_pieces_on_path; //update the bb to include pinned piece
                    self.pinned_pieces_indicies[pinned_piece_index] = u64::trailing_zeros(own_pieces_on_path) as usize;
                    self.pin_lines[pinned_piece_index] = bishopmoves;
                    pinned_piece_index+=1;

                }
                //if there is no piece between king and attack, then king is checked trough this line
                else if u64::count_ones(own_pieces_on_path) == 0{
                    self.check_line |= bishopmoves;
                }
            }
        }
    }

    fn attackers(&self, pos: usize) -> u64{

        //Count rook attackers
        let rook_moves_bb = bit_boards::RookMoves::mov_map(
            pos, 
            self.color_mask[self.to_move] |
                    self.color_mask[self.other]) & !(self.color_mask[self.to_move]);
        let rook_attackers = rook_moves_bb & (self.piece_bb[self.other][ROOK] | self.piece_bb[self.other][QUEEN]);

        //count bishop attackers
        let bishop_moves_bb = bit_boards::BishopMoves::mov_map(
            pos, 
            self.color_mask[self.to_move] |
                    self.color_mask[self.other]) & !(self.color_mask[self.to_move]);
        let bishop_attackers = bishop_moves_bb & (self.piece_bb[self.other][BISHOP] | self.piece_bb[self.other][QUEEN]);

        //Count knight attackers
        let knight_moves_bb = bit_boards::KNIGHT_MOVES[pos] & !(self.color_mask[self.to_move]);
        let knight_attackers = knight_moves_bb & self.piece_bb[self.other][KNIGHT];

        //Count pawn attackers
        let pawn_moves_bb = bit_boards::PAWN_CAPTURES[self.to_move][pos] & !(self.color_mask[self.to_move]);
        let pawn_attackers = pawn_moves_bb & self.piece_bb[self.other][PAWN];
        
        //Count king attackeers
        let king_attack_bb = bit_boards::KING_MOVES[pos] & !(self.color_mask[self.to_move]);
        let king_attacker = king_attack_bb & self.piece_bb[self.other][KING];
        return rook_attackers | bishop_attackers | knight_attackers | pawn_attackers | king_attacker;
    }

    //update checkers bitboard, return number of checkers
    fn num_checkers(&mut self) -> usize{
        
        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        self.checkers = self.attackers(king_pos);

        return u64::count_ones(self.checkers) as usize;
    }

    //?for now moves are precalculated
    pub fn gen_moves_legal(&mut self) -> ChessMoveList{
        if self.legal_moves_calculated {
            return self.legal_moves;
        }
        self.legal_moves.reset();

        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        if king_pos >= 64 {
            println!("s");
            panic!("king position is invalid");
        }

        self.update_pinned_pieces_and_check_line();

        match self.num_checkers() {
            2 => {
                self.legal_evading_king_moves(); //If double check, only generate legal king moves
            }
            1 => {
                if self.check_line == 0 {
                    self.legal_king_moves();
                    self.legal_moves(!self.checkers); //happens if knight or pawn check, in which case non king moves must capture the pawn

                     //if there is a pawn checking, and en passant possible then you should also generate the up to 2 legal en passant moves
                    if self.en_passant_possible && self.piece_bb[self.other][PAWN] & self.checkers != 0{
                        let mut pawns = self.piece_bb[self.to_move][PAWN];
                        while pawns != 0 {
                            let pawn = bit_boards::pop_lsb(&mut pawns);
                            let capture_targets = bit_boards::PAWN_CAPTURES[self.to_move][pawn];
                            let en_passant_capture = capture_targets & (1 << self.en_passant_square);
                            self.generate_en_passant(pawn, en_passant_capture);
                        }
                    }
                }else{
                    self.legal_evading_king_moves();
                    self.legal_moves(!self.check_line); //Generate only moves blocking check, or capturing checking piece
                }
            }
            0 => {
                self.legal_castles();
                self.legal_king_moves();
                self.legal_moves(0);
            }
            _ => {
                panic!("More than 2 checkers");
            }
        }
        self.legal_moves_calculated = true;
        return self.legal_moves;
    }
    
    pub fn piece_count(&self) -> i32{
        return 
         (u64::count_ones(self.piece_bb[WHITE][PAWN]) as i32)*10
        +(u64::count_ones(self.piece_bb[WHITE][KNIGHT]) as i32)*30
        +(u64::count_ones(self.piece_bb[WHITE][BISHOP]) as i32)*35
        +(u64::count_ones(self.piece_bb[WHITE][ROOK]) as i32)*50
        +(u64::count_ones(self.piece_bb[WHITE][QUEEN]) as i32)*90
        -(u64::count_ones(self.piece_bb[BLACK][PAWN]) as i32)*10
        -(u64::count_ones(self.piece_bb[BLACK][KNIGHT]) as i32)*30
        -(u64::count_ones(self.piece_bb[BLACK][BISHOP]) as i32)*35
        -(u64::count_ones(self.piece_bb[BLACK][ROOK]) as i32)*50
        -(u64::count_ones(self.piece_bb[BLACK][QUEEN]) as i32)*90;
        
    }
    pub fn update_state(&mut self){

    }

    pub fn board_setup(&mut self, board_state:&BoardState){
        self.setup_state(board_state);
    }

    pub fn perform_move(&self, chess_move:ChessMove) -> BitBoardState{
        let origin: u8 = chess_move.origin();
        let origin_bb: u64 = 1<<origin;
        let target: u8 = chess_move.target();
        let target_bb: u64 = 1<<target;
        let flag: u8 = chess_move.flag();


        let mut new_piece_bb: [[u64; 6]; 2] = self.piece_bb;
        let mut new_en_passant_square: usize = NO_EN_PASSANT_SQUARE;
        let mut new_en_passant_possible: bool = false;
        let mut new_castle_w_k: bool = self.castle_w_k;
        let mut new_castle_w_q: bool = self.castle_w_q;
        let mut new_castle_b_k: bool = self.castle_b_k;
        let mut new_castle_b_q: bool = self.castle_b_q;

        let mut move_piece = ||{
            //get which piece is being moved
            let mut piece:usize = 0;
            for i in 0..6{
                if new_piece_bb[self.to_move][i] & origin_bb != 0 {piece = i; break;}
            }

            //remove pieces from origin square
            new_piece_bb[self.to_move][piece] &= !origin_bb;

            //add new piece
            new_piece_bb[self.to_move][piece] |= target_bb;

        };

        match flag {
            NO_FLAG => {
                move_piece();

                //capture other pieces
                new_piece_bb[self.other][PAWN] &= !target_bb;
                new_piece_bb[self.other][KNIGHT] &= !target_bb;
                new_piece_bb[self.other][BISHOP] &= !target_bb;
                new_piece_bb[self.other][ROOK] &= !target_bb;
                new_piece_bb[self.other][QUEEN] &= !target_bb;

            }
            DOUBLE_PAWN_MOVE => {
                new_piece_bb[self.to_move][PAWN] &= !origin_bb;
                new_piece_bb[self.to_move][PAWN] |= target_bb;

                if self.to_move == WHITE {
                    new_en_passant_square = (origin + 8) as usize;
                    new_en_passant_possible = true;
                }else{
                    new_en_passant_square = (origin - 8) as usize;
                    new_en_passant_possible = true;
                }
            }
            BLACK_EN_PASSANT => {
                move_piece();
                new_piece_bb[WHITE][PAWN] &= !(target_bb << 8);
            }
            WHITE_EN_PASSANT => {
                move_piece();
                new_piece_bb[BLACK][PAWN] &= !(target_bb >> 8);
            }
            B_CASTLE_QUEEN => {
                new_piece_bb[BLACK][KING] = 1 << 58; //overwrite king bb with new position
                new_piece_bb[BLACK][ROOK] &= !(1 << 56); //remove corner rook
                new_piece_bb[BLACK][ROOK] |= 1 << 59; //add new rook position
                new_castle_b_k = false;
                new_castle_b_q = false;
            }
            B_CASTLE_KING => {
                new_piece_bb[BLACK][KING] = 1 << 62; //overwrite king bb with new position
                new_piece_bb[BLACK][ROOK] &= !(1 << 63); //remove corner rook
                new_piece_bb[BLACK][ROOK] |= 1 << 61; //add new rook position
                new_castle_b_k = false;
                new_castle_b_q = false;
            }
            W_CASTLE_QUEEN => {
                new_piece_bb[WHITE][KING] = 1 << 2; //overwrite king bb with new position
                new_piece_bb[WHITE][ROOK] &= !(1 << 0); //remove corner rook
                new_piece_bb[WHITE][ROOK] |= 1 << 3; //add new rook position
                new_castle_w_k = false;
                new_castle_w_q = false;
            }
            W_CASTLE_KING => {
                new_piece_bb[WHITE][KING] = 1 << 6; //overwrite king bb with new position
                new_piece_bb[WHITE][ROOK] &= !(1 << 7); //remove corner rook
                new_piece_bb[WHITE][ROOK] |= 1 << 5; //add new rook position
                new_castle_w_k = false;
                new_castle_w_q = false;
            }
            PROMOTE_TO_QUEEN => {
                new_piece_bb[self.to_move][PAWN] &= !origin_bb;
                new_piece_bb[self.to_move][QUEEN] |= target_bb;

                //capture other pieces
                new_piece_bb[self.other][PAWN] &= !target_bb;
                new_piece_bb[self.other][KNIGHT] &= !target_bb;
                new_piece_bb[self.other][BISHOP] &= !target_bb;
                new_piece_bb[self.other][ROOK] &= !target_bb;
                new_piece_bb[self.other][QUEEN] &= !target_bb;
            }
            PROMOTE_TO_ROOK => {
                new_piece_bb[self.to_move][PAWN] &= !origin_bb;
                new_piece_bb[self.to_move][ROOK] |= target_bb;

                //capture other pieces
                new_piece_bb[self.other][PAWN] &= !target_bb;
                new_piece_bb[self.other][KNIGHT] &= !target_bb;
                new_piece_bb[self.other][BISHOP] &= !target_bb;
                new_piece_bb[self.other][ROOK] &= !target_bb;
                new_piece_bb[self.other][QUEEN] &= !target_bb;
            }
            PROMOTE_TO_BISHOP => {
                new_piece_bb[self.to_move][PAWN] &= !origin_bb;
                new_piece_bb[self.to_move][BISHOP] |= target_bb;

                //capture other pieces
                new_piece_bb[self.other][PAWN] &= !target_bb;
                new_piece_bb[self.other][KNIGHT] &= !target_bb;
                new_piece_bb[self.other][BISHOP] &= !target_bb;
                new_piece_bb[self.other][ROOK] &= !target_bb;
                new_piece_bb[self.other][QUEEN] &= !target_bb;
            }
            PROMOTE_TO_KNIGHT => {
                new_piece_bb[self.to_move][PAWN] &= !origin_bb;
                new_piece_bb[self.to_move][KNIGHT] |= target_bb;

                //capture other pieces
                new_piece_bb[self.other][PAWN] &= !target_bb;
                new_piece_bb[self.other][KNIGHT] &= !target_bb;
                new_piece_bb[self.other][BISHOP] &= !target_bb;
                new_piece_bb[self.other][ROOK] &= !target_bb;
                new_piece_bb[self.other][QUEEN] &= !target_bb;
            }
            _ => {
                panic!("Illegal flag move attempted")
            }
        }

        const WHITE_KING_CASTLE_PIECES:u64 = (1 << 4) | (1 << 7);
        const WHITE_QUEEN_CASTLE_PIECES:u64 = (1 << 4) | (1 << 0);
        const BLACK_KING_CASTLE_PIECES:u64 = (1 << 60) | (1 << 63);
        const BLACK_QUEEN_CASTLE_PIECES:u64 = (1 << 60) | (1 << 56);
        
        if flag != W_CASTLE_KING && flag != B_CASTLE_KING && flag != W_CASTLE_QUEEN && flag != B_CASTLE_QUEEN{
            if WHITE_KING_CASTLE_PIECES & (origin_bb | target_bb) != 0{
                new_castle_w_k = false;
            }
            if WHITE_QUEEN_CASTLE_PIECES & (origin_bb | target_bb) != 0{
                new_castle_w_q = false;
            }
            if BLACK_KING_CASTLE_PIECES & (origin_bb | target_bb) != 0{
                new_castle_b_k = false;
            }
            if BLACK_QUEEN_CASTLE_PIECES & (origin_bb | target_bb) != 0{
                new_castle_b_q = false;
            }
        }

        if flag != DOUBLE_PAWN_MOVE {
            new_en_passant_possible = false;
            new_en_passant_square = NO_EN_PASSANT_SQUARE;
        }

        let mut new_color_mask:[u64; 2] = [0; 2];
        new_color_mask[WHITE] = new_piece_bb[WHITE][PAWN] |
                                new_piece_bb[WHITE][KNIGHT] |
                                new_piece_bb[WHITE][BISHOP] |
                                new_piece_bb[WHITE][ROOK] |
                                new_piece_bb[WHITE][QUEEN] |
                                new_piece_bb[WHITE][KING];
        new_color_mask[BLACK] = new_piece_bb[BLACK][PAWN] |
                                new_piece_bb[BLACK][KNIGHT] |
                                new_piece_bb[BLACK][BISHOP] |
                                new_piece_bb[BLACK][ROOK] |
                                new_piece_bb[BLACK][QUEEN] |
                                new_piece_bb[BLACK][KING];

        return Self { 
            piece_bb: new_piece_bb, 
            to_move: self.other, 
            other: self.to_move, 
            en_passant_square: new_en_passant_square,
            en_passant_possible: new_en_passant_possible, 
            castle_w_k: new_castle_w_k, 
            castle_w_q: new_castle_w_q, 
            castle_b_k: new_castle_b_k, 
            castle_b_q: new_castle_b_q,

            legal_moves: ChessMoveList::new(), //move lists should only be instansiated here
            legal_moves_calculated: false,


            color_mask: new_color_mask,
            checkers: 0,
            check_line: 0,
            pinned_pieces: 0,
            pinned_pieces_indicies: [64; 8], //value 64 represents no pinned piece
            pin_lines: [0; 8],
            rank_2th: [bit_boards::RANK_7, bit_boards::RANK_2]
        };
    }

    pub fn white_to_move(&self) -> bool {
        return self.to_move == WHITE;
    }

    #[inline(always)]
    pub fn piece_value(&self, square:usize) -> i32 {
        assert!(square < 64);
        let mask = 1 << square;
        return 
         (u64::count_ones(self.piece_bb[WHITE][PAWN] & mask) as i32)*10
        +(u64::count_ones(self.piece_bb[WHITE][KNIGHT] & mask) as i32)*30
        +(u64::count_ones(self.piece_bb[WHITE][BISHOP] & mask) as i32)*35
        +(u64::count_ones(self.piece_bb[WHITE][ROOK] & mask) as i32)*50
        +(u64::count_ones(self.piece_bb[WHITE][QUEEN] & mask) as i32)*90
        -(u64::count_ones(self.piece_bb[BLACK][PAWN] & mask) as i32)*10
        -(u64::count_ones(self.piece_bb[BLACK][KNIGHT] & mask) as i32)*30
        -(u64::count_ones(self.piece_bb[BLACK][BISHOP] & mask) as i32)*35
        -(u64::count_ones(self.piece_bb[BLACK][ROOK] & mask) as i32)*50
        -(u64::count_ones(self.piece_bb[BLACK][QUEEN] & mask) as i32)*90;
    }

    pub fn game_state(&mut self) -> GameState{
        //use allready calculated moves if possible
        if self.legal_moves_calculated {
            if self.legal_moves.size_fast() == 0{
                if self.checkers != 0 { //to move side is in checkmate
                    if self.to_move == WHITE{
                        return GameState::Black;
                    }else{
                        return GameState::White;
                    }
                }else{
                    return GameState::Draw; //to move side has no moves but is not in check
                }
            }else{
                return GameState::Playing;
            }
        }
        //check for only king moves, this does not need to precompute anything
        else{
            self.legal_king_moves();
            let legal_king_moves = self.legal_moves.size_fast();
            self.legal_moves.reset();
            if legal_king_moves >= 3 { 
                return GameState::Playing;
            }
        }
        //finally if all else fails, do a full search of all moves to check if there are 0 legal moves
        if self.gen_moves_legal().size_fast() == 0{
            if self.checkers != 0 { //to move side is in checkmate
                if self.to_move == WHITE{
                    return GameState::Black;
                }else{
                    return GameState::White;
                }
            }else{
                return GameState::Draw; //to move side has no moves but is not in check
            }
        }else{
            return GameState::Playing;
        }
    }

    pub fn piece_bb(&self) -> [[u64; 6]; 2]{
        return self.piece_bb;
    }

    pub fn board_state_numbers(&self) -> BoardStateNumbers{
        let mut castle_rights = 0;
        castle_rights |= if self.castle_w_k {0b0001} else {0};
        castle_rights |= if self.castle_w_q {0b0010} else {0};
        castle_rights |= if self.castle_b_k {0b0100} else {0};
        castle_rights |= if self.castle_b_q {0b1000} else {0};
        let mut pieces = [0; 6];
        for i in 0..6{
            pieces[i] = self.piece_bb[WHITE][i] | self.piece_bb[BLACK][i];
        }
        let colors = [
            self.piece_bb[BLACK][PAWN] | self.piece_bb[BLACK][KNIGHT] | self.piece_bb[BLACK][BISHOP] | self.piece_bb[BLACK][ROOK] | self.piece_bb[BLACK][QUEEN] | self.piece_bb[BLACK][KING],
            self.piece_bb[WHITE][PAWN] | self.piece_bb[WHITE][KNIGHT] | self.piece_bb[WHITE][BISHOP] | self.piece_bb[WHITE][ROOK] | self.piece_bb[WHITE][QUEEN] | self.piece_bb[WHITE][KING],
        ];
        BoardStateNumbers{
            piece_bb: pieces,
            piece_bb_color: colors,
            data: (self.to_move as u8) << 7 | ((self.en_passant_square%8) as u8) << 4 | castle_rights
        }
    }


    pub fn knights_in_center(&self, color: usize) -> i32{
        return u64::count_ones(self.piece_bb[color][KNIGHT] & BOARD_CENTER) as i32;
    }

    //returns the map of attackers for color, color=BLACK will return blacks attackers on the square
    fn get_attackers_for_color(&mut self, square: usize, color:usize) -> u64{
        let mut attackers:u64 = 0;
        if color != self.to_move{
            attackers = self.attackers(square);
        }else{
            self.other = self.to_move;
            self.to_move = color;
            attackers = self.attackers(square);
            self.to_move = self.other;
            self.other = color;
        }
        return attackers;
    }

    //returns the absolute value of the least worth piece controlling a square
    pub fn least_valuable_controller(&mut self, square: usize, color:usize) -> i32{
        let attackers:u64 = self.get_attackers_for_color(square, color);
        if attackers & self.piece_bb[color][PAWN] != 0{return 10}
        if attackers & self.piece_bb[color][KNIGHT] != 0{return 30}
        if attackers & self.piece_bb[color][BISHOP] != 0{return 35}
        if attackers & self.piece_bb[color][ROOK] != 0{return 50}
        if attackers & self.piece_bb[color][QUEEN] != 0{return 90}
        if attackers & self.piece_bb[color][KING] != 0{return 150}
        return 0; //return some high value 
    }

    //returns mask of all pieces on the board
    pub fn piece_mask(&self) -> u64{
        return self.color_mask[WHITE] | self.color_mask[BLACK];
    }
}


impl Clone for BitBoardState{
    fn clone(&self) -> Self {
        Self {
            piece_bb: [[0; 6]; 2],
            to_move: 2,
            other: 2,
            en_passant_square: 0,
            en_passant_possible: false,
            castle_w_k: false,
            castle_w_q: false,
            castle_b_k: false,
            castle_b_q: false,

            legal_moves: ChessMoveList::new(), //move lists should only be instansiated here
            legal_moves_calculated: false,

            color_mask: [0; 2],
            checkers: 0,
            check_line: 0,
            pinned_pieces: 0,
            pinned_pieces_indicies: [64; 8], //value 64 represents no pinned piece
            pin_lines: [0; 8],
            rank_2th: [bit_boards::RANK_7, bit_boards::RANK_2]
        }
    }
}

impl Copy for BitBoardState{

}
