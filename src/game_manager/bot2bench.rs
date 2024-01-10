

#[cfg(test)]
mod tests{
    use std::time::SystemTime;

    use crate::game_manager::{bot2::Bot2, bot::{Bot, GetMoveResult}, board2::BoardState, state_bitboard::{BoardStateNumbers, BitBoardState, bit_boards}, bot2_2::Bot2_2, move_string::{lan_move, move_string_short}, bot2_3::Bot2_3, bot2_4::Bot2_4};

    #[test]
    fn bot_bench(){
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
        /*
        tests many differnet positions and averages the results
         */
        let fens = [
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
                "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
                "rn1qkbnr/ppp2ppp/4b3/4p3/3pP3/5NN1/PPPP1PPP/R1BQKB1R b KQkq - 3 5",
                "rnbqkbnr/pp1p1ppp/4p3/2p5/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 1 3",
                "rnbqk2r/pp2bppp/4pn2/3p2B1/3NP3/2N5/PPP2PPP/R2QKB1R w KQkq - 1 7",
                "r1bqk2r/ppp2ppp/2n2n2/2bpp3/2B1P3/2PP1N2/PP3PPP/RNBQK2R w KQkq - 0 6",
                "8/kpp5/p7/6pp/2N5/1P4P1/P4K2/8 w - - 0 1",
                "r1bqkb1r/5ppp/p1p1pn2/2p5/8/1P2PN2/P2PBPPP/RNBQK2R w KQkq - 0 8",
                "r1bq1rk1/5ppp/p1pbp3/2pn4/8/1P1P1N2/PB2BPPP/RN1QR1K1 w - - 5 13"];
        let num_fens = fens.len();
        let mut average_time = 0;
        let mut avg_best_move_index:f64 = 0.0;
        let mut avg_nodes_searched:f64 = 0.0;
        for fen in fens{
                let mut bot2 = Bot2_4::new(7, 18, 1000000, None);
                let start_time = SystemTime::now();

                let results:GetMoveResult = bot2.get_move(BoardState::new_from_fen(fen), &mut Vec::<BoardStateNumbers>::new());
                average_time += start_time.elapsed().unwrap().as_millis()/num_fens as u128;
                avg_best_move_index += results.avg_best_move_i()/num_fens as f64;
                avg_nodes_searched += results.num_pos() as f64/num_fens as f64;
                println!("finished: {}", fen);
       }
       println!();
       println!("--------------------RESULTS--------------------");
       println!(" average time for best move: {}", average_time);
       println!("average best move placement: {}", avg_best_move_index);
       println!("     average nodes searched: {}", avg_nodes_searched);
       println!("                       kN/s: {}", avg_nodes_searched/average_time as f64);
       println!("-----------------------------------------------");
       println!();
    }


    #[test]
    fn best_line_test(){
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let mut bot2 = Bot2::default();
        bot2.get_move(BoardState::new_from_fen(fen), &mut Vec::<BoardStateNumbers>::new());
    }


    #[test]
    fn bot_debug(){
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
        let fen = "r3k1nr/pbpp1p2/1p4p1/3Pp1q1/5P1p/1PN3P1/P1P1QPBP/2R1K1R1 b kq - 0 14";
        let mut bot = Bot2_2::new(3, 3, 1000000, Some(200));

        let mut bit_board_state = BitBoardState::new();
        let board_state = BoardState::new_from_fen(fen);
        bit_board_state.board_setup(&board_state);
        let results = bot.get_move_bb(bit_board_state, &mut Vec::<BoardStateNumbers>::new());
        println!("{}", move_string_short(results.chess_move()));
        println!("eval: {}", results.eval());
    }



    
}