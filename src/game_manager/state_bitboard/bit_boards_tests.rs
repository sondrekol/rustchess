


#[cfg(test)]
mod tests {
    use crate::game_manager::state_bitboard::bit_boards::{self, RookMoves, populate_bishop_moves, populate_rook_moves};


    #[test]
    fn pop_lsb_test(){
        let mut test:[u64; 3] = [0b100010010000000100010001,
                             0b10010000100100100000,
                             0b0010000101010101000011000];
        let expect:[u64; 3] = [ 0b100010010000000100010000,
                                0b10010000100100000000,
                                0b0010000101010101000010000];
        assert_eq!(0, bit_boards::pop_lsb(&mut test[0]));
        assert_eq!(5, bit_boards::pop_lsb(&mut test[1]));
        assert_eq!(3, bit_boards::pop_lsb(&mut test[2]));
        for i in 0..3{
            assert_eq!(test[i], expect[i]);
        }

    }

    #[test]
    fn direction_test(){
        let expected_north:[u64; 4] = [
            0x0101010101010101,
            0x0404040404040400,
            0x1010101010100000,
            0x4040404040000000,
        ];
        let expected_south:[u64; 4] = [
            0x0000000000000001,
            0x0000000000000404,
            0x0000000000101010,
            0x0000000040404040,
        ];
        let expected_east:[u64; 4] = [
            0x00000000000000FF,
            0x000000000000FC00,
            0x0000000000F00000,
            0x00000000C0000000,
        ];
        let expected_west:[u64; 4] = [
            0x0000000000000001,
            0x0000000000000700,
            0x00000000001F0000,
            0x000000007F000000,
        ];

        for i in 0..4{
            assert_eq!(expected_east[i], bit_boards::east(i*10), "failed for east: {}", i*10);
            assert_eq!(expected_west[i], bit_boards::west(i*10), "failed for west: {}", i*10);
            assert_eq!(expected_north[i], bit_boards::north(i*10), "failed for north: {}", i*10);
            assert_eq!(expected_south[i], bit_boards::south(i*10), "failed for south: {}", i*10);
        }
    }

    //Use https://tearth.dev/bitboard-viewer/ to check results
    #[test]
    fn moves_for_rook_test(){
        populate_rook_moves();
        let rook_pos = 0;
        let blockers = 0xffff00000000ffff;
        assert_eq!(RookMoves::mov_map(rook_pos, blockers), 0x102);

        let rook_pos:usize = 0;
        assert_eq!(RookMoves::mov_map(rook_pos, 0), 0x1010101010101fe);

        let rook_pos:usize = 1;
        assert_eq!(RookMoves::mov_map(rook_pos, 0), 0x2020202020202fd);

        let rook_pos:usize = 32;
        let blockers:u64 = 0x20481c0000000;
        assert_eq!(RookMoves::mov_map(rook_pos, blockers), 0x10101fe01010101);
        
    }
    #[test]
    fn moves_for_bishop_test(){
        populate_bishop_moves();
    }
}