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
    // channels
    pub send_to_client: mpsc::Sender<Vec<u8>>,
    // entity
    pub entity: entity::DynamicEntity,
    // controls & state
    pub jump_buffer_ticks: u8,
    coyote_ticks: u8,
    pub holding_left: bool, 
    pub holding_right: bool, 
}

impl User {

    pub const JUMP_BUFFER_TICKS: u8 = 5;
    const COYOTE_TICKS: u8 = 3;

    const JUMP_FORCE: i8 = -40;
    const JUMP_CUTOFF: i8 = 2;

    const RUN_START_FORCE: i8 = 2;
    const RUN_END_FORCE: i8 = 4;
    const RUN_MAX_SPEED: i8 = 8;

    pub fn new(send_to_client: mpsc::Sender<Vec<u8>>) -> Self {

        let entity: entity::DynamicEntity = entity::DynamicEntity {
            x: 0,
            y: 0,
            dx: 0,
            dy: game::GRAVITY,
            width: 10, 
            height: 10, 
            weight: 2,
        };

        Self {
            // channels
            send_to_client,
            // entity
            entity,
            // control & state
            jump_buffer_ticks: 0,
            coyote_ticks: 0,          
            holding_left: false,
            holding_right: false,
        }

    }

    pub fn tick(&mut self, platforms: &[platform::Platform]) {

        // let hortizontal_movement_direction: HortizontalMovementDirection = match self.dx.cmp(&0) {
        //     std::cmp::Ordering::Equal => HortizontalMovementDirection::None,
        //     std::cmp::Ordering::Greater => HortizontalMovementDirection::Right,
        //     std::cmp::Ordering::Less => HortizontalMovementDirection
        // }

        let mut horizontal_collision: entity::HorizontalCollision = self.entity.horizontal_bounds_collision();
        let mut vertical_collision: entity::VerticalCollision = self.entity.vertical_bounds_collision();

        for platform in platforms {
            match (&horizontal_collision, &vertical_collision) {
                (entity::HorizontalCollision::None, entity::VerticalCollision::None) => {
                    horizontal_collision = self.entity.hortizonal_static_collision(&platform.entity);
                    vertical_collision = self.entity.vertical_static_collision(&platform.entity);
                }
                (entity::HorizontalCollision::None, _) => {
                    horizontal_collision = self.entity.hortizonal_static_collision(&platform.entity);
                }
                (_, entity::VerticalCollision::None) => {
                    vertical_collision = self.entity.vertical_static_collision(&platform.entity);
                }
                _ => {
                    break;
                }
            }
        }

        match horizontal_collision {
            entity::HorizontalCollision::None => {
                
                match self.entity.dx.cmp(&0) {
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => {

                        self.entity.x += self.entity.dx as u8;

                        if self.holding_right == false {
                            self.end_run_right();
                        }
    
                    }
                    std::cmp::Ordering::Less => {

                        self.entity.x -= (self.entity.dx * -1) as u8;

                        if self.holding_left == false {
                            self.end_run_left();
                        }
    
                    }
                }

            }
            entity::HorizontalCollision::Left => {
                self.entity.dx = 0;
            },
            entity::HorizontalCollision::Right => {
                self.entity.dx = 0;
            },
        }

        match vertical_collision {
            entity::VerticalCollision::None => {

                match self.entity.dy.cmp(&0) {
                    std::cmp::Ordering::Equal => (),
                    std::cmp::Ordering::Greater => {
                        self.entity.y += self.entity.dy as u8;
                    }
                    std::cmp::Ordering::Less => {
                        self.entity.y -= (self.entity.dy * -1) as u8;
                    }
                }

                if self.coyote_ticks > 0 {
                    self.coyote_ticks -= 1;
                }

            }
            entity::VerticalCollision::Down => {

                self.entity.dy = 0;

                if self.jump_buffer_ticks > 0 {
                    self.jump();
                } else {
                    self.coyote_ticks = Self::COYOTE_TICKS;
                }

            },
            entity::VerticalCollision::Up => {
                self.entity.dy = 0;
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
        self.entity.dy += game::GRAVITY;
    }

    fn jump(&mut self) {
        self.entity.dy = Self::JUMP_FORCE / self.entity.weight;
        self.coyote_ticks = 0;
        self.jump_buffer_ticks = 0;
    }

    pub fn end_jump(&mut self) {
        // maybe check if actually jumping?
        self.entity.dy /= Self::JUMP_CUTOFF;
    }

    fn run_left(&mut self) {
        self.entity.dx = std::cmp::max(
            -Self::RUN_MAX_SPEED, 
            self.entity.dx - Self::RUN_START_FORCE / self.entity.weight,
        );
    }

    fn end_run_left(&mut self) {
        self.entity.dx = std::cmp::min(
            0, 
            self.entity.dx + Self::RUN_END_FORCE / self.entity.weight,
        ); 
    }

    fn run_right(&mut self) {
        self.entity.dx = std::cmp::min(
            Self::RUN_MAX_SPEED, 
            self.entity.dx + Self::RUN_START_FORCE / self.entity.weight,
        );
    }

    fn end_run_right(&mut self) {
        self.entity.dx = std::cmp::max(
            0, 
            self.entity.dx - Self::RUN_END_FORCE / self.entity.weight,
        ); 
    }

}