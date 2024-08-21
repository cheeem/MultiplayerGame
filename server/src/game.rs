use tokio::sync::mpsc;
use crate::{ bullet, client, entity, ray, room, slice, user };
use slice::IterPlucked;

pub struct Game {
    receive_from_client: mpsc::Receiver<client::Message>,
    users: Vec<Option<user::User>>,
    rooms_mut: room::RoomsMut,
    rooms_to_render: Vec<usize>,
}

pub const TICK_DT: u64 = 16;
pub const MAX_PLAYERS: usize = u8::MAX as usize;

impl Game {

    pub async fn init(receive_from_client: mpsc::Receiver<client::Message>) {

        let mut game: Self = Self {
            receive_from_client,
            users: Vec::with_capacity(MAX_PLAYERS),
            rooms_mut: room::rooms_mut(),
            rooms_to_render: Vec::with_capacity(room::ROOM_COUNT), 
        };

        let mut timer: tokio::time::Interval = tokio::time::interval(tokio::time::Duration::from_millis(TICK_DT));

        loop {
            tokio::select! {
                
                client_msg = game.receive_from_client.recv() => {

                    let client_msg: client::Message = match client_msg {
                        Some(msg) => msg,
                        None => return println!("no client message found"),
                    };

                    game.handle_client_msg(client_msg);

                },

                _ = timer.tick() => game.tick(),

            }
        }

    }

    fn handle_client_msg(&mut self, client_msg: client::Message) {

        match client_msg {
            client::Message::Connect { send_idx_to_client, send_to_client } => {

                match self.users.iter().position(|user| user.is_none()) {
                    Some(idx) => {
                        
                        let target_user_idx: usize = user::User::get_target_idx(&mut self.users, idx);
                        
                        if send_idx_to_client.send(idx).is_ok() {
                            self.users[idx] = Some(user::User::new(idx as u8, 0, target_user_idx,  send_to_client));
                        }

                    }
                    None => {
                        
                        let idx: usize = self.users.len();
                        
                        let target_user_idx: usize = user::User::get_target_idx(&mut self.users, idx);
                        
                        if send_idx_to_client.send(idx).is_ok() {
                            self.users.push(Some(user::User::new(idx as u8, 0, target_user_idx, send_to_client)));
                        }

                    }
                }

            },
            client::Message::UpStart(idx) => { self.users[idx].as_mut().map(|user| user.jump_buffer_ticks = user::User::JUMP_BUFFER_TICKS); },
            client::Message::UpEnd(idx) => { self.users[idx].as_mut().map(|user| user.end_jump()); },
            client::Message::DownStart(idx) => { self.users[idx].as_mut().map(|user| user.holding_down = true); }
            client::Message::DownEnd(idx) => { self.users[idx].as_mut().map(|user| user.holding_down = false); }
            client::Message::LeftStart(idx) => { self.users[idx].as_mut().map(|user| user.holding_left = true); },
            client::Message::LeftEnd(idx) => { self.users[idx].as_mut().map(|user| user.holding_left = false); },
            client::Message::RightStart(idx) => { self.users[idx].as_mut().map(|user| user.holding_right = true); },
            client::Message::RightEnd(idx) => { self.users[idx].as_mut().map(|user| user.holding_right = false); },
            client::Message::Click(idx, x, y) => { 
                
                let user: &user::User = match self.users[idx].as_ref() {
                    Some(user) => user,
                    None => return,
                };

                self.rooms_mut[user.room_idx].bullets.push(bullet::Bullet {
                    user_idx: idx,
                    target_user_idx: user.target_user_idx,
                    room_idx: user.room_idx,
                    ray: ray::Ray::from_entity_and_position(&user.dynamic_entity.entity, x, y),
                }); 

            }
        };

    }

    fn tick(&mut self) {

        let len: usize = self.users.len();

        if len == 0 { 
            return;
        }

        for room_mut in &mut self.rooms_mut {
                                    
            for bullet in &room_mut.bullets {
                room_mut.bullet_paths.push(bullet.tick(&mut self.users));
            }
            
            room_mut.bullets.clear();
            
        }

        for idx in 0..self.users.len() {

            if self.users[idx].is_none() {
                continue;
            }

            let (plucked, iter) = self.users.iter_plucked(idx).unwrap(); // none len = 0 (can't happen if in a loop)
            let user: &mut user::User = plucked.as_mut().unwrap();
            let users_iter = iter.filter_map(|u| u.as_ref());

            self.rooms_to_render.push(user.room_idx);

            user.tick(users_iter);

        }

        for idx in 0..self.rooms_to_render.len() {
            let room_idx: usize = self.rooms_to_render[idx]; // indexing to avoid dealing with additional pointer indirection 
            let buf: Vec<u8> = self.render_room(room_idx);
            self.send_render_buffer(room_idx, buf);
        }

        self.rooms_to_render.clear();

        for room_mut in &mut self.rooms_mut {
            room_mut.bullet_paths.clear();
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

    fn send_render_buffer(&mut self, room_idx: usize, mut buf: Vec<u8>) {

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

            let mut buf: Vec<u8> = buf.clone();

            // footer
            buf.push(user.idx);
            buf.push(user.target_user_idx as u8);

            match user.send_to_client.try_send(buf) {
                Ok(_) => (),
                Err(mpsc::error::TrySendError::Closed(_)) => self.users[idx] = None,
                Err(err) => return println!("failed to send render buffer: {:#?}", err),
            }

        }

        let last_user: &user::User = match &self.users[last_idx] {
            Some(user) => user,
            None => return,
        };

        // footer
        buf.push(last_user.idx);
        buf.push(last_user.target_user_idx as u8);

        match last_user.send_to_client.try_send(buf) {
            Ok(_) => (),
            Err(mpsc::error::TrySendError::Closed(_)) => self.users[last_idx] = None,
            Err(err) => return println!("failed to send render buffer: {:#?}", err),
        }

    }

}