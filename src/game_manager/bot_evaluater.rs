#[cfg(test)]
#[allow(dead_code)]
mod test{
    use std::{fs::File, io::Write};

    use crate::game_manager::bot2_3::Bot2_3;
    use crate::game_manager::bot2_4::Bot2_4;
    use crate::game_manager::bot2_5::Bot2_5;
    use crate::game_manager::bot2_6::Bot2_6;
    use crate::game_manager::{bot2::Bot2, bot::{Bot, GetMoveResult}, board2::{BoardState, GameState, ChessMove, W_CASTLE_KING, PROMOTE_TO_BISHOP, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, PROMOTE_TO_KNIGHT, PROMOTE_TO_ROOK}, state_bitboard::{BitBoardState, bit_boards, BoardStateNumbers}, bot2_2::Bot2_2, move_string::{lan_move, move_string_short}};

    fn play_match<T1: Bot + Clone, T2: Bot + Clone>(mut bot_white: T1, mut bot_black: T2, fen: &str) -> (GameState, String){

        let mut game_string = String::new();
        let mut board_state = BitBoardState::new();
        board_state.board_setup(&BoardState::new_from_fen(fen));

        let mut match_history = Vec::<BoardStateNumbers>::new();
        while board_state.game_state() == GameState::Playing {
            if board_state.white_to_move() {
                let results = bot_white.get_move_bb(board_state, &mut match_history);
                match_history.push(board_state.board_state_numbers());
                if results.chess_move().is_null() {
                    println!("recieved null move");
                    return (board_state.game_state(), game_string);
                }

                board_state = board_state.perform_move(*results.chess_move());
                bot_white = bot_white.clone();
                //println!("{}: white moved: {}, eval: {}", ply/2 + 1, move_string_short(results.chess_move()), results.eval() as f64/ 100.0);
                game_string.push_str(lan_move(*results.chess_move()).as_str())
            }else {
                let results = bot_black.get_move_bb(board_state, &mut match_history);
                match_history.push(board_state.board_state_numbers());
                if results.chess_move().is_null() {
                    println!("recieved null move");
                    return (board_state.game_state(), game_string);
                }

                board_state = board_state.perform_move(*results.chess_move());
                bot_black = bot_black.clone();
                //println!("{}: black moved: {}, eval: {}", ply/2 + 1, move_string_short(results.chess_move()), results.eval() as f64/ 100.0);
                game_string.push_str(lan_move(*results.chess_move()).as_str())
            }
            game_string.push('\n');
            if match_history.len() > 300{
                return (board_state.game_state(), game_string);
            }
            if match_history.iter().filter(|&n| *n == board_state.board_state_numbers()).count() == 3{
                return (GameState::Draw, game_string);
            }
        }
        return (board_state.game_state(), game_string);

    }

    #[test]
    fn bot_match(){
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
        let mut bot1 = Bot2_3::new(15, 20, 1000000, Some(100));
        let mut bot2 = Bot2_2::new(15, 20, 1000000, Some(100));

        let (result, game_string) = play_match(bot2, bot1, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        match result {
            GameState::Black => {
                println!("black won");
            }
            GameState::White => {
                println!("white won");
            }
            GameState::Draw => {
                println!("draw");
            }
            GameState::Playing => {
                println!("match aborted");
            }
        }
        let mut file = File::create("game_result.txt").unwrap();
        file.write_all(game_string.as_bytes());
        
    }


    fn print_wins(bot1:i32, draws:i32, bot2:i32, pos_nr:usize){
        println!("bot1 {}/{}/{} bot2 -- pos:{}", bot1, draws, bot2, pos_nr);
    }
    #[test]
    fn bot_comparer(){
        let fens = [
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
                "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
                "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
                "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R b KQkq - 3 3",
                "rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2",
                "rnbqkb1r/pp1ppppp/5n2/2p5/3P4/5N2/PPP1PPPP/RNBQKB1R w KQkq - 0 3",
                "rnbqkb1r/pppppp1p/5np1/8/3P4/5N2/PPP1PPPP/RNBQKB1R w KQkq - 0 3",
                "rnbqkbnr/pp3ppp/4p3/1Npp4/3P1B2/8/PPP1PPPP/R2QKBNR b KQkq - 1 4",
                "rnbqk1nr/ppppppbp/6p1/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3",
                "rnbqkb1r/pp1p1ppp/4pn2/2p5/3P1B2/2P2N2/PP2PPPP/RN1QKB1R b KQkq - 0 4",
                "rnbqk2r/ppp1ppbp/5np1/3p4/2PP4/5NP1/PP2PP1P/RNBQKB1R w KQkq - 0 5",
                "r1bqkbnr/pppp1ppp/8/4n3/4PP2/8/PPP3PP/RNBQKBNR b KQkq - 0 4",
                "rnbqkbnr/p1p1pppp/8/1p6/2pPP3/5N2/PP3PPP/RNBQKB1R b KQkq - 1 4",
                "rnbqkbnr/pp3ppp/4p3/2pp4/3P4/4PN2/PPP2PPP/RNBQKB1R w KQkq - 0 4",
                "rnbqkbnr/pp2pppp/3p4/2p5/8/4PN2/PPPPBPPP/RNBQK2R b KQkq - 1 3",
                "r1bqkb1r/pp1p1ppp/4pn2/8/3P1B2/8/PP1NPPPP/R2QKB1R b KQkq - 0 7",
                "rn1qkb1r/pp2pppp/2p2n2/3p1b2/3P1B2/2N2P2/PPP1P1PP/R2QKBNR w KQkq - 0 5",
                "rnbq1rk1/pp1pppbp/2p2np1/8/3P4/5NP1/PPP1PPBP/RNBQ1RK1 w - - 0 6",
                "r1bqk1nr/pppp1pp1/2n4p/2b1p3/2B1P3/P1N2N2/1PPP1PPP/R1BQK2R b KQkq - 0 5",
                "rnbqkbnr/pppp1ppp/8/4p3/3PP3/8/PPP2PPP/RNBQKBNR b KQkq - 0 2",
                "rnbqkb1r/p1pp1ppp/1p2pn2/8/2PP4/5NP1/PP2PP1P/RNBQKB1R b KQkq - 0 4"];
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
        let bot1 = Bot2_6::new(15, 30, 1000000, Some(200));
        let bot2 = Bot2_5::new(15, 30, 1000000, Some(200));

        let mut bot1_wins = 0;
        let mut draws = 0;
        let mut bot2_wins = 0;
        for i in 0..fens.len(){
            let fen = fens[i];
            let (result, game_string) = play_match(bot2.clone(), bot1.clone(), fen);
            match result {
                GameState::Black => {
                    bot1_wins += 1;
                }
                GameState::White => {
                    bot2_wins += 1;
                }
                GameState::Draw => {
                    draws += 1;
                }
                GameState::Playing => {
                    draws += 1;
                }
            }
            print_wins(bot1_wins, draws, bot2_wins, i);
            let (result, game_string) = play_match(bot1.clone(), bot2.clone(), fen);
            match result {
                GameState::Black => {
                    bot2_wins += 1;
                }
                GameState::White => {
                    bot1_wins += 1;
                }
                GameState::Draw => {
                    draws += 1;
                }
                GameState::Playing => {
                    draws += 1;
                }
            }
            print_wins(bot1_wins, draws, bot2_wins, i);

        }
        println!("Bot1: {bot1_wins}/{draws}/{bot2_wins} Bot 2");
        
    }



    
}