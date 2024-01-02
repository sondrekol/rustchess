

#[cfg(test)]
mod tests{
    use crate::game_manager::{bot2::Bot2, bot::Bot, board2::BoardState};

    #[test]
    fn bot_bench(){
        let fens = [
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
                "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "];
        let mut bot2 = Bot2::new();
        for fen in fens{
                bot2.get_move(BoardState::new_from_fen(fen));
        }
    }
}