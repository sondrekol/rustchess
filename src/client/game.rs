
use licheszter::models::chat::ChatRoom;
use std::env;
use std::time::Duration;

use futures::StreamExt;
use licheszter::{client::Licheszter, models::game::GameStatus};
use licheszter::models::board::{BoardState};

mod engine;
use engine::state_bitboard::{BitBoardState, BoardStateNumbers};
use engine::board;

use crate::client::game::engine::GetMoveResult;
use crate::client::game::engine::move_string::lan_move;

pub struct Game{
    game_id: String
}

const BOT_NAME:&str = "sonkolbot";
const TABLE_SIZE:usize = 1000000;
const SEARCH_DEPTH:i64 = 10;
const MAX_DEPTH:usize = 20;
const MAX_TIME:Option<u128> = Some(500);
const STARTING_POS:&str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/*
game handler for a specific game
*/


fn log_search(move_result: GetMoveResult){
    let chess_move = move_result.chess_move();
    let uci_move = lan_move(*chess_move);
    println!("Move: {:>1} | eval: {:>6} | depth: {:>1} | nodes: {:>9}",
        uci_move,
        move_result.eval(),
        move_result.depth_reached(),
        move_result.num_pos()
    );
}

impl Game{
    pub fn new(game_id: String) -> Self {
        Self { game_id }
    }

    async fn play_move(&self, client:&Licheszter, bot: &engine::Engine, bb_state:&BitBoardState, game_history:&mut Vec<BoardStateNumbers>) {
        let search_result = bot.clone().get_move_bb(*bb_state, game_history);
        let uci_move = lan_move(*search_result.chess_move());
        log_search(search_result);
        
        for attempt in 0..3 {
            match client.bot_play_move(&self.game_id, &uci_move, false).await {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to send move (attempt {}): {}", attempt + 1, e);
                    if attempt == 2 {
                        eprintln!("Giving up sending move {}", uci_move);
                    } else {
                        tokio::time::sleep(Duration::from_millis(500 * (attempt + 1))).await;
                    }
                }
            }
        }
    }

    pub async fn game_handler(&self) {
        // Example: Fetch and print the current state of the game
        
        dotenvy::dotenv().ok();
        let key = env::var("LICHESS_API_KEY").unwrap();

        let client = Licheszter::builder()
            .with_authentication(key)
            .build();
        
        let mut game_events = client.bot_game_connect(&self.game_id).await.unwrap();

        //assuming that previous line indicates that the game has started
        let bot = engine::Engine::new(SEARCH_DEPTH, MAX_DEPTH, TABLE_SIZE, MAX_TIME);
        let mut board_state = board::BoardState::new_from_fen(STARTING_POS);
        let mut game_history = Vec::<BoardStateNumbers>::new();

        let mut bot_color = 1;

        engine::state_bitboard::bit_boards::populate_rook_moves();
        engine::state_bitboard::bit_boards::populate_bishop_moves();

        while let Some(result) = game_events.next().await {
            match result {
                Ok(event)=>{
                    match event {
                        BoardState::GameState ( game_state ) => {
                            match game_state.status {
                                GameStatus::Mate | GameStatus::Resign | GameStatus::Timeout | GameStatus::Draw=> {
                                    println!("Game ended with status: {:?}", game_state.status);
                                    return;
                                },
                                _ => {}
                            }

                            //find the last move played
                            let moves = game_state.moves.split_whitespace().collect::<Vec<&str>>();
                            let last_move = *moves.last().unwrap();
                            let chess_move = board::ChessMove::from_uci(last_move, &board_state);

                            //update local board
                            board_state.perform_move(chess_move);

                            //copy BoardState to BitBoardState and append to history
                            let mut bb_state = BitBoardState::new();
                            bb_state.setup_state(&board_state);
                            game_history.push(bb_state.board_state_numbers());


                            if moves.len() % 2 == bot_color {
                                //it is bots turn, last move was opponents
                                self.play_move(&client, &bot, &bb_state, &mut game_history).await;
                            }
                            else{
                                println!("Opponent played move: {}, interpreted as {}, raw = {:x}", last_move, lan_move(chess_move), chess_move.move_data());
                            }
                        },
                        BoardState::GameFull(game_state) => {

                            let mut bb_state = BitBoardState::new();
                            bb_state.setup_state(&board_state);

                            game_history.push(bb_state.board_state_numbers());

                            if game_state.white.name == BOT_NAME {
                                bot_color = 0;
                                self.play_move(&client, &bot, &bb_state, &mut game_history).await;
                            }
                        },
                        BoardState::ChatLine(chat)=>{
                            if chat.username == BOT_NAME {
                                continue;
                            }
                            

                            client.bot_chat_write(&self.game_id, ChatRoom::Player, &chat.text).await.unwrap();
                        }
                        _ => {}
                    }
                }
                
                Err(e)=>{
                    println!("Error in game stream: {}", e);
                }
            }
        }


    }
}