use super::board2::{ChessMove, W_CASTLE_KING, W_CASTLE_QUEEN, B_CASTLE_KING, B_CASTLE_QUEEN, PROMOTE_TO_BISHOP, PROMOTE_TO_KNIGHT, PROMOTE_TO_ROOK};

pub fn move_string_short(chess_move:&ChessMove) -> String{
    return format!("{}{} f({})", string_square(chess_move.origin()), string_square(chess_move.target()), chess_move.flag()); 
}

pub fn lan_move(chess_move:ChessMove) -> String{
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

pub fn string_square(square:u8) -> String{
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