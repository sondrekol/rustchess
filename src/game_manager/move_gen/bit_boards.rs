


pub const RANK_1:u64 = 0x00000000000000FF;
pub const RANK_2:u64 = RANK_1 << 8;
pub const RANK_3:u64 = RANK_1 << 16;
pub const RANK_4:u64 = RANK_1 << 24;
pub const RANK_5:u64 = RANK_1 << 32;
pub const RANK_6:u64 = RANK_1 << 40;
pub const RANK_7:u64 = RANK_1 << 48;
pub const RANK_8:u64 = RANK_1 << 56;

pub const FILE_A:u64 = 0x0101010101010101;
pub const FILE_B:u64 = FILE_A << 1;
pub const FILE_C:u64 = FILE_A << 2;
pub const FILE_D:u64 = FILE_A << 3;
pub const FILE_E:u64 = FILE_A << 4;
pub const FILE_F:u64 = FILE_A << 5;
pub const FILE_G:u64 = FILE_A << 6;
pub const FILE_H:u64 = FILE_A << 7;

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


//ROOK
const fn rook_attack_map_for_square(square: i32) -> u64{
    let mut map:u64 = 0;
    let mut i:i32 = 1;
    while i < 8{
        let north = square+8*i;
        let south = square-8*i;
        let east = square+1*i;
        let west = square-1*i;
        if north >= 0 && north < 64 && north%8 == square%8{
            map |= 1 << north;
        }
        if south >= 0 && south < 64 && south%8 == square%8{
            map |= 1 << south;
        }
        if east >= 0 && east < 64 && east/8 == square%8{
            map |= 1 << east;
        }
        if west >= 0 && west < 64 && west/8 == square%8{
            map |= 1 << west;
        }
        i+=1;
    }
    map
}

pub const ROOK_ATTACK_MAPS:[u64; 64] = {
    let mut rook_attack_maps:[u64; 64] = [0; 64];
    let mut i = 0;
    while i < 64{
        rook_attack_maps[i] = rook_attack_map_for_square(i as i32);
        i+=1;
    }
    rook_attack_maps
};

pub struct RookMoves{

}
impl RookMoves{
    pub fn mov_map(pos:usize, blockers:u64) -> u64{
        return 0; //TODO
    }
}

//BISHOP
const fn bishop_attack_map_for_square(square:i32) -> u64{
    let mut map:u64 = 0;
    let mut i = 1;
    while i < 8{
        let northeast = square+9*i;
        let northewest = square+7*i;
        let southeast = square-7*i;
        let southwest = square-9*i;
        if (northeast%8 - square%8).abs() == (northeast/8 - square/8).abs() && northeast >= 0 && northeast < 64{
            map |= 1 << northeast;
        }
        if (northewest%8 - square%8).abs() == (northewest/8 - square/8).abs() && northewest >= 0 && northewest < 64{
            map |= 1 << northewest;
        }
        if (southeast%8 - square%8).abs() == (southeast/8 - square/8).abs() && southeast >= 0 && southeast < 64{
            map |= 1 << southeast;
        }
        if (southwest%8 - square%8).abs() == (southwest/8 - square/8).abs() && southwest >= 0 && southwest < 64{
            map |= 1 << southwest;
        }
        i+=1;
    }
    map
}

pub const BISHOP_ATTACK_MAPS:[u64; 64] = {
    let mut bishop_attack_map:[u64; 64] = [0; 64];
    let mut i = 0;
    while i < 64{
        bishop_attack_map[i] = bishop_attack_map_for_square(i as i32);
        i+=1;
    }
    bishop_attack_map
};

pub struct BishopMoves{

}
impl BishopMoves{
    pub fn mov_map(pos:usize, blockers:u64) -> u64{
        return 0; //TODO
    }
}

//KING
const fn king_move(king_pos: i32, offset: i32) -> u64{
    //check for out of bounds
    if king_pos+offset < 0 || king_pos+offset >= 64{
        return 0;
    }

    //check for wraparound
    if (king_pos%8 -king_pos+offset%8).abs() > 1 || (king_pos/8 - king_pos+offset/8).abs() > 1{
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
    if (pawn_pos%8 -pawn_pos+offset%8).abs() != 1 || (pawn_pos/8 -pawn_pos+offset/8).abs() != 1{
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
    if (knight_pos%8 -knight_pos+offset%8).abs() + (knight_pos/8 -knight_pos+offset/8).abs() != 3{
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
pub fn pop_LSB(d_word:&mut u64) -> usize{
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

