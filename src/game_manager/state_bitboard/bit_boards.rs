use self::magics::{ROOK_MAP_SIZE, BISHOP_MAP_SIZE, ROOK_MAGICS, BISHOP_MAGICS, MagicEntry};

mod magics;


pub const RANK_1:u64 = 0x00000000000000FF;
pub const RANK_2:u64 = RANK_1 << 8;
pub const RANK_3:u64 = RANK_1 << 16;
pub const RANK_4:u64 = RANK_1 << 24;
pub const RANK_5:u64 = RANK_1 << 32;
pub const RANK_6:u64 = RANK_1 << 40;
pub const RANK_7:u64 = RANK_1 << 48;
pub const RANK_8:u64 = RANK_1 << 56;

pub const RANKS:[u64; 8] = [
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8
];

pub const FILE_A:u64 = 0x0101010101010101;
pub const FILE_B:u64 = FILE_A << 1;
pub const FILE_C:u64 = FILE_A << 2;
pub const FILE_D:u64 = FILE_A << 3;
pub const FILE_E:u64 = FILE_A << 4;
pub const FILE_F:u64 = FILE_A << 5;
pub const FILE_G:u64 = FILE_A << 6;
pub const FILE_H:u64 = FILE_A << 7;

pub const FILES:[u64; 8] = [
    FILE_A,
    FILE_B,
    FILE_C,
    FILE_D,
    FILE_E,
    FILE_F,
    FILE_E,
    FILE_G
];

pub const EAST_OF:[u64; 8] = [ //Indexed by file
    FILE_A | FILE_B | FILE_C | FILE_D | FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_B | FILE_C | FILE_D | FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_C | FILE_D | FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_D | FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_E | FILE_F | FILE_G | FILE_H,
    FILE_F | FILE_G | FILE_H,
    FILE_G | FILE_H,
    FILE_H,
];

pub const WEST_OF:[u64; 8] = [ //Indexed by file
    FILE_A,
    FILE_A | FILE_B,
    FILE_A | FILE_B | FILE_C,
    FILE_A | FILE_B | FILE_C | FILE_D,
    FILE_A | FILE_B | FILE_C | FILE_D | FILE_E,
    FILE_A | FILE_B | FILE_C | FILE_D | FILE_E | FILE_F,
    FILE_A | FILE_B | FILE_C | FILE_D | FILE_E | FILE_F | FILE_G,
    FILE_A | FILE_B | FILE_C | FILE_D | FILE_E | FILE_F | FILE_G | FILE_H,
];

pub const NEIGHBOUR_FILES:[u64; 8] = [
    FILE_B,
    FILE_A | FILE_C,
    FILE_B | FILE_D,
    FILE_C | FILE_E,
    FILE_D | FILE_F,
    FILE_E | FILE_G,
    FILE_F | FILE_H,
    FILE_G
];

pub const CASTLE_W_K_LINE:u64 = 0x0000000000000060;
pub const CASTLE_W_Q_LINE:u64 = 0x000000000000000E;
pub const CASTLE_B_K_LINE:u64 = 0x6000000000000000;
pub const CASTLE_B_Q_LINE:u64 = 0x0E00000000000000;



pub struct RookMoves{

}
impl RookMoves{
    pub fn generate_rook_moves(pos:usize, blockers:u64) -> u64{
        let pos = pos as i32;
        let mut map:u64 = 0;

        let north = 8;
        let south = -8;
        let east = 1;
        let west = -1;
        for direction in [north, south, west, east]{

            for i in 1..8{
                let new_square = pos+direction*i;
                if new_square < 0 || new_square >= 64{
                    break;
                }
                if pos%8 == new_square%8 || pos/8 == new_square/8{
                    map |= 1 << new_square;
                }
                if blockers & (1 << new_square) != 0{
                    break;
                }
            }
        }
        return map;
    }

    pub fn mov_map(pos:usize, blockers:u64) -> u64{// !temp until magics
        //return Self::generate_rook_moves(pos, blockers);
        unsafe{
            return ROOK_MOVES[magic_index(blockers, &magics::ROOK_MAGICS[pos])];
        }
    }
}





