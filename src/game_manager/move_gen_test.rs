


#[cfg(test)]
mod tests {
    use std::str::Bytes;

    use crate::game_manager::board2::BoardState;
    use crate::game_manager::move_gen::MoveGen;

    fn string_square(square:u8) -> String{
        let mut str = "".to_owned();
        match square%8{
            0 => {str.push_str("a")}
            1 => {str.push_str("b")}
            2 => {str.push_str("c")}
            3 => {str.push_str("d")}
            4 => {str.push_str("e")}
            5 => {str.push_str("f")}
            6 => {str.push_str("g")}
            7 => {str.push_str("h")}
            _ => {}
        }
        match square/8{
            0 => {str.push_str("1")}
            1 => {str.push_str("2")}
            2 => {str.push_str("3")}
            3 => {str.push_str("4")}
            4 => {str.push_str("5")}
            5 => {str.push_str("6")}
            6 => {str.push_str("7")}
            7 => {str.push_str("8")}
            _ => {}
        }
        return str;
    }
    

    #[test]
    fn get_moves_test(){
        let board_state = BoardState::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut move_generator = MoveGen::new();
        let moves = move_generator.gen_moves_legal(&board_state);

        println!("eyy")
    }

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
            let pos = BoardState::new_from_fen(fen);
            let mut move_generator = MoveGen::new();
            let chess_move_list = move_generator.gen_moves_legal(&pos);
            let moves = chess_move_list.moves_vec();
            let legal_move_count = moves.len();
            if(legal_move_count != expected[i]){
                println!("debug");
                for chess_move in moves{
                    println!("move: origin: {}, target: {}, flag: {}", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag());
                }
            }
            assert_eq!(legal_move_count, expected[i], "failed for {fen}");
        }
    }

    #[test]
    fn number_of_legal_moves_against_old() {

        let fens = [
            "8/1PP1PK2/P2Bn3/3R4/1N3p2/1p5p/7Q/1nr2k2 w - - 0 1",
            "3b4/3Bn1BP/3N4/3P2k1/2K5/P6p/2P3Np/2q3r1 w - - 0 1",
            "k6q/4pp2/PP6/2bp1B2/Pr6/8/2P3K1/N1B1N3 w - - 0 1",
            "1B6/1P3N2/8/3b2bP/B2K4/P4ppp/2pk3P/1Q6 w - - 0 1",
            "2q5/b6p/PRp2P1K/8/p1P4P/2N1B2P/7P/7k w - - 0 1",
            "8/NPKp4/r2B2P1/5p2/1P6/pkn2qr1/2p1P3/8 w - - 0 1",
            "8/1Rp2Pq1/5BPP/Kp1P4/3R4/4p3/3Pr1P1/3k4 w - - 0 1",
            "2Q2K2/1p2PB2/NpP3rP/2Pp3b/2p5/1kP1P1B1/p3R2p/2b3rq w - - 0 1",
            "1N1b4/R1p2bPp/Kn3p2/3kP3/3P4/2Q1BP2/pP1PP1q1/4R1nr w - - 0 1",
            "3b4/3PP1pb/P2pK1B1/Nr3p2/2PpPP1N/1R3Q1r/P4p2/k3n3 w - - 0 1",
            "bBk5/p1Pn1N1P/P2p4/1B3Pr1/n4Rpp/1K3p2/1b1Pp3/4q1r1 w - - 0 1", 
            "1Nk2q2/Kpp2pPr/1P1bP2P/3PP3/PnRPnpp1/2rQ3p/bNpp1R2/4BB2 w - - 0 1",
            "1Nq4r/3PPpRp/r1P1nK1p/2P1p1PQ/2pp1B1p/N1Rb2pB/2PP1kP1/b4n2 w - - 0 1"
            ];

        for fen in fens{
            let mut pos = BoardState::new_from_fen(fen);
            let mut move_generator = MoveGen::new();
            let chess_move_list = move_generator.gen_moves_legal(&pos);
            let moves = chess_move_list.moves_vec();
            let legal_move_count = moves.len();
            let expected = pos.legal_move_count();
            if legal_move_count != expected {
                println!("debug");
                for chess_move in moves{
                    println!("move: origin: {}, target: {}, flag: {}", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag());
                }
            }
            assert_eq!(legal_move_count, expected, "failed for {fen}");
        }
    }


    #[test]
    fn print_history(){
        let history:[u16; 5] = [63213, 62212, 63259, 61708, 61724];
        for mov in history{
            println!("move: origin: {}, target: {}, flag: {}", string_square((mov&0b111111)as u8), string_square(((mov >> 6)&0b111111)as u8), ((mov >> 12)&0b111111)as u8);
        }
        
    }
}