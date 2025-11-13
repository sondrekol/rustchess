

pub(crate) mod board;
pub(crate) mod state_bitboard;
pub(crate) mod move_string;
mod search;
mod eval;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::time::SystemTime;

extern crate fxhash;
use eval::{capture_score, evaluate, is_capture, promising_move};
use fxhash::FxHasher;

use board::{ChessMove, GameState};
use state_bitboard::{BitBoardState, BoardStateNumbers};

// !add conditional compilation for tests
#[cfg(test)]
mod state_bitboard_tests;


pub struct GetMoveResult{
    chess_move: ChessMove,
    searched_positions: usize,
    eval: i32,
    depth_reached: u32,
}

impl GetMoveResult{
    pub fn new(chess_move:ChessMove, searched_positions:usize, eval:i32, depth_reached: u32) -> Self{
        Self { chess_move: chess_move, searched_positions: searched_positions, eval: eval, depth_reached: depth_reached}
    }

    pub fn chess_move(&self) -> &ChessMove{
        &self.chess_move
    }

    pub fn num_pos(&self) -> usize{
        self.searched_positions
    }
    pub fn eval(&self) -> i32{
        self.eval
    }

    pub fn depth_reached(&self) -> u32{
        return self.depth_reached;
    }
}

pub struct Engine{
    search_depth: i64,
    max_depth: usize,
    num_pos: usize,
    table: HashMap<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>,
    table_size: usize,
    start_time: SystemTime,
    average_best_move_placement: f64,
    average_best_move_index_placement: u64,
    search_stopped: bool,
    max_time: Option<u128>
}


impl Engine{

    pub fn new(search_depth: i64, max_depth: usize, table_size: usize, max_time: Option<u128>) -> Self{
        Self{
            search_depth: search_depth,
            max_depth: max_depth,
            num_pos: 0,
            table: HashMap::<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>::default(),
            table_size: table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false,
            max_time: max_time
            
        }
    }
    
    pub fn get_move_bb(&mut self, board_state:BitBoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult{
        self.start_time = SystemTime::now();

        let mut bit_board_state = board_state;
        let mut best_move:ChessMove = ChessMove::new_empty();
        let mut best_eval:i32 = 0;
        let mut depth = 0;
        
        for i in 2..self.search_depth+1{
            depth = i as u32;
            self.num_pos = 0;
            let search_result = self.search(&mut bit_board_state, i, i32::MIN, i32::MAX, 0, true, match_history);
            //self.table.clear();
            if self.search_stopped {
                break;
            }
            best_move = search_result.1;
            if search_result.0 < 30000 && search_result.0 > -30000 {//if depth stopped before calculating the evaluation of the best move, use the previous
                best_eval = search_result.0;
            }
        }

        return GetMoveResult::new(
            best_move,
            self.num_pos,
            best_eval,
            depth
        );
    }

    fn evaluate(&mut self, bit_board_state:&BitBoardState) -> i32{
        self.num_pos += 1;
        return evaluate(bit_board_state);
    }

    //finishes the search by looking at any captures in a position, and subsequent "capture-backs" on the same square
    //all nodes are evaluated, a node is evaluated as the min/max of its children and itself (works on the assumption that there is a non capturing move)
    fn capture_search(&mut self, bit_board_state:&mut BitBoardState, mut alpha:i32, mut beta:i32, capture_depth:usize, opt_capture_square:Option<u8>) -> i32{

        //Not directly related to piece count but should work
        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return -1000}
            GameState::White => {return 1000}
            GameState::Draw => {return 0}
            GameState::Playing => {}
        }

        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        moves.retain(|m|{
            is_capture(bit_board_state, m)
        });
        //moves should only contain captures at this point


        //after initial capture, only check if can capture back, dont check any potential "danger level captures"
        if let Some(capture_square) = opt_capture_square{
            moves.retain(|m|{
                m.target() == capture_square
            });
        }
        let this_eval = self.evaluate(bit_board_state);
        //if there are no more captures available, return the piece count
        if moves.len() == 0 {
            return this_eval;
        }

        moves.sort_by(|a, b| 
            capture_score(bit_board_state, a)
            .cmp(&capture_score(bit_board_state, b))
            .reverse()
            );

        //at worst either player can choose to not capture
        let mut min = this_eval;
        let mut max = this_eval;


        for capture in moves{

            let mut result = self.capture_search(&mut bit_board_state.perform_move(capture), alpha, beta, capture_depth+1, Some(capture.target()));

            if result >= 900 {
                result -= 1;
            }
            if result <= -900{
                result += 1;
            }
            if result > max {
                max = result;
            }
            if result < min{
                min = result;
            }
            if max > alpha{
                alpha = max;
            }
            if min < beta{
                beta = min;
            }
            if alpha > beta{
                break;
            }
        }