pub struct BishopMoves{

}
impl BishopMoves{
    pub fn generate_bishop_moves(pos:usize, blockers:u64) -> u64{
        let pos = pos as i32;
        let mut map:u64 = 0;

        let northeast = 9;
        let northewest = 7;
        let southeast = -7;
        let southwest = -9;
        for direction in [northeast, northewest, southeast, southwest]{
            for i in 1..8{
                let new_square = pos + direction*i;
                if new_square < 0 || new_square >= 64{
                    break;
                }
                if (new_square%8 - pos%8).abs() == (new_square/8 - pos/8).abs(){
                    map |= 1 << new_square;
                }
                if blockers & (1 << new_square) != 0{
                    break;
                }
            }

        }
        return map;
    }
    pub fn mov_map(pos:usize, blockers:u64) -> u64{// !TEMP UNTIL MAGICS
        //return Self::generate_bishop_moves(pos, blockers);
        unsafe{
            return BISHOP_MOVES[magic_index(blockers, &magics::BISHOP_MAGICS[pos])];
        }
    }


}

//KING
const fn king_move(king_pos: i32, offset: i32) -> u64{
    //check for out of bounds
    if king_pos+offset < 0 || king_pos+offset >= 64{
        return 0;
    }

    //check for wraparound
    if (king_pos%8 - (king_pos+offset)%8).abs() > 1 || (king_pos/8 - (king_pos+offset)/8).abs() > 1{
        return 0;
    }
    return 1 << king_pos + offset;
}


pub const KING_MOVES:[u64; 64] = {
    let mut king_moves:[u64; 64] = [0; 64];
    let mut i:i32 = 0;
    while i < 64{
        king_moves[i as usize] |= king_move(i, 1);
        king_moves[i as usize] |= king_move(i, -1);
        king_moves[i as usize] |= king_move(i, 7);
        king_moves[i as usize] |= king_move(i, 8);
        king_moves[i as usize] |= king_move(i, 9);
        king_moves[i as usize] |= king_move(i, -7);
        king_moves[i as usize] |= king_move(i, -8);
        king_moves[i as usize] |= king_move(i, -9);
        i+=1;
    }
    king_moves
};

//PAWN CAPTURES
const fn pawn_capture(pawn_pos: i32, offset: i32) -> u64{
    //check for out of bounds
    if pawn_pos+offset < 0 || pawn_pos+offset >= 64 {
        return 0;
    }

    //check for wraparound
    if (pawn_pos%8 -(pawn_pos+offset)%8).abs() != 1 || (pawn_pos/8 -(pawn_pos+offset)/8).abs() != 1{
        return 0;
    }

    return 1 << pawn_pos+offset;
}

pub const PAWN_CAPTURES:[[u64; 64]; 2] = {
    let mut pawn_captures:[[u64; 64]; 2] = [[0; 64]; 2];
    let mut i:i32 = 0;
    while i < 64{
        pawn_captures[1][i as usize] |= pawn_capture(i, 9);
        pawn_captures[1][i as usize] |= pawn_capture(i, 7);
        pawn_captures[0][i as usize] |= pawn_capture(i, -9);
        pawn_captures[0][i as usize] |= pawn_capture(i, -7);
        i+=1;
    }
    pawn_captures
};


//KNIGHT
const fn knight_move(knight_pos:i32, offset: i32) -> u64{
    //check for out of bounds
    if knight_pos+offset < 0 || knight_pos+offset >= 64 {
        return 0;
    }

    //check for wraparound
    if (knight_pos%8 -(knight_pos+offset)%8).abs() + (knight_pos/8 -(knight_pos+offset)/8).abs() != 3{
        return 0;
    }

    return 1 << knight_pos+offset;
}

pub const KNIGHT_MOVES:[u64; 64] = {
    let mut knight_moves:[u64; 64] = [0; 64];

    let mut i:i32 = 0;
    while i < 64{
        knight_moves[i as usize] |= knight_move(i, -17);
        knight_moves[i as usize] |= knight_move(i, -15);
        knight_moves[i as usize] |= knight_move(i, -10);
        knight_moves[i as usize] |= knight_move(i, -6);
        knight_moves[i as usize] |= knight_move(i, 17);
        knight_moves[i as usize] |= knight_move(i, 15);
        knight_moves[i as usize] |= knight_move(i, 10);
        knight_moves[i as usize] |= knight_move(i, 6);
        i+=1;
    }
    knight_moves
};







