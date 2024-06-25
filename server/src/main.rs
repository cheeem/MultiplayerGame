use tokio;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use futures_util::{ SinkExt, StreamExt };
use tokio_tungstenite;
use tokio_tungstenite::tungstenite;

// // probably slow due to indirection associated with trait objects and the sheer # of function calls required 
// trait Entity {

//     fn x(&self) -> u8;
//     fn y(&self) -> u8;
//     fn width(&self) -> u8;
//     fn height(&self) -> u8;

//     fn aabb_collision(&self, other: &impl Entity) -> bool {
//         self.x() < other.x() + other.width() &&
//         self.x() + self.width() > other.x() &&
//         self.y() < other.y() + other.height() &&
//         self.y() + self.height() > other.y()
//     }

// }

#[derive(Debug)]
enum ClientMessage {
    Connect { 
        send_idx_to_client: oneshot::Sender<usize>,
        send_to_client: mpsc::Sender<Vec<u8>>,
    },
    UpStart(usize),
    UpEnd(usize),
    LeftStart(usize),
    LeftEnd(usize),
    RightStart(usize),
    RightEnd(usize),
}

// #[derive(Debug)]
// enum UserState {
//     Idle, 
//     RunningLeft,
//     RunningRight,
//     Jumping, 
// }

#[derive(Debug)]
struct User {
    send_to_client: mpsc::Sender<Vec<u8>>,
    //
    x: u8,
    y: u8,
    dx: i8,
    dy: i8,
    width: u8,
    height: u8,
    weight: i8,
    grounded: bool,
    holding_left: bool, 
    holding_right: bool, 
}

struct Game {
    receive_from_client: mpsc::Receiver<ClientMessage>,
    users: Vec<Option<User>>,
}

struct Client {
    idx: usize,
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    receive_from_game: mpsc::Receiver<Vec<u8>>,
    send_to_game: mpsc::Sender<ClientMessage>,
}

impl User {

    const GRAVITY: i8 = 2;
    const JUMP_FORCE: i8 = -40;
    const JUMP_CUTOFF: i8 = 2;

    const RUN_START_FORCE: i8 = 2;
    const RUN_END_FORCE: i8 = 4;
    const RUN_MAX_SPEED: i8 = 8;

    fn new(send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        Self {
            send_to_client,
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            width: 10, 
            height: 10, 
            weight: 2,
            grounded: false,            
            holding_left: false,
            holding_right: false,
        }

    }