        if bit_board_state.white_to_move(){
            return max;
        }else{
            return min;
        }
    }

    fn search(&mut self, bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, _first: bool, match_history:&mut Vec<BoardStateNumbers>) -> (i32, ChessMove){



        let game_state = bit_board_state.game_state();
        match game_state{
            GameState::Black => {return (-10000, ChessMove::new_empty())}
            GameState::White => {return (10000, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }


        let board_state_numbers = bit_board_state.board_state_numbers();

        //check for draw by repetition
        if match_history.iter().filter(|&n| *n == board_state_numbers).count() == 2{
            return (0, ChessMove::new_empty()); 
        }


        match_history.push(board_state_numbers);
        

        if depth <= 0 || true_depth >= self.max_depth{
            match_history.pop();
            return (self.capture_search(bit_board_state, alpha, beta, 0, None), ChessMove::new_empty());
        }
        let mut moves = bit_board_state.gen_moves_legal().moves_vec();

        
        

        let previous_best_moves = self.table.get(&board_state_numbers);

        //add promising level to the moves for later sorting
        for i in 0..moves.len(){
            promising_move(bit_board_state, &mut moves[i], previous_best_moves);
        }

        self.table.insert(board_state_numbers, Vec::<(ChessMove, i32)>::new());
        
        //at this point previous_best_moves_mut should contain an empty vec

        //Sort moves by how promising they are
        moves.sort_unstable_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
            );
        if bit_board_state.white_to_move() {
            moves.reverse();
        }
        
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;
        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        let mut move_placement = 0;
        let mut best_move_placement: f64 = 0.0;

        let move_count = moves.len() as f64;
        for chess_move in moves{

            //Maybe maybe not
            let extension = 0;

            /*if self.is_check(bit_board_state, &chess_move) && depth == 1{
                extension += 1;
            }*/
 
            let mut result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history);    
            
            /*if true_depth == 0 {
                result.0 += rand::random_range(-2..2); //add some randomness to top level moves to avoid always playing the same move in equal positions
            }*/

            if let Some(max_time) = self.max_time{
                if self.start_time.elapsed().unwrap().as_millis() > max_time{
                    self.search_stopped = true;
                    break;
                }
            }

            if result.0 >= 1000 {
                result.0 -= 1;
            }else if result.0 <= -1000{
                result.0 += 1;
            }

            if result.0 >= max{
                
                if !(result.0 == 0 && max > -30){//dont go for draw in a roughly equal position
                    max = result.0;
                    max_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                    
                    //replace or add best move
                    let best_moves = self.table.get_mut(&board_state_numbers).unwrap();
                    let mut found_move: bool = false;
                    for i in 0..best_moves.len(){
                        if max_move == best_moves[i].0{
                            best_moves[i].1 = max;
                            found_move = true;
                            break;
                        }
                    }
                    if !found_move{
                        self.table.get_mut(&board_state_numbers).unwrap().push((max_move, max));
                    }
                }
                
            }
            if result.0 <= min{
                if !(result.0 == 0 && min < 30){//dont go for draw in a roughly equal position
                    min = result.0;
                    min_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;

                    //replace or add best move
                    let best_moves = self.table.get_mut(&board_state_numbers).unwrap();
                    let mut found_move: bool = false;
                    for i in 0..best_moves.len(){
                        if min_move == best_moves[i].0{
                            best_moves[i].1 = min;
                            found_move = true;
                            break;
                        }
                    }
                    if !found_move{
                        self.table.get_mut(&board_state_numbers).unwrap().push((min_move, min));
                    }
                }

            }

            if bit_board_state.white_to_move() {
                if max > alpha {
                    alpha = max;
                }
            }else {
                if min < beta{
                    beta = min;
                }
            }
            if alpha > beta{
                break;
            }

            move_placement += 1;
        }

        self.average_best_move_index_placement += 1;
        self.average_best_move_placement += (best_move_placement as f64 - self.average_best_move_placement)/self.average_best_move_index_placement as f64;

        match_history.pop();
        if bit_board_state.white_to_move(){
            return (max, max_move);
        }else{
            return (min, min_move);
        }
    }
}


impl Clone for Engine{
    fn clone(&self) -> Self {
        Self {  
            search_depth: self.search_depth,
            max_depth: self.max_depth,
            num_pos: self.num_pos,
            table: HashMap::<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>::default(),
            table_size: self.table_size,
            start_time: SystemTime::now(),
            average_best_move_placement: 0.0,
            average_best_move_index_placement: 0,
            search_stopped: false,
            max_time: self.max_time
        }
    }
}