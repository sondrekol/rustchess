#[cfg(test)]
mod tests {

    use std::time::SystemTime;

    use board::{BoardState, ChessMove};
    use crate::client::game::engine::{board, state_bitboard::{BitBoardState, bit_boards}};

    fn setup_sliding_magics(){
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
    }

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
        setup_sliding_magics();
        let board_state = BoardState::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        bit_board_state.gen_moves_legal();
    }

    #[test]
    fn number_of_legal_moves() {
        setup_sliding_magics();
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "];
        let expected = [20, 44, 6, 46, 14];

        for i in 0..5{
            let fen = fens[i];
            let pos = BoardState::new_from_fen(fen);
            let mut bit_board_state = BitBoardState::new();
            bit_board_state.setup_state(&pos);
            let chess_move_list = bit_board_state.gen_moves_legal();
            let moves = chess_move_list.moves_vec();
            let legal_move_count = moves.len();
            if legal_move_count != expected[i] {
                println!("debug");
                for chess_move in moves{
                    println!("move: origin: {}, target: {}, flag: {}", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag());
                }
            }
            assert_eq!(legal_move_count, expected[i], "failed for {fen}");
        }
    }


    fn perft(bit_board_state:&mut BitBoardState, depth:usize) -> usize{
        if depth == 0{
            return 0;
        }
        else if depth == 1 {
            let number_of_moves = bit_board_state.gen_moves_legal().size();
            return number_of_moves;
        }
        else {
            let mut sum = 0;
            for chess_move in bit_board_state.gen_moves_legal().moves_vec(){
                sum += perft(&mut bit_board_state.perform_move(chess_move), depth-1);
            }
            return sum;
        }
    }

    #[test]
    fn perft_start_pos(){
        setup_sliding_magics();
        let board_state = BoardState::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        let start = SystemTime::now();
        assert_eq!(perft(&mut bit_board_state, 1), 20);
        assert_eq!(perft(&mut bit_board_state, 2), 400);
        assert_eq!(perft(&mut bit_board_state, 3), 8902);
        assert_eq!(perft(&mut bit_board_state, 4), 197281);
        assert_eq!(perft(&mut bit_board_state, 5), 4865609);
        assert_eq!(perft(&mut bit_board_state, 6), 119060324);
        assert_eq!(perft(&mut bit_board_state, 7), 3195901860);
        //assert_eq!(perft(&mut bit_board_state, 8), 84998978956);

        println!("perft used {} ms", start.elapsed().unwrap().as_millis());

    }

    #[test]
    fn perft_extras(){
        setup_sliding_magics();
        let board_state = BoardState::new_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        assert_eq!(perft(&mut bit_board_state, 1), 48);
        assert_eq!(perft(&mut bit_board_state, 2), 2039);
        assert_eq!(perft(&mut bit_board_state, 3), 97862);
        assert_eq!(perft(&mut bit_board_state, 4), 4085603);
        assert_eq!(perft(&mut bit_board_state, 5), 193690690);
        assert_eq!(perft(&mut bit_board_state, 6), 8031647685);

        let board_state = BoardState::new_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        assert_eq!(perft(&mut bit_board_state, 1), 14);
        assert_eq!(perft(&mut bit_board_state, 2), 191);
        assert_eq!(perft(&mut bit_board_state, 3), 2812);
        assert_eq!(perft(&mut bit_board_state, 4), 43238);
        assert_eq!(perft(&mut bit_board_state, 5), 674624);

        

        let board_state = BoardState::new_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        assert_eq!(perft(&mut bit_board_state, 1), 6);
        assert_eq!(perft(&mut bit_board_state, 2), 264);
        assert_eq!(perft(&mut bit_board_state, 3), 9467);
        assert_eq!(perft(&mut bit_board_state, 4), 422333);
        assert_eq!(perft(&mut bit_board_state, 5), 15833292);
        assert_eq!(perft(&mut bit_board_state, 6), 706045033);

        let board_state = BoardState::new_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8 ");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);
        assert_eq!(perft(&mut bit_board_state, 1), 44);
        assert_eq!(perft(&mut bit_board_state, 2), 1486);
        assert_eq!(perft(&mut bit_board_state, 3), 62379);
        assert_eq!(perft(&mut bit_board_state, 4), 2103487);
        assert_eq!(perft(&mut bit_board_state, 5), 89941194);

    }


    fn perft_verbose(bit_board_state:&mut BitBoardState, depth:usize) -> usize{
        if depth == 0{
            return 0;
        }
        else if depth == 1 {
            let number_of_moves = bit_board_state.gen_moves_legal().size();
            return number_of_moves;
        }
        else {
            let mut sum = 0;
            for chess_move in bit_board_state.gen_moves_legal().moves_vec(){
                println!("{}: ", move_string_short(&chess_move));
                let number_of_moves = perft(&mut bit_board_state.perform_move(chess_move), depth-1);
                println!("{}", number_of_moves);
                sum += number_of_moves;

            }
            return sum;
        }
    }

    #[test]
    fn perft_verbose_test(){
        setup_sliding_magics();
        let board_state = BoardState::new_from_fen("8/4k3/3P4/1N6/4R3/8/3K4/8 b - - 0 1");
        let mut bit_board_state = BitBoardState::new();
        bit_board_state.setup_state(&board_state);

        assert_eq!(perft_verbose(&mut bit_board_state, 3), 531);// !doesnt match
    }



    fn move_string_short(chess_move:&ChessMove) -> String{
        return format!("{}{} f({})", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag()); 
    }

}