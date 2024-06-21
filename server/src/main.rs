use tokio;
use tokio::sync::mpsc;
use futures_util::{ SinkExt, StreamExt };
use tokio_tungstenite;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
enum ClientMessage {
    Connect {
        send_to_client: mpsc::Sender<Vec<u8>>
    },
    Jump,
    WalkLeft,
    WalkRight,
}

struct Game {
    receive_from_client: mpsc::Receiver<ClientMessage>,
    // must use try_send to avoid deadlocks
    users: Vec<User>,
}

struct User {
    send_to_client: mpsc::Sender<Vec<u8>>,
    id: u8,
    x: u8,
    y: u8,
    width: u8,
    height: u8,
}

struct Client {
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    receive_from_game: mpsc::Receiver<Vec<u8>>,
    send_to_game: mpsc::Sender<ClientMessage>,
}

impl Game {

    async fn init(receive_from_client: mpsc::Receiver<ClientMessage>) {

        let mut game: Self = Self {
            receive_from_client,
            users: Vec::new(),
        };

        let mut timer: tokio::time::Interval = tokio::time::interval(tokio::time::Duration::from_millis(30));

        loop {
            // first one listens for client input, second one ticks on interval 
            tokio::select! {
                
                client_msg = game.receive_from_client.recv() => {

                    let client_msg: ClientMessage = match client_msg {
                        Some(client_msg) => client_msg,
                        None => {
                            println!("no client message found");
                            return;
                        }
                    };

                    println!("{:#?}", client_msg);

                    let id = 0;

                    match client_msg {
                        ClientMessage::Connect { send_to_client } => game.users.push(User::new(send_to_client)),
                        ClientMessage::Jump => (),
                        ClientMessage::WalkLeft => {
                            match game.users.iter_mut().find(|user| user.id == id) {
                                Some(user) => user.walk_left(),
                                None => continue,
                            }
                        },
                        ClientMessage::WalkRight => {
                            match game.users.iter_mut().find(|user| user.id == id) {
                                Some(user) => user.walk_right(),
                                None => continue,
                            }
                        },
                    }

                },

                _ = timer.tick() => {

                    if game.users.len() == 0 { 
                        continue;
                    }

                    let buf: Vec<u8> = game.create_render_buffer();

                    let mut idx: usize = 0;

                    while idx < game.users.len() - 1 {

                        let send_to_client = &game.users[idx].send_to_client;

                        match send_to_client.try_send(buf.clone()) {
                            Ok(_) => idx += 1,
                            Err(mpsc::error::TrySendError::Closed(_)) => {
                                // drop user + sender
                                let _ = game.users.remove(idx);
                                continue;
                            },
                            Err(err) => {
                                println!("failed to send render buffer: {:#?}", err);
                                return; 
                            }
                        }
                        
                    }

                    match game.users.last().unwrap().send_to_client.try_send(buf) {
                        Ok(_) => (),
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            // drop user + sender
                            let _ = game.users.remove(game.users.len() - 1);
                        },
                        Err(err) => {
                            println!("failed to send render buffer: {:#?}", err);
                            return; 
                        }
                    }

                },

            }
        }

    }

    fn create_render_buffer(&self) -> Vec<u8> {

        let mut buf: Vec<u8> = Vec::new();

        for user in &self.users {
            buf.push(0);
            buf.push(user.x);
            buf.push(user.y);
            buf.push(user.width);
            buf.push(user.height);
        }

        return buf;

    }

}

impl User {

    fn new(send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        Self {
            send_to_client,
            id: 0, // pass in id as parameter as uuid or smth else
            x: 0,
            y: 0,
            width: 10, 
            height: 10, 
        }

    }

    fn walk_left(&mut self) {
        self.x -= 5;
    }

    fn walk_right(&mut self) {
        self.x += 5; 
    }

}

impl Client {

    async fn init(stream: tokio::net::TcpStream, send_to_game: mpsc::Sender<ClientMessage>) {

        let ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = match tokio_tungstenite::accept_async(stream).await {
            Ok(ws) => ws,
            Err(_) => {
                println!("failed to connect to websocket");
                return;
            }
        };

        let (
            send_to_client, 
            receive_from_game,
        ) = mpsc::channel(100);

        let mut client: Self = Self {
            ws,
            receive_from_game,
            send_to_game,
        };

        if let Err(err) = client.send_to_game.send(ClientMessage::Connect { send_to_client, }).await {
            println!("failed to connect to game: {:#?}", err);
            return;
        }

        // use tokio select to create 2 tasks, one for passing on client messages (below), and one for listening for render commands from game
        loop {
            tokio::select! {

                buf = client.receive_from_game.recv() => {

                    let buf: Vec<u8> = match buf {
                        Some(buf) => buf,
                        None => {
                            println!("no render buffer found");
                            break;
                        }
                    };

                    if let Err(err) = client.ws.send(tungstenite::Message::binary(buf)).await {
                        println!("failed to send on websocket stream: {:#?}", err);
                        break;
                    }

                }

                ws_msg = client.ws.next() => {
                    
                    let ws_msg: tungstenite::Message = match ws_msg {
                        Some(Ok(ws_msg)) => ws_msg,
                        Some(Err(err)) => {
                            println!("failed to listen on websocket stream: {:#?}", err);
                            break;
                        }
                        None => {
                            println!("no tungstenite message found");
                            break;
                        }
                    };

                    let buf: Vec<u8> = match ws_msg {
                        tungstenite::Message::Binary(buf) => buf,
                        _ => {
                            println!("invalid tungstenite message format");
                            break;
                        }
                    };

                    let client_message: ClientMessage = match Self::parse_binary(&buf) {
                        Some(client_message) => client_message,
                        None => {
                            println!("invalid client message binary format");
                            break;
                        }
                    };

                    if let Err(err) = client.send_to_game.send(client_message).await { 
                        println!("error sending client message: {:#?}", err);
                        break;
                    }

                }

            }

        }

    }

    fn parse_binary(buf: &[u8]) -> Option<ClientMessage> {

        match buf[0] {
            0 => Some(ClientMessage::Jump),
            1 => Some(ClientMessage::WalkLeft),
            2 => Some(ClientMessage::WalkRight),
            _ => None,
        }

    }
    
}

static ADDR: &'static str = "127.0.0.1:9002";

#[tokio::main]
async fn main() {

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(&ADDR).await.expect("tcp listeniner bind");

    println!("Listening on:\n{}\n", ADDR);

    let (
        send_to_game, 
        receive_from_client
    ) = mpsc::channel(100);

    tokio::spawn(Game::init(receive_from_client));

    while let Ok((stream, _)) = listener.accept().await {
        // make copy sender
        tokio::spawn(Client::init(stream, send_to_game.clone()));
    }

}