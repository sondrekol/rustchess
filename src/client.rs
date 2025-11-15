use licheszter::{client::Licheszter, config::challenges::ChallengeOptions};
use futures_util::StreamExt;
use licheszter::models::board::Event;



use std::env;
mod game;

static BOTS: [&str; 3] = ["GarboBot", "Jibbby", "halcyonbot"];


async fn attempt_challenge(client:& Licheszter) {
    let options = ChallengeOptions::new()
        .rated(true)
        .clock(900, 10);


    loop{
        //let mut bot_stream = client.bots_online(3).await.unwrap();
        for bot in BOTS.iter() {
    
            println!("Found online bot: {}", *bot);
            let bot_i = rand::random_range(0..3);
            if bot_i != 0 {continue;} 
            
            
            match client.challenge_create(*bot, Some(&options)).await {
                Ok(challenge) => {
                    println!("Challenge sent to {} with ID: {}", *bot, challenge.id);
                }
                Err(e) => {
                    eprintln!("Failed to send challenge: {}", e);
                    continue;
                }
            }
            return;
        }
    }
    

    
}

async fn handle_event(event: Event, client: &Licheszter) -> u8{
    match event {
        Event::GameStart { game } => {

            println!("Game started with session ID: {} \n 
                    against players: {:?}", 
                    game.id, game.opponent.username);

            let game = game::Game::new(game.id);

            //TODO: handle in seperate thread
            game.game_handler().await;
            return 1;
        },
        Event::Challenge { challenge } => {
            if challenge.challenger.id == "sonkolbot" {
                //challenge from self, ignore
                return 1;
            }
            println!("Received challenge from: {}", challenge.challenger.id);
            /*if challenge.challenger.id != "sondrekol" {
                client.challenge_decline(&challenge.id, None).await.unwrap();
                return;
            }*/
            
            client.challenge_accept(&challenge.id).await.unwrap();
            return 1;
        },
        Event::GameFinish { game } => {
            println!("Game finished with ID: {}", game.id);
            return 0;
        },
        Event::ChallengeCanceled { challenge } => {
            println!("Challenge canceled with ID: {}", challenge.id);
            return 1;
        },
        _ => {return 1;}
    }
}

pub async fn li_bot() {

    game::engine::state_bitboard::bit_boards::populate_rook_moves();
    game::engine::state_bitboard::bit_boards::populate_bishop_moves();

    match dotenvy::dotenv().ok() {
        Some(path) => println!("Loaded .env file {}", path.display()),
        None => println!(".env file not found, proceeding without it"),
    }
    let key = env::var("LICHESS_API_KEY").unwrap();

    let client = Licheszter::builder()
        .with_authentication(key)
        .build();

    
    loop{
        attempt_challenge(&client).await;
        
        let mut events = client.connect().await.unwrap();
        while let Some(result) = events.next().await {
            match result {
                Ok(event) => {
                    if handle_event(event, &client).await == 0 {
                        break;
                    }
                },
                Err(e) => eprintln!("Error receiving event: {:?}", e),
            }
        }

    }
}