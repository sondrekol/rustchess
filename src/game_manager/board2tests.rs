


#[cfg(test)]
mod tests {
    use crate::game_manager::board2::{BoardState, HORIZONTAL_DISTANCE, VERTICAL_DISTANCE};

    #[test]
    fn number_of_legal_moves() {

        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "];
        let expected = [20, 6, 44, 46, 14];

        for i in 0..5{
            let fen = fens[i];
            let mut pos = BoardState::new_from_fen(fen);
            let legal_moves_debug = pos.legal_moves_debug();
            let legal_move_count = pos.legal_move_count();
            if(legal_move_count != expected[i]){
                println!("debug");
            }
            assert_eq!(legal_move_count, expected[i], "failed for {fen}");
        }
    }


    #[test]
    fn mutable_move(){
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "];
        for i in 0..5{
            let fen = fens[i];
            let mut pos = BoardState::new_from_fen(fen);
            let copy = pos.clone();
            let moves = pos.legal_moves().moves_vec();
            for chess_move in moves{
                let origin = chess_move.origin();
                let target = chess_move.target();
                let flag = chess_move.flag();


                let captured_piece = pos.piece(chess_move.target() as usize);
                let castle_rights = pos.castle_rights();
                pos.perform_move_mutable(chess_move);
                pos.undo_move_mutable(chess_move);
                pos.set_piece(chess_move.target() as usize, captured_piece);
                pos.set_castle_rights(castle_rights);
                let is_equal = BoardState::equal_game_state(&pos, &copy);
                if !is_equal {
                    println!("wrong move");
                }
                assert!(is_equal);
            }
        }
    }
    #[test]
    fn test_distances(){
        let hor = HORIZONTAL_DISTANCE;
        let ver = VERTICAL_DISTANCE;
        for i in 0..64{
            for j in 0..64{
                assert_eq!(BoardState::horizontal_distance(i, j), HORIZONTAL_DISTANCE[i as usize][j as usize] as u8, "Horizontal failed for {i}, {j}");
                assert_eq!(BoardState::vertical_distance(i, j), VERTICAL_DISTANCE[i as usize][j as usize] as u8, "Vertical failed for {i}, {j}");
            }
        }
    }
}