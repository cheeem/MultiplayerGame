use tokio::sync::mpsc;
use crate::{ bullet, client, entity, room, slice, user };
use slice::IterPlucked;

pub struct Game {
    receive_from_client: mpsc::Receiver<client::Message>,
    users: Vec<Option<user::User>>,
    rooms_mut: room::RoomsMut,
    rooms_to_render: Vec<usize>, 
}

pub const TICK_DT: u64 = 20;

impl Game {

    pub async fn init(receive_from_client: mpsc::Receiver<client::Message>) {

        let mut game: Self = Self {
            receive_from_client,
            users: Vec::with_capacity(u8::MAX as usize), // could change to smaller #
            rooms_mut: room::rooms_mut(),
            rooms_to_render: Vec::with_capacity(u8::MAX as usize), // could change to smaller #
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
                                        game.users[idx] = Some(user::User::new(idx as u8, 0, send_to_client));
                                    }
                                }
                                None => {
                                    let idx: usize = game.users.len();
                                    if send_idx_to_client.send(idx).is_ok() {
                                        game.users.push(Some(user::User::new(idx as u8, 0, send_to_client)));
                                    }
                                }
                            }

                        },
                        client::Message::UpStart(idx) => { game.users[idx].as_mut().map(|user| user.jump_buffer_ticks = user::User::JUMP_BUFFER_TICKS); },
                        client::Message::UpEnd(idx) => { game.users[idx].as_mut().map(|user| user.end_jump()); },
                        client::Message::DownStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_down = true); }
                        client::Message::DownEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_down = false); }
                        client::Message::LeftStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = true); },
                        client::Message::LeftEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_left = false); },
                        client::Message::RightStart(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = true); },
                        client::Message::RightEnd(idx) => { game.users[idx].as_mut().map(|user| user.holding_right = false); },
                        client::Message::Click(idx, x, y) => { 
                            
                            let user: &user::User = match game.users[idx].as_ref() {
                                Some(user) => user,
                                None => continue,
                            };

                            game.rooms_mut[user.room_idx].bullets.push(bullet::Bullet::from_click_position(user, idx, x, y)); 

                        }
                    };

                },

                _ = timer.tick() => {

                    let len: usize = game.users.len();

                    if len == 0 { 
                        continue;
                    }

                    for room_mut in &mut game.rooms_mut {
                                                
                        for bullet in &room_mut.bullets {
                            room_mut.bullet_paths.push(bullet.tick(&mut game.users));
                        }
                        
                        room_mut.bullets.clear();
                        
                    }

                    for idx in 0..game.users.len() {

                        if game.users[idx].is_none() {
                            continue;
                        }

                        let (plucked, iter) = game.users.iter_plucked(idx).unwrap(); // none len = 0 (can't happen if in a loop)
                        let user = plucked.as_mut().unwrap();
                        let users_iter = iter.filter_map(|u| u.as_ref());

                        game.rooms_to_render.push(user.room_idx);

                        user.tick(users_iter);

                    }

                    for idx in 0..game.rooms_to_render.len() {
                        let room_idx = game.rooms_to_render[idx];
                        let buf: Vec<u8> = game.render_room(room_idx);
                        game.send_render_buffer(room_idx, buf);
                    }

                    game.rooms_to_render.clear();

                    for room_mut in &mut game.rooms_mut {
                        room_mut.bullet_paths.clear();
                    }

                },

            }
        }

    }

    fn render_room(&self, room_idx: usize) -> Vec<u8> {

        let mut buf: Vec<u8> = Vec::new();

        let room: &room::Room = &room::ROOMS[room_idx];
        let room_mut: &room::RoomMut = &self.rooms_mut[room_idx];

        for entity in room.platforms
        {
            
            buf.push(1);
            buf.push(2);

            buf.push(entity.width as u8);
            buf.push(entity.height as u8);

            buf.extend_from_slice(&(entity.x as u16).to_be_bytes());
            buf.extend_from_slice(&(entity.y as u16).to_be_bytes());

        }

        for entity in room.doors.iter().map(|door| &door.entity) {

            buf.push(3);
            buf.push(4);

            buf.push(entity.width as u8);
            buf.push(entity.height as u8);

            buf.extend_from_slice(&(entity.x as u16).to_be_bytes());
            buf.extend_from_slice(&(entity.y as u16).to_be_bytes());

        }

        for path in &room_mut.bullet_paths {

            buf.push(4);
            buf.push(5);

            buf.extend_from_slice(&(path.origin_x as u16).to_be_bytes());
            buf.extend_from_slice(&(path.origin_y as u16).to_be_bytes());

            buf.extend_from_slice(&(path.end_x as u16).to_be_bytes());
            buf.extend_from_slice(&(path.end_y as u16).to_be_bytes());

        }

        for user in self.users
            .iter()
            .filter_map(|user| user.as_ref())
            .filter(|user| user.room_idx == room_idx)
        {
                        
            buf.push(0);
            buf.push(0);
            buf.push(0);
            buf.push(user.idx);

            let entity: &entity::Entity = &user.dynamic_entity.entity;

            buf.push(entity.width as u8);
            buf.push(entity.height as u8);

            buf.extend_from_slice(&(entity.x as u16).to_be_bytes());
            buf.extend_from_slice(&(entity.y as u16).to_be_bytes());

        }

        return buf;

    }

    fn send_render_buffer(&mut self, room_idx: usize, buf: Vec<u8>) {

        let mut last_idx: Option<usize> = None;

        for idx in (0..self.users.len()).rev() {

            let user_room_idx: usize = match &self.users[idx] {
                Some(user) => user.room_idx,
                None => continue,
            };

            if user_room_idx != room_idx {
                continue;
            }

            last_idx = Some(idx);
            break;

        }

        let last_idx: usize = match last_idx {
            Some(idx) => idx,
            None => return,
        };

        for idx in 0..last_idx {

            let user: &user::User = match &self.users[idx] {
                Some(user) => user,
                None => continue,
            };

            if user.room_idx != room_idx {
                continue;
            }

            match user.send_to_client.try_send(buf.clone()) {
                Ok(_) => (),
                Err(mpsc::error::TrySendError::Closed(_)) => self.users[idx] = None,
                Err(err) => return println!("failed to send render buffer: {:#?}", err),
            }

        }

        let last_user: &user::User = match &self.users[last_idx] {
            Some(user) => user,
            None => return,
        };

        match last_user.send_to_client.try_send(buf) {
            Ok(_) => (),
            Err(mpsc::error::TrySendError::Closed(_)) => self.users[last_idx] = None,
            Err(err) => return println!("failed to send render buffer: {:#?}", err),
        }

    }

}