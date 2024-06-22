use tokio;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use futures_util::{ SinkExt, StreamExt };
use tokio_tungstenite;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
enum ClientMessage {
    Connect { 
        send_idx_to_client: oneshot::Sender<usize>,
        send_to_client: mpsc::Sender<Vec<u8>>,
    },
    Jump(usize),
    WalkLeft(usize),
    WalkRight(usize),
}

struct Game {
    receive_from_client: mpsc::Receiver<ClientMessage>,
    // must use try_send to avoid deadlocks
    users: Vec<Option<User>>,
}

#[derive(Debug)]
struct User {
    send_to_client: mpsc::Sender<Vec<u8>>,
    x: u8,
    y: u8,
    width: u8,
    height: u8,
}

struct Client {
    idx: usize,
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

                    match client_msg {
                        ClientMessage::Connect { send_idx_to_client, send_to_client } => {

                            match game.users.iter().position(|user| user.is_none()) {
                                Some(idx) => {
                                    if send_idx_to_client.send(idx).is_ok() {
                                        game.users[idx] = Some(User::new(send_to_client));
                                    }
                                }
                                None => {
                                    if send_idx_to_client.send(game.users.len()).is_ok() {
                                        game.users.push(Some(User::new(send_to_client)));
                                    }
                                }
                            }

                        },
                        ClientMessage::Jump(idx) => (),
                        ClientMessage::WalkLeft(idx) => { game.users[idx].as_mut().map(|user| user.walk_left()); },
                        ClientMessage::WalkRight(idx) => { game.users[idx].as_mut().map(|user| user.walk_right()); },
                    }

                },

                _ = timer.tick() => {

                    let len: usize = game.users.len();

                    if len == 0 { 
                        continue;
                    }

                    let mut last_idx: Option<usize> = None;

                    for idx in (0..len).rev() {
                        if game.users[idx].is_some() {
                            last_idx = Some(idx);
                            break;
                        }
                    }

                    let last_idx: usize = match last_idx {
                        Some(idx) => idx,
                        None => continue,
                    };

                    let buf: Vec<u8> = game.create_render_buffer();

                    for idx in 0..last_idx {

                        let user: &User = match &game.users[idx] {
                            Some(user) => user,
                            None => continue,
                        };

                        match user.send_to_client.try_send(buf.clone()) {
                            Ok(_) => (),
                            Err(mpsc::error::TrySendError::Closed(_)) => {
                                // drop user + sender
                                let _ = game.users[idx] = None;
                            },
                            Err(err) => {
                                println!("failed to send render buffer: {:#?}", err);
                                return; 
                            }
                        }

                    }

                    let last_user: &User = match &game.users[last_idx] {
                        Some(user) => user,
                        None => continue,
                    };

                    match last_user.send_to_client.try_send(buf) {
                        Ok(_) => (),
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            // drop user + sender
                            let _ = game.users[last_idx] = None;
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

        for user in self.users.iter().filter_map(|user| user.as_ref()) {
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

        let (
            send_idx_to_client,
            receive_idx_from_game 
        ) = oneshot::channel();

        if let Err(err) = send_to_game.send(ClientMessage::Connect { 
            send_idx_to_client, 
            send_to_client, 
        }).await {
            println!("failed to connect to game: {:#?}", err);
            return;
        }

        let idx: usize = match receive_idx_from_game.await {
            Ok(idx) => idx,
            Err(err) => {
                println!("error receiving index: {:#?}", err);
                return;
            }
        };

        let mut client: Self = Self {
            idx,
            ws,
            receive_from_game,
            send_to_game,
        };

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

                    let client_message: ClientMessage = match Self::parse_binary(&buf, client.idx) {
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

    fn parse_binary(buf: &[u8], idx: usize) -> Option<ClientMessage> {

        match buf[0] {
            0 => Some(ClientMessage::Jump(idx)),
            1 => Some(ClientMessage::WalkLeft(idx)),
            2 => Some(ClientMessage::WalkRight(idx)),
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