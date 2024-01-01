
mod bit_ops;
use crate::board_bitboard::bit_ops::pop_LSB;

use self::bit_ops::seperate_bits;

#[derive(PartialEq, Eq)]
enum Piece{
    pawn,
    knight,
    bishop,
    rook,
    queen,
    king
}

pub struct Move{
    origin_square: u64,
    target_square: u64,
    piece: Piece
} 

struct BitBoards{
    white: [u64; 6],
    black: [u64; 6]
}

pub struct BoardState{
    bit_boards:BitBoards,
    white_to_move: bool,
    white_lost_castle_k: bool,
    white_lost_castle_q: bool,
    black_lost_castle_k: bool,
    black_lost_castle_q: bool,
    attack_maps: AttackMaps
}



pub struct AttackMaps{
    white_pawn: [u64; 64],
    black_pawn: [u64; 64],
    knight: [u64; 64],
    bishop: [u64; 64],
    rook: [u64; 64],
    queen: [u64; 64],
    king: [u64; 64],
}


impl AttackMaps{
    pub fn new()-> Self{
        let mut white_pawn: [u64; 64] = [0; 64];
        let mut black_pawn: [u64; 64] = [0; 64];
        let mut knight: [u64; 64] = [0; 64];
        let mut bishop: [u64; 64] = [0; 64];
        let mut rook: [u64; 64] = [0; 64];
        let mut queen: [u64; 64] = [0; 64];
        let mut king: [u64; 64] = [0; 64];

        //white pawn
        for i in 8..56{
            white_pawn[i] = 1 << ((i) + 8);
            if i < 16 {
                white_pawn[i] |= 1 << ((i) + 16);
            }
        }

        //black pawn
        for i in 8..56{
            black_pawn[i] = 1 << ((i) - 8);
            if i > 48 {
                black_pawn[i] |= 1 << ((i) - 16);
            }
        }

        //bishop
        for i in 0..64{
            let mut attack_map:u64 = 0;

            for k in 1..8{
                attack_map |= ((0x1 as u64) << i).checked_shl(7*k).unwrap_or(0);
                attack_map |= ((0x1 as u64) << i).checked_shl(9*k).unwrap_or(0);
                attack_map |= ((0x1 as u64) << i).checked_shr(7*k).unwrap_or(0);
                attack_map |= ((0x1 as u64) << i).checked_shr(9*k).unwrap_or(0);
            }
            bishop[i] = attack_map;
        }

        Self { 
            white_pawn: white_pawn, 
            black_pawn: black_pawn, 
            knight: knight, 
            bishop: bishop, 
            rook: rook, 
            queen: queen, 
            king: king, 
         }
    }

}

impl BoardState{

    pub fn new() -> Self{
        Self{
            bit_boards: BitBoards{
                white: [0;6],
                black: [0;6]
            },
            white_to_move: true,
            white_lost_castle_k: false,
            white_lost_castle_q: false,
            black_lost_castle_k: false,
            black_lost_castle_q: false,
            attack_maps: AttackMaps::new()

        }
    }

    pub fn new_from_graphic(board_graphic_state:&[[i8; 8]; 8]) -> Self{

        let mut bit_boards_new:BitBoards = BitBoards{
            white: [0;6],
            black: [0;6]
        };

        for i in 0..8{
            for j in 0..8{
                let piece = board_graphic_state[i][j];
                if piece == 0{continue}
                if piece > 0 {
                    bit_boards_new.white[(piece-1) as usize] |= 1 << (63-(i*8+j));
                }else{
                    bit_boards_new.black[(-piece-1) as usize] |= 1 << (63-(i*8+j));
                }
            }
        }
        Self{
            bit_boards: bit_boards_new,
            white_to_move: true,
            white_lost_castle_k: false,
            white_lost_castle_q: false,
            black_lost_castle_k: false,
            black_lost_castle_q: false,
            attack_maps: AttackMaps::new()

        }
    }

    pub fn get_board(&self) -> [[i8; 8];8]{
        let mut result:[[i8; 8];8] = [[0; 8];8];
        for i in 0..8{
            for j in 0..8{
                for k in 0..6{
                    result[i][j] += ((self.bit_boards.white[k] >> 63-(i*8+j)) as i8 & 0x1) *(1+k as i8);
                    result[i][j] += ((self.bit_boards.black[k] >> 63-(i*8+j)) as i8 & 0x1) *(-1-k as i8);
                }
                
            }
        }
        return result;
    }

