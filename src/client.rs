use licheszter::client::Licheszter;
use futures_util::StreamExt;
use licheszter::models::board::Event;
use licheszter::models::chat::ChatRoom;


use std::fs;

mod game;



async fn handle_event(event: Event, client: &Licheszter) {
    match event {
        Event::GameStart { game } => {
            // Handle game start event
            println!("Game started with session ID: {}", game.id);
            println!("against players: {:?}", game.opponent.username);
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
            let game = game::Game::new(challenge.id.to_string());
            game.game_handler().await

        },
        _ => {}
    }
}

pub async fn li_bot() {
    // Create a new instance of Licheszter with your account token

    let key = fs::read_to_string("lichess_api_key.txt")
        .expect("Failed to read API key from file")
        .trim()
        .to_string();

    let client = Licheszter::builder()
        .with_authentication(key)
        .build();

    println!("{}", client.account_email().await.unwrap().email);

    // Use the client to fetch online bots, for example...

    // ...or open the event stream

    // *Should not be mutable
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