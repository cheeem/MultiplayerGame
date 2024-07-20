
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use futures_util::{ SinkExt, StreamExt };
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum Message {
    Connect { 
        send_idx_to_client: oneshot::Sender<usize>,
        send_to_client: mpsc::Sender<Vec<u8>>,
    },
    UpStart(usize),
    UpEnd(usize),
    DownStart(usize),
    DownEnd(usize),
    LeftStart(usize),
    LeftEnd(usize),
    RightStart(usize),
    RightEnd(usize),
    Click(usize, f32, f32),
}

pub struct Client {
    idx: usize,
    idx_u8: u8,
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    receive_from_game: mpsc::Receiver<Vec<u8>>,
    send_to_game: mpsc::Sender<Message>,
}

impl Client {

    pub async fn init(stream: tokio::net::TcpStream, send_to_game: mpsc::Sender<Message>) {

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

        if let Err(err) = send_to_game.send(Message::Connect { 
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
            idx_u8: idx as u8,
            ws,
            receive_from_game,
            send_to_game,
        };

        // use tokio select to create 2 tasks, one for passing on client messages (below), and one for listening for render commands from game
        loop {
            tokio::select! {

                buf = client.receive_from_game.recv() => {

                    let mut buf: Vec<u8> = match buf {
                        Some(buf) => buf,
                        None => return println!("no render buffer found"),
                    };

                    // footer
                    buf.push(client.idx_u8);

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

                    let client_message: Message = match Self::parse_binary(&buf, client.idx) {
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

    fn parse_binary(buf: &[u8], idx: usize) -> Option<Message> {

        match buf[0] {
            0 => Some(Message::UpStart(idx)),
            1 => Some(Message::UpEnd(idx)),
            2 => Some(Message::DownStart(idx)),
            3 => Some(Message::DownEnd(idx)),
            4 => Some(Message::LeftStart(idx)),
            5 => Some(Message::LeftEnd(idx)),
            6 => Some(Message::RightStart(idx)),
            7 => Some(Message::RightEnd(idx)),
            8 => Some(Message::Click(
                idx, 
                u16::from_be_bytes(buf[1..3].try_into().ok()?) as f32, 
                u16::from_be_bytes(buf[3..5].try_into().ok()?) as f32,
            )),
            _ => None,
        }

    }
    
}