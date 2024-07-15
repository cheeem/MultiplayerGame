use tokio::sync::mpsc;
use crate::{ entity, game, platform };

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

    pub fn new(idx: u8, send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        let entity: entity::Entity = entity::Entity {
            x: 0.0,
            y: 0.0,
            width: 10.0, 
            height: 10.0, 
        };

        let dynamic_entity: entity::DynamicEntity = entity::DynamicEntity {
            entity,
            dx: 0.0,
            dy: game::GRAVITY,
            weight: 3.0,
        };

        Self {
            // index 
            idx,
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

    pub fn tick<'a, 'b>(&'a mut self, users: impl Iterator<Item = &'b User>, platforms: &'b [platform::Platform]) {

        let mut horizontal_collision: entity::HorizontalCollision = entity::HorizontalCollision::None;
        let mut vertical_collision: entity::VerticalCollision = entity::VerticalCollision::None;

        let mut horizontal_time: f32 = f32::INFINITY;
        let mut vertical_time: f32 = f32::INFINITY;
        
        // for platform in platforms {
        //     match (&horizontal_collision, &vertical_collision) {
        //         (entity::HorizontalCollision::None, entity::VerticalCollision::None) => {
        //             horizontal_collision = self.entity.horizonal_static_collision(&platform.entity);
        //             vertical_collision = self.entity.vertical_static_collision(&platform.entity);
        //         }
        //         (entity::HorizontalCollision::None, _) => {
        //             horizontal_collision = self.entity.horizonal_static_collision(&platform.entity);
        //         }
        //         (_, entity::VerticalCollision::None) => {
        //             vertical_collision = self.entity.vertical_static_collision(&platform.entity);
        //         }
        //         _ => break
        //     }
        // }

        for user in users {
            
            let (time, horizontal, vertical) = self.dynamic_entity.swept_collision(&user.dynamic_entity.entity, entity::CollisionType::User);
            
            if horizontal.is_some() {
                if time < horizontal_time {
                    horizontal_time = time;
                    horizontal_collision = horizontal;
                }
            } else if vertical.is_some() {
                if time < vertical_time {
                    vertical_time = time;
                    vertical_collision = vertical;
                }                
            }

        }

        for platform in platforms {
            
            let (time, _, vertical) = self.dynamic_entity.swept_collision(&platform.entity, entity::CollisionType::Platform);
            
            if self.dynamic_entity.dy > 0.0 && vertical.is_some() {
                if self.holding_down {
                    self.holding_down = false;
                } else if time < vertical_time {
                    vertical_time = time;
                    vertical_collision = vertical;
                }                
            }

        }

        if let entity::HorizontalCollision::None = horizontal_collision {
            horizontal_collision = self.dynamic_entity.horizontal_bounds_collision();
        }

        if let entity::VerticalCollision::None = vertical_collision {
            vertical_collision = self.dynamic_entity.vertical_bounds_collision();
        }

        match horizontal_collision {
            entity::HorizontalCollision::None => {

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
            entity::HorizontalCollision::Left(collision_entity) => {

                self.dynamic_entity.entity.x = if let entity::CollisionEntity::Bounds = collision_entity {
                    game::BOUNDS_X_MIN
                } else {
                    let entity: &entity::Entity = collision_entity.entity().unwrap();
                    entity.x + entity.width
                };
            
                self.dynamic_entity.dx = 0.0;

            },
            entity::HorizontalCollision::Right(collision_entity) => {

                self.dynamic_entity.entity.x = if let entity::CollisionEntity::Bounds = collision_entity {
                    game::BOUNDS_X_MAX - self.dynamic_entity.entity.width
                } else {
                    let entity: &entity::Entity = collision_entity.entity().unwrap();
                    entity.x - self.dynamic_entity.entity.width
                };

                self.dynamic_entity.dx = 0.0;

            },
        }

        match vertical_collision {
            entity::VerticalCollision::None => {

                self.dynamic_entity.entity.y += self.dynamic_entity.dy;

                if self.coyote_ticks > 0 {
                    self.coyote_ticks -= 1;
                }

            }
            entity::VerticalCollision::Down(collision_entity) => {

                self.dynamic_entity.entity.y = if let entity::CollisionEntity::Bounds = collision_entity {
                    game::BOUNDS_Y_MAX - self.dynamic_entity.entity.height
                } else {
                    let entity: &entity::Entity = collision_entity.entity().unwrap();
                    entity.y - self.dynamic_entity.entity.height
                };

                self.dynamic_entity.dy = 0.0;

                if self.jump_buffer_ticks > 0 {
                    self.jump();
                } else {
                    self.coyote_ticks = Self::COYOTE_TICKS;
                }

            },
            entity::VerticalCollision::Up(collision_entity) => {

                self.dynamic_entity.entity.y = if let entity::CollisionEntity::Bounds = collision_entity {
                    game::BOUNDS_Y_MIN
                } else {
                    let entity: &entity::Entity = collision_entity.entity().unwrap();
                    entity.y + entity.height
                };

                self.dynamic_entity.dy = 0.0;

            },
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

    }

    fn fall(&mut self) {
        self.dynamic_entity.dy += game::GRAVITY;
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