    pub fn is_legal_position(board_state:BoardState) -> bool{

        //Check for overlapping pieces
        let mut overlap:u64 = 0;
        for k in 0..6{
            overlap &= board_state.bit_boards.white[k] & board_state.bit_boards.black[k];
        }
        if overlap!=0 {
            return false;
        }


        //check if kings are present
        if board_state.bit_boards.white[5] == 0 || board_state.bit_boards.black[5] == 0{
            return false
        }





        return true;
    }

    pub fn is_in_check(){

    }


    fn pawn_moves(){

    }


    pub fn psuedo_legal_moves(&mut self) -> Vec<Move>{
        let mut result = Vec::<Move>::new();

        let mut full_white_bit_board:u64 = 0;
        let mut full_black_bit_board:u64 = 0;
        let white_to_move = self.white_to_move;

        for k in 0..6{
            full_white_bit_board |= self.bit_boards.white[k];
            full_black_bit_board |= self.bit_boards.black[k];
        }
        for k in 0..6{
            let mut remaining_pieces = if white_to_move {self.bit_boards.white[k]} else {self.bit_boards.black[k]};
            while remaining_pieces != 0{
                let index = pop_LSB(&mut remaining_pieces);

                
                
                let u64moves =
                if k==0{
                    if white_to_move {self.attack_maps.white_pawn[index]} else {self.attack_maps.black_pawn[index]}
                }else if k==1{
                    self.attack_maps.knight[index]
                }else if k==2{
                    self.attack_maps.bishop[index]
                }else if k==3{
                    self.attack_maps.rook[index]
                }else if k==4{
                    self.attack_maps.queen[index]
                }else{
                    self.attack_maps.king[index]
                };
                
                for u64move in seperate_bits(&u64moves){
                    result.push(Move { 
                        origin_square: 0x1 << index,
                        target_square: u64move, 
                        piece: if k==0{
                                    Piece::pawn
                                }else if k==1{
                                    Piece::knight
                                }else if k==2{
                                    Piece::bishop
                                }else if k==3{
                                    Piece::rook
                                }else if k==4{
                                    Piece::queen
                                }else{
                                    Piece::king
                                } })
                }

            }
        }

        


        return result;
    }

    fn perform_move(&mut self, chess_move:&Move){
        let mut piece_type = 0;
        match  chess_move.piece{
            Piece::pawn => {piece_type = 0;}
            Piece::knight => {piece_type = 1;}
            Piece::bishop => {piece_type = 2;}
            Piece::rook => {piece_type = 3;}
            Piece::queen => {piece_type = 4;}
            Piece::king => {piece_type = 5;}
        }
        if(move_in_movelist(chess_move, &self.psuedo_legal_moves())){
            if(self.white_to_move){
                self.bit_boards.white[piece_type] ^= chess_move.origin_square;
                self.bit_boards.white[piece_type] |= chess_move.target_square;
                for i in 0..6{
                    self.bit_boards.black[i] &= !chess_move.target_square;
                }
            }else{
                self.bit_boards.black[piece_type] ^= chess_move.origin_square;
                self.bit_boards.black[piece_type] |= chess_move.target_square;
                for i in 0..6{
                    self.bit_boards.white[i] &= !chess_move.target_square;
                }
            }
            self.white_to_move = !self.white_to_move;
        }
    }

    pub fn perform_move_api(&mut self, origin:(usize, usize), target:(usize, usize)){
        let origin_u64:u64 = 1 << (63-(origin.0*8+origin.1));
        let target_u64:u64 = 1 << (63-(target.0*8+target.1));
        self.perform_move(
            &Move { 
                origin_square: origin_u64, 
                target_square: target_u64, 
                piece: Piece::pawn }
        )
    }
    

}

fn move_in_movelist(chess_move:&Move, moves:&Vec<Move>) -> bool{
    for m in moves{
        if
            m.origin_square == chess_move.origin_square &&
            m.target_square == chess_move.target_square &&
            m.piece == chess_move.piece
        {
            return true;
        }
    }
    return false;
}
