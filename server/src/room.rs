use crate::{ bullet, entity, };

pub struct Room {
    pub gravity: f32,
    pub bounds: Bounds,
    pub platforms: &'static [entity::Entity],
    pub doors: &'static [Door],
}

pub struct RoomMut {
    pub bullets: Vec<bullet::Bullet>, 
    pub bullet_paths: Vec<bullet::BulletPath>,
}

pub struct Door {
    pub entity: entity::Entity, 
    pub room_idx: usize,
    pub door_idx: usize,
}

pub struct Bounds {
    pub x_max: f32,
    pub y_max: f32,
}

pub type RoomsMut = [RoomMut; ROOM_COUNT];

pub const ROOM_COUNT: usize = 2;

pub const ROOMS: [Room; ROOM_COUNT] = [
    Room {
        gravity: 1.5,
        bounds: Bounds { x_max: u8::MAX as f32, y_max: u8::MAX as f32 },
        platforms: &[
            entity::Entity { x: 70.0, y: 245.0, width: 50.0, height: 3.0 },
            entity::Entity { x: 100.0, y: 220.0, width: 50.0, height: 3.0 },
            entity::Entity { x: 170.0, y: 230.0, width: 50.0, height: 3.0 },
        ],
        doors: &[
            Door {
                entity: entity::Entity { x: 250.0, y: 225.0, width: 5.0, height: 30.0 },
                room_idx: 1,
                door_idx: 0,
            }
        ],
    },
    Room {
        gravity: 1.5,
        bounds: Bounds { x_max: 255.0, y_max: 255.0 },
        platforms: &[
            entity::Entity { x: 70.0, y: 245.0, width: 50.0, height: 3.0 },
            //entity::Entity { x: 100.0, y: 220.0, width: 50.0, height: 3.0 },
            //entity::Entity { x: 170.0, y: 230.0, width: 50.0, height: 3.0 },
        ],
        doors: &[
            Door {
                entity: entity::Entity { x: 0.0, y: 225.0, width: 5.0, height: 30.0 },
                room_idx: 0,
                door_idx: 0,
            }
        ],
    }
];

impl RoomMut {
    const fn new() -> Self {
        RoomMut { 
            bullets: Vec::new(), 
            bullet_paths: Vec::new(), 
        }
    }
}

pub const fn rooms_mut() -> RoomsMut {
    [
        RoomMut::new(),
        RoomMut::new(),
    ]
}
