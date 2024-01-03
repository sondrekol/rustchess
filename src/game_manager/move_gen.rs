

use super::board2::{BoardState, ChessMoveList};

mod bit_boards;
mod bit_boards_tests;

//Piece indexes
const PAWN:usize = 0;
const KNIGHT:usize = 1;
const BISHOP:usize = 2;
const ROOK:usize = 3;
const QUEEN:usize = 4;
const KING:usize = 5;

//Castle Rights
const WHITE_CASTLE_KING: u8 = 0b00000001;
const WHITE_CASTLE_QUEEN: u8 = 0b00000010;
const BLACK_CASTLE_KING: u8 = 0b00000100;
const BLACK_CASTLE_QUEEN: u8 = 0b00001000;

//Color define
const WHITE: usize = 1;
const BLACK: usize = 0;

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



pub struct MoveGen{
    //Board state
    piece_bb: [[u64; 6];2],
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
    check_moves: ChessMoveList,
    capture_moves: ChessMoveList,

    //helpers
    color_mask: [u64; 2], //indexed by color, return a bitboard of pieces with said color
    checkers: u64, //bitboard containing the current checkers
    check_line: u64, //line bitboard of checkline if king is checked by sliding piece
    pinned_pieces: u64, //bitboard containing all pinned pieces
    pinned_pieces_indicies: [usize; 8], //list of indicies of pinned pieces
    pin_lines: [u64; 8], //8 line bitboards corresponding to pinned pieces
    rank_2th: [u64; 2] //bit board for each color of corresponding 7th rank
}

impl MoveGen{
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
            check_moves: ChessMoveList::new(), //move lists should only be instansiated here
            capture_moves: ChessMoveList::new(), //move lists should only be instansiated here

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
            let target = bit_boards::pop_LSB(&mut targets);
            self.legal_moves.add_no_alloc(origin_square as u8, target as u8, flag);
        }
    }

    //like generate_moves but does not add moves moving into attacked square
    fn generate_moves_for_king(&mut self, origin_square: usize, mut targets: u64, flag: u8) {
        while targets != 0 {
            let target = bit_boards::pop_LSB(&mut targets);
            if self.attackers(target) != 0 {
                continue;
            }
            self.legal_moves.add_no_alloc(origin_square as u8, target as u8, flag);
        }
    }

    //generates legal pawn moves for pawns on 2nd rank
    fn legal_pawn_on_2nd(&mut self, mut pawns: u64, mask:u64){
        while pawns != 0 {
            let pawn = bit_boards::pop_LSB(&mut pawns);
            let normal_captures = bit_boards::PAWN_CAPTURES[self.to_move][pawn] & self.color_mask[self.other];
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
            let normal_moves = normal_captures | ((1 << normal_move) & !(self.color_mask[self.to_move] | self.color_mask[self.other]));

            self.generate_moves_target_masked(pawn, normal_moves, 0b1111, mask);
            if (1 << normal_move) & (self.color_mask[self.to_move] | self.color_mask[self.other]) == 0{
                let double_move = (1 << (if self.to_move == WHITE {pawn+16} else {pawn-16})) & !(self.color_mask[self.to_move] | self.color_mask[self.other]);
                self.generate_moves_target_masked(pawn, double_move, 0b1010, mask);
            }
        }
    }

    //generates legal pawn moves for pawns on 7th rank
    fn legal_pawn_on_7th(&mut self, mut pawns: u64, mask:u64){
        while pawns != 0 {
            let pawn = bit_boards::pop_LSB(&mut pawns);
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
            let pawn = bit_boards::pop_LSB(&mut pawns);
            let capture_targets = bit_boards::PAWN_CAPTURES[self.to_move][pawn];
            let normal_captures = capture_targets & self.color_mask[self.other];
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
            let normal_moves = normal_captures | ((1 << normal_move) & !(self.color_mask[self.to_move] | self.color_mask[self.other]));
            self.generate_moves_target_masked(pawn, normal_moves, 0b1111, mask);


            if self.en_passant_possible{
                let en_passant_captures = capture_targets & (1 << self.en_passant_square) & mask;
                //TODO: guarded en_passant add
            }
            let normal_move = if self.to_move == WHITE {pawn+8} else {pawn-8};
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

            let knight = bit_boards::pop_LSB(&mut knights);

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
        //TODO: find index of closes checkline square
        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        let moves_bb = bit_boards::KING_MOVES[king_pos] & !(self.color_mask[self.to_move]);

        let towards_check = moves_bb & self.check_line; //should return the one and only move that will move towards the checking piece 
        let offset = u64::trailing_zeros(towards_check) as i32 - king_pos as i32; //return the offset of towards_check move
        let opposite_offset = king_pos as i32 - offset; //uses the offset to calculate position of away_from_check

        let away_from_check:u64 = 1 << opposite_offset;
        self.generate_moves_for_king(king_pos, moves_bb & !away_from_check, 0b1111);


        /*NOTE: 
        no check for validity of away_from_check as a king move,
        if it is not valid, then no such move exists and the map has no effect on the moves
        */
    }

    //Calculate all pseudo legal rook moves, on squares provided by "rooks"
    fn legal_rook_moves(&mut self, mut rooks:u64, mask:u64){
        while rooks != 0 {
            let rook_pos =bit_boards::pop_LSB(&mut rooks);

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
            let bishop_pos =bit_boards::pop_LSB(&mut bishops);

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
            let queen_pos =bit_boards::pop_LSB(&mut queens);

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
                    self.check_line = rookmoves;
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
                    self.check_line = bishopmoves;
                }
            }
        }
    }

    fn attackers(&mut self, pos: usize) -> u64{

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

    pub fn gen_moves_legal(&mut self, board_state:&BoardState) -> ChessMoveList{
        self.legal_moves.reset();
        self.setup_state(board_state);

        let king_pos = u64::trailing_zeros(self.piece_bb[self.to_move][KING]) as usize;
        if king_pos >= 64 {
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
                    self.legal_moves(!self.checkers); //happens if knight check, in which case non king moves must capture the knight
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

        return self.legal_moves;
    }
    
    //TODO filter from gen_moves_legal for now 
    pub fn gen_moves_checks(&mut self, board_state:&BoardState) -> ChessMoveList{
        self.check_moves.reset();
        self.setup_state(board_state);
        return self.check_moves;
    }
    //TODO filter from gen_moves_legal
    pub fn gen_moves_captures(&mut self, board_state:&BoardState) -> ChessMoveList{
        self.check_moves.reset();
        self.setup_state(board_state);
        return self.capture_moves;
    }
}



impl Clone for MoveGen{
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
            check_moves: ChessMoveList::new(), //move lists should only be instansiated here
            capture_moves: ChessMoveList::new(), //move lists should only be instansiated here

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

impl Copy for MoveGen{

}