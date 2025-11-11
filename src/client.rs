use licheszter::client::Licheszter;
use futures_util::StreamExt;
use licheszter::models::board::Event;



use std::env;

mod game;



async fn handle_event(event: Event, client: &Licheszter) {
    match event {
        Event::GameStart { game } => {

            println!("Game started with session ID: {} \n 
                    against players: {:?}", 
                    game.id, game.opponent.username);

            let game = game::Game::new(game.id);

            //TODO: handle in seperate thread
            game.game_handler().await

        },
        Event::Challenge { challenge } => {
            println!("Received challenge from: {}", challenge.challenger.id);
            if challenge.challenger.id != "sondrekol" {
                client.challenge_decline(&challenge.id, None).await.unwrap();
                return;
            }
            
            client.challenge_accept(&challenge.id).await.unwrap();

        },
        _ => {}
    }
}

pub async fn li_bot() {


    match dotenvy::dotenv().ok() {
        Some(path) => println!("Loaded .env file {}", path.display()),
        None => println!(".env file not found, proceeding without it"),
    }
    let key = env::var("LICHESS_API_KEY").unwrap();

    let client = Licheszter::builder()
        .with_authentication(key)
        .build();

    let mut events = client.connect().await.unwrap();
    
    while let Some(result) = events.next().await {
        match result {
            Ok(event) => {
                handle_event(event, &client).await;
            },
            Err(e) => eprintln!("Error receiving event: {:?}", e),
        }
    }
}