    fn tick(&mut self) { 

        self.fall(); 

        if self.holding_left {
            self.run_left();
        }

        if self.holding_right {
            self.run_right();
        }

        match self.dx.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let dx_magnitude: u8 = self.dx as u8;
                let no_bounds_collision: bool = u8::MAX - self.width + 1 - dx_magnitude > self.x;

                if no_bounds_collision {

                    if self.holding_right == false {
                        self.end_run_right();
                    }

                    self.x += dx_magnitude;

                } else {
                    self.x = u8::MAX - self.width + 1;
                    self.dx = 0;
                }

            }
            std::cmp::Ordering::Less => {

                let dx_magnitude: u8 = (self.dx * -1) as u8;
                let no_bounds_collision: bool = self.x > dx_magnitude;

                if no_bounds_collision {

                    if self.holding_left == false {
                        self.end_run_left();
                    }

                    self.x -= dx_magnitude;

                } else {
                    self.x = 0;
                    self.dx = 0;
                }

            }
        }

        match self.dy.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let dy_magnitude: u8 = self.dy as u8;
                let no_bounds_collision: bool = u8::MAX - self.height + 1 - dy_magnitude > self.y;

                if no_bounds_collision {

                    self.y += dy_magnitude;

                    self.grounded = false;

                } else {

                    self.y = u8::MAX - self.height + 1;
                    self.dy = 0;
                    
                    self.grounded = true;

                }

            }
            std::cmp::Ordering::Less => {

                let dy_magnitude: u8 = (self.dy * -1) as u8;
                let no_bounds_collision: bool = self.y > dy_magnitude;

                if no_bounds_collision {
                    self.y -= dy_magnitude;
                } else {
                    self.y = 0;
                    self.dy = 0;
                }

            }
        }

    }

    fn fall(&mut self) {
        self.dy += Self::GRAVITY;
    }

    fn jump(&mut self) {
        if self.grounded {
            self.dy = Self::JUMP_FORCE / self.weight;
        }
    }

    fn end_jump(&mut self) {
        self.dy /= Self::JUMP_CUTOFF;
    }

    fn run_left(&mut self) {
        self.dx = std::cmp::max(
            -Self::RUN_MAX_SPEED, 
            self.dx - Self::RUN_START_FORCE / self.weight,
        );
    }

    fn end_run_left(&mut self) {
        self.dx = std::cmp::min(
            0, 
            self.dx + Self::RUN_END_FORCE / self.weight,
        ); 
    }

    fn run_right(&mut self) {
        self.dx = std::cmp::min(
            Self::RUN_MAX_SPEED, 
            self.dx + Self::RUN_START_FORCE / self.weight,
        );
    }

    fn end_run_right(&mut self) {
        self.dx = std::cmp::max(
            0, 
            self.dx - Self::RUN_END_FORCE / self.weight,
        ); 
    }

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
                        None => return println!("no client message found"),
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
                        ClientMessage::UpStart(idx) => { game.users[idx].as_mut().map(|user| user.jump()); },
                        ClientMessage::UpEnd(idx) => { game.users[idx].as_mut().map(|user| user.end_jump()); },
                        ClientMessage::LeftStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = true ); },
                        ClientMessage::LeftEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = false); },
                        ClientMessage::RightStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = true); },
                        ClientMessage::RightEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = false); },
                    }

                },

                _ = timer.tick() => {

                    let len: usize = game.users.len();

                    if len == 0 { 
                        continue;
                    }

                    for user in game.users.iter_mut().filter_map(|user| user.as_mut()) {
                        user.tick();
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
                            Err(mpsc::error::TrySendError::Closed(_)) => game.users[idx] = None,
                            Err(err) => return println!("failed to send render buffer: {:#?}", err),
                        }

                    }

                    let last_user: &User = match &game.users[last_idx] {
                        Some(user) => user,
                        None => continue,
                    };

                    match last_user.send_to_client.try_send(buf) {
                        Ok(_) => (),
                        Err(mpsc::error::TrySendError::Closed(_)) => game.users[last_idx] = None,
                        Err(err) => return println!("failed to send render buffer: {:#?}", err),
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

impl Client {

    async fn init(stream: tokio::net::TcpStream, send_to_game: mpsc::Sender<ClientMessage>) {

        let ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = match tokio_tungstenite::accept_async(stream).await {
            Ok(ws) => ws,
            Err(_) => return println!("failed to connect to websocket"),
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
            return println!("failed to connect to game: {:#?}", err);
        }

        let idx: usize = match receive_idx_from_game.await {
            Ok(idx) => idx,
            Err(err) => return println!("error receiving index: {:#?}", err),
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
                        None => return println!("no render buffer found"),
                    };

                    if let Err(err) = client.ws.send(tungstenite::Message::binary(buf)).await {
                        return println!("failed to send on websocket stream: {:#?}", err);
                    }

                }

                ws_msg = client.ws.next() => {
                    
                    let ws_msg: tungstenite::Message = match ws_msg {
                        Some(Ok(ws_msg)) => ws_msg,
                        Some(Err(err)) => return println!("failed to listen on websocket stream: {:#?}", err),
                        None => return println!("no tungstenite message found"),
                    };

                    let buf: Vec<u8> = match ws_msg {
                        tungstenite::Message::Binary(buf) => buf,
                        _ => return println!("invalid tungstenite message format"),
                    };

                    let client_message: ClientMessage = match Self::parse_binary(&buf, client.idx) {
                        Some(client_message) => client_message,
                        None => return println!("invalid client message binary format"),
                    };

                    if let Err(err) = client.send_to_game.send(client_message).await { 
                        return println!("error sending client message: {:#?}", err);
                    }

                }

            }

        }

    }

    fn parse_binary(buf: &[u8], idx: usize) -> Option<ClientMessage> {

        match buf[0] {
            0 => Some(ClientMessage::UpStart(idx)),
            1 => Some(ClientMessage::UpEnd(idx)),
            2 => Some(ClientMessage::LeftStart(idx)),
            3 => Some(ClientMessage::LeftEnd(idx)),
            4 => Some(ClientMessage::RightStart(idx)),
            5 => Some(ClientMessage::RightEnd(idx)),
            _ => None,
        }

    }
    
}

static ADDR: &'static str = "127.0.0.1:3000";

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