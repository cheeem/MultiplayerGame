use tokio::sync::mpsc;
use crate::{ entity, room, };

// #[derive(Debug)]
// enum UserState {
//     Idle, 
//     RunningLeft,
//     RunningRight,
//     Jumping, 
// }

#[derive(Debug)]
pub struct User {
    // index 
    pub idx: u8,
    pub room_idx: usize,
    // channels
    pub send_to_client: mpsc::Sender<Vec<u8>>,
    // dynamic entity
    pub dynamic_entity: entity::DynamicEntity,
    // controls & state
    pub jump_buffer_ticks: u8,
    coyote_ticks: u8,
    pub holding_left: bool, 
    pub holding_right: bool, 
    pub holding_down: bool,
}

impl User {

    pub const JUMP_BUFFER_TICKS: u8 = 5;
    const COYOTE_TICKS: u8 = 3;

    const JUMP_FORCE: f32 = -40.0;
    const JUMP_CUTOFF: f32 = 0.5;

    const RUN_START_FORCE: f32 = 2.0;
    const RUN_END_FORCE: f32 = 4.0;
    const RUN_MAX_SPEED: f32 = 5.0;

    pub fn new(idx: u8, room_idx: usize, send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        let entity: entity::Entity = entity::Entity {
            x: 0.0,
            y: 0.0,
            width: 10.0, 
            height: 10.0, 
        };

        let dynamic_entity: entity::DynamicEntity = entity::DynamicEntity {
            entity,
            dx: 0.0,
            dy: 0.0,
            weight: 3.0,
        };

        Self {
            // index 
            idx,
            room_idx,
            // channels
            send_to_client,
            // entity
            dynamic_entity,
            // control & state
            jump_buffer_ticks: 0,
            coyote_ticks: 0,          
            holding_left: false,
            holding_right: false,
            holding_down: false,
        }

    }

    // pub fn respawn(&mut self, room_idx: usize) {
    //     self.room_idx = room_idx;
    //     self.dynamic_entity.dx = 0.0;
    //     self.dynamic_entity.dy = 0.0;
    //     self.dynamic_entity.entity.x = 0.0;
    //     self.dynamic_entity.entity.y = 0.0;
    //     self.jump_buffer_ticks = 0;
    //     self.coyote_ticks = 0;
    //     self.holding_left = false;
    //     self.holding_right = false;
    //     self.holding_down = false;
    // }

