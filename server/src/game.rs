use tokio::sync::mpsc;
use crate::{ client, entity, platform, slice, user };
use slice::IterPlucked;

pub struct Game {
    receive_from_client: mpsc::Receiver<client::Message>,
    users: Vec<Option<user::User>>,
    platforms: &'static [platform::Platform], 
}

pub const BOUNDS_X_MIN: f32 = 0.0;
pub const BOUNDS_X_MAX: f32 = 255.0;
pub const BOUNDS_Y_MIN: f32 = 0.0;
pub const BOUNDS_Y_MAX: f32 = 255.0;

pub const GRAVITY: f32 = 2.0;
pub const TICK_DT: u64 = 20;

const PLATFORMS: &'static [platform::Platform] = &[
    platform::Platform {
        entity: entity::Entity { x: 70.0, y: 245.0, width: 50.0, height: 3.0 },
    },
    platform::Platform {
        entity: entity::Entity { x: 100.0, y: 220.0, width: 50.0, height: 3.0 },
    },
    platform::Platform {
        entity: entity::Entity { x: 170.0, y: 230.0, width: 50.0, height: 3.0 },
    }
];

impl Game {

    pub async fn init(receive_from_client: mpsc::Receiver<client::Message>) {

        let mut game: Self = Self {
            receive_from_client,
            users: Vec::with_capacity(u8::MAX as usize),
            platforms: PLATFORMS,
        };

        let mut timer: tokio::time::Interval = tokio::time::interval(tokio::time::Duration::from_millis(TICK_DT));

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
                                        game.users[idx] = Some(user::User::new(idx as u8, send_to_client));
                                    }
                                }
                                None => {
                                    let idx: usize = game.users.len();
                                    if send_idx_to_client.send(idx).is_ok() {
                                        game.users.push(Some(user::User::new(idx as u8, send_to_client)));
                                    }
                                }
                            }

                        },
                        client::Message::UpStart(idx) => { game.users[idx].as_mut().map(|user| user.jump_buffer_ticks = user::User::JUMP_BUFFER_TICKS); },
                        client::Message::UpEnd(idx) => { game.users[idx].as_mut().map(|user| user.end_jump()); },
                        client::Message::DownStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_down = true ); }
                        client::Message::DownEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_down = false ); }
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

                    // for idx in 0..game.users.len() {
                    //     game.users[idx].as_mut().unwrap().tick(&game.users.split_, &game.platforms);
                    // }

                    for idx in 0..game.users.len() {

                        if game.users[idx].is_none() {
                            continue;
                        }

                        let (plucked, iter) = game.users.iter_plucked(idx).unwrap();
                        let user = plucked.as_mut().unwrap();
                        let users_iter = iter.filter_map(|user| user.as_ref());

                        user.tick(users_iter, game.platforms);

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

        for entity in self.platforms
            .iter()
            .map(|platform| &platform.entity) 
        {
            
            buf.push(1);
            buf.push(1);

            buf.push(entity.width as u8);
            buf.push(entity.height as u8);

            let bytes_x: [u8; 2] = (entity.x as u16).to_be_bytes();
            let bytes_y: [u8; 2] = (entity.y as u16).to_be_bytes();

            buf.push(bytes_x[1]);
            buf.push(bytes_x[0]);
            buf.push(bytes_y[1]);
            buf.push(bytes_y[0]);


        }

        for user in self.users
            .iter()
            .filter_map(|user| user.as_ref())
        {
                        
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(user.idx);

            let entity: &entity::Entity = &user.dynamic_entity.entity;

            buf.push(entity.width as u8);
            buf.push(entity.height as u8);

            let bytes_x: [u8; 2] = (entity.x as u16).to_be_bytes();

            buf.push(bytes_x[1]);
            buf.push(bytes_x[0]);

            let bytes_y: [u8; 2] = (entity.y as u16).to_be_bytes();

            buf.push(bytes_y[1]);
            buf.push(bytes_y[0]);

        }

        return buf;

    }

}