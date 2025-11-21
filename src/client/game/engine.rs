

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

use crate::client::game::engine::eval::{game_state, is_check};


#[cfg(test)]
mod state_bitboard_tests;


pub struct GetMoveResult{
    chess_move: ChessMove,
    searched_positions: usize,
    eval: i32,
    depth_reached: u32,
    max_depth_reached: usize,
}

impl GetMoveResult{
    pub fn new(chess_move:ChessMove, searched_positions:usize, eval:i32, depth_reached: u32, max_depth_reached: usize) -> Self{
        Self { chess_move: chess_move, searched_positions: searched_positions, eval: eval, depth_reached: depth_reached, max_depth_reached: max_depth_reached }
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

    pub fn max_depth_reached(&self) -> usize{
        return self.max_depth_reached;
    }
}

pub struct Engine{
    search_depth: i64, //initial search depth, in ply, can be increased with extensions
    max_depth: usize, //maximum search depth, in ply, hard limit, even if search is extended
    num_pos: usize, //used to store the number of positions searched in the current search
    table: HashMap<BoardStateNumbers, Vec<(ChessMove, i32)>, BuildHasherDefault<FxHasher>>, //transposition table storing best moves for positions, from previous searches
    table_size: usize, //size of transposition table
    start_time: SystemTime, 
    average_best_move_placement: f64,
    average_best_move_index_placement: u64,
    search_stopped: bool,
    max_time: Option<u128>,
    max_depth_reached: usize,
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
            max_time: max_time,
            max_depth_reached: 0,
            
        }
    }
    
    pub fn get_move_bb(&mut self, board_state:BitBoardState, match_history:&mut Vec<BoardStateNumbers>) -> GetMoveResult{


        self.num_pos = 0;
        self.search_stopped = false;
        self.average_best_move_index_placement = 0;
        self.average_best_move_placement = 0.0;
        self.start_time = SystemTime::now();
        self.max_depth_reached = 0;


        let mut bit_board_state = board_state;
        let mut best_move:ChessMove = ChessMove::new_empty();
        let mut best_eval:i32 = 0;
        let mut depth = 0;
        let mut use_extensions = false;
        
        for i in 2..self.search_depth+1{
            
            if i >= 5 {
                use_extensions = true;
            }

            depth = i as u32;

            let last_board_state = match_history.pop().unwrap();
            let search_result = self.search(&mut bit_board_state, i, i32::MIN, i32::MAX, 0, true, match_history, use_extensions);
            match_history.push(last_board_state);

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
            depth,
            self.max_depth_reached
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
        
        let mut moves = bit_board_state.gen_moves_legal();


        if let Some(capture_square) = opt_capture_square{
            moves = moves.retain(|m|{
                m.target() == capture_square && is_capture(bit_board_state, m)
            });
        }else{
            moves = moves.retain(|m|{
                is_capture(bit_board_state, m)
            });
        }

        let this_eval = self.evaluate(bit_board_state);
        //if there are no more captures available, return the evaluation
        if moves.size() == 0 {
            return this_eval;
        }



        moves.sort(|m|{
            -capture_score(bit_board_state, m)
        });

        //capture search works on the assumption that a player does not need to make a capture
        let mut min = this_eval;
        let mut max = this_eval;

        for i in 0..moves.size(){
            let &capture = moves.get_mut(i);

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
            if max > alpha && bit_board_state.white_to_move(){
                alpha = max;
            }
            if min < beta && !bit_board_state.white_to_move(){
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

    fn search(&mut self, bit_board_state:&mut BitBoardState, depth:i64, mut alpha:i32, mut beta:i32, true_depth:usize, _first: bool, match_history:&mut Vec<BoardStateNumbers>, use_extensions: bool) -> (i32, ChessMove){

        // ! cancel search once depth is reached
        if depth <= 0 || true_depth >= self.max_depth{
            return (self.capture_search(bit_board_state, alpha, beta, 0, None), ChessMove::new_empty());
        }

        // ! Statistics
        if true_depth > self.max_depth_reached{
            self.max_depth_reached = true_depth;
        }
        let mut move_placement = 0;
        let mut best_move_placement: f64 = 0.0;

        // ! check for mate, stalemate or if still playing, uses number of legal moves to determine state
        match game_state(bit_board_state, match_history){
            GameState::Black => {return (-10000, ChessMove::new_empty())}
            GameState::White => {return (10000, ChessMove::new_empty())}
            GameState::Draw => {return (0, ChessMove::new_empty())}
            GameState::Playing => {}
        }
    


        // ! retrive moves in position
        let mut moves = bit_board_state.gen_moves_legal().moves_vec();
        let board_state_numbers = bit_board_state.board_state_numbers();
        let previous_best_moves = self.table.get(&board_state_numbers);
        for i in 0..moves.len(){
            promising_move(bit_board_state, &mut moves[i], previous_best_moves);
        }

        self.table.insert(board_state_numbers, Vec::<(ChessMove, i32)>::new());

        moves.sort_unstable_by(|a, b| 
                a.promising_level()
                .cmp(&b.promising_level())
            );
        if bit_board_state.white_to_move() {
            moves.reverse();
        }
        
        // ! best eval/move variables
        let mut min:i32 = i32::MAX;
        let mut max:i32 = i32::MIN;
        let mut min_move:ChessMove = *moves.get(0).unwrap();
        let mut max_move:ChessMove = *moves.get(0).unwrap();

        // ! indices
        let move_count = moves.len() as f64;
        let mut cur_move_index = 0;
        let total_moves = moves.len();

        for &chess_move in moves.iter(){

            // ! extensions and reductions
            let mut extension = 0;
            if use_extensions{
                if cur_move_index <= 2 && true_depth < 3{ //add extensions for the most promising moves
                    extension = 1;
                    if chess_move.promising_level().abs() >= 1000{//extra extension if this was calculated as best move previously
                        extension = 2;
                    }
                }
                if is_check(bit_board_state, &chess_move) && true_depth < 8{
                    extension+=1;
                }
                else if total_moves - cur_move_index < 10 && true_depth < 2{
                    extension = -1;
                }
            }
            


            // ! recursive search call
            let mut result = self.search(&mut bit_board_state.perform_move(chess_move), depth-1+extension, alpha, beta, true_depth +1, false, match_history, use_extensions);    
            

            // ! check for time limit exceeded
            // ! a little bit costly, find som other way of doing this maybe
            if let Some(max_time) = self.max_time{
                if self.start_time.elapsed().unwrap().as_millis() > max_time{
                    self.search_stopped = true;
                    break;
                }
            }

            // ! makes sure that the bot choses the fastest checkmate available
            if result.0 >= 1000 {
                result.0 -= 1;
            }else if result.0 <= -1000{
                result.0 += 1;
            }

            // ! update best move/eval
            // ! DRY
            if result.0 >= max{
                
                if !(result.0 == 0 && max > -30){//dont go for draw in a roughly equal position
                    max = result.0;
                    max_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                    self.table.get_mut(&board_state_numbers).unwrap().push((max_move, max));

                }
                
            }
            if result.0 <= min{
                if !(result.0 == 0 && min < 30){//dont go for draw in a roughly equal position
                    min = result.0;
                    min_move = chess_move;
                    best_move_placement = move_placement as f64/move_count;
                    self.table.get_mut(&board_state_numbers).unwrap().push((min_move, min));

                }

            }

            // ! alpha-beta pruning
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

            // ! statistics
            move_placement += 1;
            cur_move_index += 1;
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
            max_time: self.max_time,
            max_depth_reached: 0,
        }
    }
}