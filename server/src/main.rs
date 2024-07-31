use tokio;
use tokio::sync::mpsc;

mod bullet;
mod client;
mod entity;
mod game;
mod ray;
mod room;
mod slice;
mod user;

const ADDR: &'static str = "127.0.0.1:3000";

#[tokio::main]
async fn main() {

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(&ADDR).await.expect("tcp listeniner bind");

    println!("Listening on:\n{}\n", ADDR);

    let (
        send_to_game, 
        receive_from_client
    ) = mpsc::channel(game::MAX_PLAYERS);

    tokio::spawn(game::Game::init(receive_from_client));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(client::Client::init(stream, send_to_game.clone()));
    }

}