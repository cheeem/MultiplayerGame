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

enum HorizontalCollision {
    None,
    Left,
    Right,
}

enum VerticalCollision {
    None,
    Up,
    Down,
}

// enum HortizontalMovementDirection {
//     None, 
//     Right,
//     Left, 
// }

// enum VerticalMovementDirection {
//     None, 
//     Up, 
//     Down,
// }

#[derive(Debug)]
struct User {
    // channels
    send_to_client: mpsc::Sender<Vec<u8>>,
    // entity
    x: u8,
    y: u8,
    dx: i8,
    dy: i8,
    width: u8,
    height: u8,
    weight: i8,
    // controls & state
    jump_buffer_ticks: u8,
    coyote_ticks: u8,
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

const GRAVITY: i8 = 2;

impl User {

    const JUMP_BUFFER_TICKS: u8 = 3;
    const COYOTE_TICKS: u8 = 3;

    const JUMP_FORCE: i8 = -40;
    const JUMP_CUTOFF: i8 = 2;

    const RUN_START_FORCE: i8 = 2;
    const RUN_END_FORCE: i8 = 4;
    const RUN_MAX_SPEED: i8 = 8;

    fn new(send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        Self {
            // channels
            send_to_client,
            // entity
            x: 0,
            y: 0,
            dx: 0,
            dy: GRAVITY,
            width: 10, 
            height: 10, 
            weight: 2,
            // control & state
            jump_buffer_ticks: 0,
            coyote_ticks: 0,          
            holding_left: false,
            holding_right: false,
        }

    }

    fn tick(&mut self) {

        let mut horizontal_collision: HorizontalCollision = HorizontalCollision::None;
        let mut vertical_collision: VerticalCollision = VerticalCollision::None;

        // let hortizontal_movement_direction: HortizontalMovementDirection = match self.dx.cmp(&0) {
        //     std::cmp::Ordering::Equal => HortizontalMovementDirection::None,
        //     std::cmp::Ordering::Greater => HortizontalMovementDirection::Right,
        //     std::cmp::Ordering::Less => HortizontalMovementDirection
        // }

        match self.dx.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let no_bounds_collision: bool = u8::MAX - self.width + 1 - self.dx as u8 > self.x;

                if !no_bounds_collision {
                    horizontal_collision = HorizontalCollision::Right;
                    self.x = u8::MAX - self.width + 1;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.x <= (self.dx * -1) as u8;

                if bounds_collision {
                    horizontal_collision = HorizontalCollision::Left;
                    self.x = 0;
                }

            }
        }

        match self.dy.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let no_bounds_collision: bool = u8::MAX - self.height + 1 - self.dy as u8 > self.y;

                if !no_bounds_collision {
                    vertical_collision = VerticalCollision::Down;
                    self.y = u8::MAX - self.height + 1;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.y <= (self.dy * -1) as u8;

                if bounds_collision {
                    vertical_collision = VerticalCollision::Up;
                    self.y = 0;
                }

            }
        }

        self.fall();

        // maybe check if grounded and do something different if in air
        // or move inside horizontal collision :: none
        if self.holding_left {
            self.run_left();
        }

        if self.holding_right {
            self.run_right();
        }

        if self.jump_buffer_ticks > 0 {

            if self.coyote_ticks > 0 { 
                self.jump();
            } else {
                self.jump_buffer_ticks -= 1;
            }

        }

        match horizontal_collision {
            HorizontalCollision::None => {
                
                match self.dx.cmp(&0) {
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => {

                        if self.holding_left == false {
                            self.end_run_right();
                        }

                        self.x += self.dx as u8;
    
                    }
                    std::cmp::Ordering::Less => {

                        if self.holding_left == false {
                            self.end_run_left();
                        }

                        self.x -= (self.dx * -1) as u8;
    
                    }
                }

            }
            HorizontalCollision::Left => {
                self.dx = 0;
            },
            HorizontalCollision::Right => {
                self.dx = 0;
            },
        }

        match vertical_collision {
            VerticalCollision::None => {

                if self.coyote_ticks > 0 {
                    self.coyote_ticks -= 1;
                }

                match self.dy.cmp(&0) {
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => {
                        self.y += self.dy as u8;
                    }
                    std::cmp::Ordering::Less => {
                        self.y -= (self.dy * -1) as u8;
                    }
                }

            }
            VerticalCollision::Down => {

                self.dy = 0;

                if self.jump_buffer_ticks > 0 {
                    self.jump();
                } else {
                    self.coyote_ticks = Self::COYOTE_TICKS;
                }

            },
            VerticalCollision::Up => {
                self.dy = 0;
            },
        }

    }

    fn fall(&mut self) {
        self.dy += GRAVITY;
    }

    fn jump(&mut self) {
        self.dy = Self::JUMP_FORCE / self.weight;
        self.coyote_ticks = 0;
        self.jump_buffer_ticks = 0;
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
                        ClientMessage::UpStart(idx) => { game.users[idx].as_mut().map(|user| user.jump_buffer_ticks = User::JUMP_BUFFER_TICKS); },
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