    pub fn tick<'a, 'b>(&'a mut self, users: impl Iterator<Item = &'b User>) {

        let room: &room::Room = &room::ROOMS[self.room_idx];

        let mut horizontal_collision: Option<entity::HorizontalCollision> = None;
        let mut vertical_collision: Option<entity::VerticalCollision> = None;

        let mut horizontal_time: f32 = f32::INFINITY; // could rely on 
        let mut vertical_time: f32 = f32::INFINITY;

        for user in users {

            if user.room_idx != self.room_idx {
                continue;
            }

            let entity: &entity::Entity = &user.dynamic_entity.entity;
            
            let (time, horizontal, vertical) = self.dynamic_entity.swept_collision(entity);
            
            if let Some(direction) = horizontal {
                if time < horizontal_time {
                    horizontal_time = time;
                    horizontal_collision = Some(entity::HorizontalCollision {
                        variant: entity::CollisionVariant::User(entity),
                        direction,
                        time,
                    });
                }
            } else if let Some(direction) = vertical {
                if time < vertical_time {
                    vertical_time = time;
                    vertical_collision = Some(entity::VerticalCollision {
                        variant: entity::CollisionVariant::User(entity),
                        direction,
                        time,
                    });
                }                
            }

        }

        for entity in room.platforms {
            
            let (time, _, vertical) = self.dynamic_entity.swept_collision(entity);
            
            if self.dynamic_entity.dy > 0.0 {
                if let Some(direction) = vertical {
                    if self.holding_down {
                        self.holding_down = false;
                    } else if time < vertical_time {
                        vertical_time = time;
                        vertical_collision = Some(entity::VerticalCollision {
                            variant: entity::CollisionVariant::Platform(entity),
                            direction,
                            time,
                        });
                    }
                }
            }

        }

        for door in room.doors {
            
            let (time, horizontal, vertical) = self.dynamic_entity.swept_collision(&door.entity);
            
            if let Some(direction) = horizontal {
                if time < horizontal_time {
                    horizontal_time = time;
                    horizontal_collision = Some(entity::HorizontalCollision {
                        variant: entity::CollisionVariant::Door(door),
                        direction,
                        time,
                    });
                }
            } else if let Some(direction) = vertical {
                if time < vertical_time {
                    vertical_time = time;
                    vertical_collision = Some(entity::VerticalCollision {
                        variant: entity::CollisionVariant::Door(door),
                        direction,
                        time,
                    });
                }                
            }

        }

        if horizontal_collision.is_none() {
            horizontal_collision = self.dynamic_entity.horizontal_bounds_collision(&room.bounds);
        }

        if vertical_collision.is_none() {
            vertical_collision = self.dynamic_entity.vertical_bounds_collision(&room.bounds);
        }

        match horizontal_collision {
            None => {
                
                self.dynamic_entity.entity.x += self.dynamic_entity.dx;
                
                match self.dynamic_entity.dx.partial_cmp(&0.0) {
                    Some(std::cmp::Ordering::Greater) => {

                        if self.holding_right == false {
                            self.end_run_right();
                        }
    
                    }
                    Some(std::cmp::Ordering::Less) => {

                        if self.holding_left == false {
                            self.end_run_left();
                        }
    
                    }
                    _ => (),
                }

            }
            Some(entity::HorizontalCollision { variant, direction, time }) => {
                match direction {
                    entity::HorizontalCollisionDirection::Left => {

                        match variant {
                            entity::CollisionVariant::Bounds => {
                                self.dynamic_entity.entity.x = 0.0;
                            }
                            entity::CollisionVariant::User(entity) => {
                                self.dynamic_entity.entity.x = entity.x + entity.width;
                            }
                            entity::CollisionVariant::Platform(_) => {
                                unreachable!();
                            }
                            entity::CollisionVariant::Door(door) => {
                                let room_idx: usize = door.room_idx;
                                let entity: &entity::Entity = &room::ROOMS[room_idx].doors[door.door_idx].entity;

                                self.room_idx = room_idx;
                                self.dynamic_entity.entity.x = entity.x - self.dynamic_entity.entity.width;
                            }
                        }

                        self.dynamic_entity.dx = 0.0;

                    }
                    entity::HorizontalCollisionDirection::Right => {

                        match variant {
                            entity::CollisionVariant::Bounds => {
                                self.dynamic_entity.entity.x = room.bounds.x_max - self.dynamic_entity.entity.width;
                            }
                            entity::CollisionVariant::User(entity) => {
                                self.dynamic_entity.entity.x = entity.x - self.dynamic_entity.entity.width;
                            }
                            entity::CollisionVariant::Platform(_) => {
                                unreachable!();
                            }
                            entity::CollisionVariant::Door(door) => {
                                let room_idx: usize = door.room_idx;
                                let entity: &entity::Entity = &room::ROOMS[room_idx].doors[door.door_idx].entity;

                                self.room_idx = room_idx;
                                self.dynamic_entity.entity.x = entity.x + entity.width;
                            }
                        }

                        self.dynamic_entity.dx = 0.0;

                    }
                }
            }
        }

        match vertical_collision {
            None => {

                self.dynamic_entity.entity.y += self.dynamic_entity.dy;

                if self.coyote_ticks > 0 {
                    self.coyote_ticks -= 1;
                }

            }
            Some(entity::VerticalCollision { variant, direction, time, }) => {
                match direction {
                    entity::VerticalCollisionDirection::Down => {

                        match variant {
                            entity::CollisionVariant::Bounds => {
                                self.dynamic_entity.entity.y = room.bounds.y_max - self.dynamic_entity.entity.height;
                            }
                            entity::CollisionVariant::User(entity) => {
                                self.dynamic_entity.entity.y = entity.y - self.dynamic_entity.entity.height;
                            }
                            entity::CollisionVariant::Platform(entity) => {
                                self.dynamic_entity.entity.y = entity.y - self.dynamic_entity.entity.height;
                            }
                            entity::CollisionVariant::Door(door) => {
                                self.dynamic_entity.entity.y = door.entity.y - self.dynamic_entity.entity.height;
                            }
                        }
        
                        self.dynamic_entity.dy = 0.0;
        
                        if self.jump_buffer_ticks > 0 {
                            self.jump();
                        } else {
                            self.coyote_ticks = Self::COYOTE_TICKS;
                        }

                    }
                    entity::VerticalCollisionDirection::Up => {

                        match variant {
                            entity::CollisionVariant::Bounds => {
                                self.dynamic_entity.entity.y = 0.0;
                            }
                            entity::CollisionVariant::User(entity) => {
                                self.dynamic_entity.entity.y = entity.y + entity.height;
                            }
                            entity::CollisionVariant::Platform(_) => {
                                unreachable!();
                            }
                            entity::CollisionVariant::Door(door) => {
                                self.dynamic_entity.entity.y = door.entity.y + door.entity.height;
                            }
                        }

                        self.dynamic_entity.dy = 0.0;

                    }
                }

            }
        }

        self.fall(room.gravity);

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

    }

    fn fall(&mut self, gravity: f32) {
        self.dynamic_entity.dy += gravity;
    }

    fn jump(&mut self) {
        self.dynamic_entity.dy = Self::JUMP_FORCE / self.dynamic_entity.weight;
        self.coyote_ticks = 0;
        self.jump_buffer_ticks = 0;
    }

    pub fn end_jump(&mut self) {
        // maybe check if actually jumping?
        self.dynamic_entity.dy *= Self::JUMP_CUTOFF;
    }

    fn run_left(&mut self) {

        let run_speed: f32 = self.dynamic_entity.dx - Self::RUN_START_FORCE / self.dynamic_entity.weight;
        
        self.dynamic_entity.dx = if run_speed > -Self::RUN_MAX_SPEED {
            run_speed
        } else {
            -Self::RUN_MAX_SPEED
        };

    }

    fn end_run_left(&mut self) {

        let run_speed: f32 = self.dynamic_entity.dx + Self::RUN_END_FORCE / self.dynamic_entity.weight;

        self.dynamic_entity.dx = if run_speed < 0.0 {
            run_speed
        } else {
            0.0
        };

    }

    fn run_right(&mut self) {
        
        let run_speed: f32 = self.dynamic_entity.dx + Self::RUN_START_FORCE / self.dynamic_entity.weight;
        
        self.dynamic_entity.dx = if run_speed < Self::RUN_MAX_SPEED {
            run_speed
        } else {
            Self::RUN_MAX_SPEED
        };

    }

    fn end_run_right(&mut self) {

        let run_speed: f32 = self.dynamic_entity.dx - Self::RUN_END_FORCE / self.dynamic_entity.weight;

        self.dynamic_entity.dx = if run_speed > 0.0 {
            run_speed
        } else {
            0.0
        };
        
    }

}