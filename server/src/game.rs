use tokio::sync::mpsc;
use crate::{ client, entity::StaticEntity, platform, user };

pub struct Game {
    receive_from_client: mpsc::Receiver<client::Message>,
    users: Vec<Option<user::User>>,
    platforms: Vec<platform::Platform>, 
}

pub const GRAVITY: i8 = 2;

impl Game {

    pub async fn init(receive_from_client: mpsc::Receiver<client::Message>) {

        let mut game: Self = Self {
            receive_from_client,
            users: Vec::new(),
            platforms: Vec::new(),
        };

        game.platforms.push(platform::Platform { 
            entity: StaticEntity { x: 50, y: 20, width: 50, height: 5 }, 
        });

        let mut timer: tokio::time::Interval = tokio::time::interval(tokio::time::Duration::from_millis(25));

        loop {
            tokio::select! {
                
                client_msg = game.receive_from_client.recv() => {

                    let client_msg: client::Message = match client_msg {
                        Some(client_msg) => client_msg,
                        None => return println!("no client message found"),
                    };

                    match client_msg {
                        client::Message::Connect { send_idx_to_client, send_to_client } => {

                            match game.users.iter().position(|user| user.is_none()) {
                                Some(idx) => {
                                    if send_idx_to_client.send(idx).is_ok() {
                                        game.users[idx] = Some(user::User::new(send_to_client));
                                    }
                                }
                                None => {
                                    if send_idx_to_client.send(game.users.len()).is_ok() {
                                        game.users.push(Some(user::User::new(send_to_client)));
                                    }
                                }
                            }

                        },
                        client::Message::UpStart(idx) => { game.users[idx].as_mut().map(|user| user.jump_buffer_ticks = user::User::JUMP_BUFFER_TICKS); },
                        client::Message::UpEnd(idx) => { game.users[idx].as_mut().map(|user| user.end_jump()); },
                        client::Message::LeftStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = true ); },
                        client::Message::LeftEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = false); },
                        client::Message::RightStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = true); },
                        client::Message::RightEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = false); },
                    }

                },

                _ = timer.tick() => {

                    let len: usize = game.users.len();

                    if len == 0 { 
                        continue;
                    }

                    for user in game.users.iter_mut().filter_map(|user| user.as_mut()) {
                        user.tick(&game.platforms);
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

                        let user: &user::User = match &game.users[idx] {
                            Some(user) => user,
                            None => continue,
                        };

                        match user.send_to_client.try_send(buf.clone()) {
                            Ok(_) => (),
                            Err(mpsc::error::TrySendError::Closed(_)) => game.users[idx] = None,
                            Err(err) => return println!("failed to send render buffer: {:#?}", err),
                        }

                    }

                    let last_user: &user::User = match &game.users[last_idx] {
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
            buf.push(user.entity.x);
            buf.push(user.entity.y);
            buf.push(user.entity.width);
            buf.push(user.entity.height);
        }

        return buf;

    }

}