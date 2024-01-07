#[cfg(test)]
mod tests{
    use std::{time::SystemTime, fs::File, io::Write};

    use crate::game_manager::{bot2::Bot2, bot::{Bot, GetMoveResult}, board2::{BoardState, GameState, ChessMove, W_CASTLE_KING, PROMOTE_TO_BISHOP, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, PROMOTE_TO_KNIGHT, PROMOTE_TO_ROOK}, state_bitboard::{BitBoardState, bit_boards, BoardStateNumbers}};

    fn move_string_short(chess_move:&ChessMove) -> String{
        return format!("{}{} f({})", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag()); 
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

    fn lan_move(chess_move:ChessMove) -> String{
        match chess_move.flag() {
            W_CASTLE_KING => {
                return "e1g1".to_string();
            }
            W_CASTLE_QUEEN => {
                return "e1c1".to_string();
            }
            B_CASTLE_KING => {
                return "e8g8".to_string();
            }
            B_CASTLE_QUEEN => {
                return "e8c8".to_string();
            }
            PROMOTE_TO_BISHOP => {
                return format!("{}{}b", string_square(chess_move.origin()), string_square(chess_move.target()));
            }
            PROMOTE_TO_KNIGHT => {
                return format!("{}{}n", string_square(chess_move.origin()), string_square(chess_move.target()));
            }
            PROMOTE_TO_ROOK => {
                return format!("{}{}r", string_square(chess_move.origin()), string_square(chess_move.target()));
            }
            _ => {
                return format!("{}{}", string_square(chess_move.origin()), string_square(chess_move.target()));
            }
        }
    }

    fn play_match<T: Bot + Clone>(mut bot_white: T, mut bot_black: T) -> (GameState, String){

        let mut game_string = String::new();
        let mut board_state = BitBoardState::new();
        board_state.board_setup(&BoardState::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));

        let mut match_history = Vec::<BoardStateNumbers>::new();
        while board_state.game_state() == GameState::Playing {

            if board_state.white_to_move() {
                let results = bot_white.get_move_bb(board_state, &mut match_history);
                match_history.push(board_state.board_state_numbers());
                if results.chess_move().is_null() {
                    return (board_state.game_state(), game_string);
                }

                board_state = board_state.perform_move(*results.chess_move());
                bot_white = bot_white.clone();
                println!("white moved: {}, eval: {}", move_string_short(results.chess_move()), results.eval() as f64/ 100.0);
                game_string.push_str(lan_move(*results.chess_move()).as_str())
            }else {
                let results = bot_black.get_move_bb(board_state, &mut match_history);
                match_history.push(board_state.board_state_numbers());
                if results.chess_move().is_null() {
                    return (board_state.game_state(), game_string);
                }

                board_state = board_state.perform_move(*results.chess_move());
                bot_black = bot_black.clone();
                println!("black moved: {}, eval: {}", move_string_short(results.chess_move()), results.eval() as f64/ 100.0);
                game_string.push_str(lan_move(*results.chess_move()).as_str())
            }
            game_string.push('\n');
            if match_history.len() > 300{
                return (board_state.game_state(), game_string);
            }
        }
        return (board_state.game_state(), game_string);

    }

    #[test]
    fn bot_comparer(){
        bit_boards::populate_rook_moves();
        bit_boards::populate_bishop_moves();
        let mut bot1 = Bot2::new(7, 9, 1000000, Some(2000));
        let mut bot2 = Bot2::new(7, 9, 1000000, Some(2000));

        let (result, game_string) = play_match(bot1, bot2);
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


    
}