use licheszter::client::Licheszter;
use futures_util::StreamExt;
use licheszter::models::board::Event;
use licheszter::models::chat::ChatRoom;



use std::env;

mod game;



async fn handle_event(event: Event, client: &Licheszter) {
    match event {
        Event::GameStart { game } => {
            // Handle game start event
            println!("Game started with session ID: {}", game.id);
            println!("against players: {:?}", game.opponent.username);
            let game = game::Game::new(game.id);
            game.game_handler().await
        },
        Event::Challenge { challenge } => {
            // Handle challenge event
            println!("Received challenge from: {}", challenge.challenger.id);
            if challenge.challenger.id != "sondrekol" {
                client.challenge_decline(&challenge.id, None).await.unwrap();
                return;
            }
            
            client.challenge_accept(&challenge.id).await.unwrap();
            client.bot_chat_write(&challenge.id, ChatRoom::Player, "Lets gooo!")
                .await
                .unwrap();

            //client.bot_play_move(&challenge.id, "e2e4", false).await.unwrap();

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