//help functions
pub fn pop_lsb(d_word:&mut u64) -> usize{
    let index = u64::trailing_zeros(*d_word);
    *d_word ^= 1 << index;
    return index as usize;
}

//returns a bitboard of all squares directly north of square
#[inline(always)]
pub fn north(square:usize) -> u64{
    FILE_A << square
}

//returns a bitboard of all squares directly south of square
#[inline(always)]
pub fn south(square:usize) -> u64{
    FILE_H >> 63-square
}

//returns a bitboard of all squares directly east of square
#[inline(always)]
pub fn east(square:usize) -> u64{
    (RANK_1 << square) & (RANK_1 << (square/8)*8)
}

//returns a bitboard of all squares directly west of square
#[inline(always)]
pub fn west(square:usize) -> u64{
    (RANK_8 >> 63-square) & (RANK_1 << (square/8)*8)
}

//returns a bitboard of all squares on the same rank or on a rank further north
#[inline(always)]
pub fn north_of(square:usize) -> u64{
    0xFFFFFFFFFFFFFFFF << (square/8)*8
}

//returns a bitboard of all squares on the same rank or on a rank further south
#[inline(always)]
pub fn south_of(square:usize) -> u64{
    0xFFFFFFFFFFFFFFFF >> 56-(square/8)*8
}

//returns a bitboard of all squares on the same file or on a file further east
#[inline(always)]
pub fn east_of(square:usize) -> u64{
    EAST_OF[square%8]
}

//returns a bitboard of all squares on the same file or on a file further west
#[inline(always)]
pub fn west_of(square:usize) -> u64{
    WEST_OF[square%8]
}

#[inline(always)]
pub fn file_of(square: usize) -> u64{
    return FILES[square%8];
}

#[inline(always)]
pub fn rank_of(square: usize) -> u64{
    return RANKS[square/8];
}




static mut ROOK_MOVES:[u64; ROOK_MAP_SIZE] = [0;ROOK_MAP_SIZE];


pub fn populate_rook_moves(){
    //Generate rook_moves
    for i in 0..64{
        let entry = &ROOK_MAGICS[i];
        let mut blockers = entry.mask + 1 ;
        loop {

            blockers = (blockers - 1) & entry.mask;
            let index = magic_index(blockers, entry);

            unsafe{
                ROOK_MOVES[index] = RookMoves::generate_rook_moves(i, blockers);
            }

            if blockers == 0 {
                break;
            }
        }

        //TODO loop trough all subsets of mask
    }
}

static mut BISHOP_MOVES:[u64; BISHOP_MAP_SIZE] = [0; BISHOP_MAP_SIZE];

pub fn populate_bishop_moves(){
    //Generate bishop moves
    for i in 0..64{
        let entry = &BISHOP_MAGICS[i];

        let mut blockers = entry.mask + 1 ;
        loop {

            blockers = (blockers - 1) & entry.mask;
            let index = magic_index(blockers, entry);

            unsafe{
                BISHOP_MOVES[index] = BishopMoves::generate_bishop_moves(i, blockers);
            }
            

            if blockers == 0 {
                break;
            }
        }
    }
}

#[inline(always)]
fn magic_index(blockers: u64, entry:&MagicEntry) -> usize{
    let mut hash = blockers & entry.mask;
    hash = hash.wrapping_mul(entry.magic) >> entry.shift;
    return hash as usize + entry.offset
}


pub const BOARD_CENTER:u64 = 0x00003c3c3c3c0000;


pub const TOP_TIER_PAWN:[u64; 2] = [
    0xe7001818000000,
    0x181800e700
];

pub const SEC_TIER_PAWN:[u64; 2] = [
    0xdb8100180000,
    0x180081db0000
];


pub const TOP_TIER_BISHOP:[u64; 2] = [
    0x8142241818244281,
    0x8142241818244281
];

pub const SEC_TIER_BISHOP:[u64; 2] = [
    0x24422424422400,
    0x24422424422400
];