

#[derive(Debug)]
pub enum HorizontalCollision {
    None,
    Left,
    Right,
}

#[derive(Debug)]
pub enum VerticalCollision {
    None,
    Up,
    Down,
}

#[derive(Debug)]
pub struct StaticEntity {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

#[derive(Debug)]
pub struct DynamicEntity {
    pub x: u8,
    pub y: u8,
    pub dx: i8,
    pub dy: i8,
    pub width: u8,
    pub height: u8,
    pub weight: i8,
}

// could refactor to only check left or right / up or down side
// this would allow you to check self.dx.cmp one for bounds and static entities
// might not work for dynamic entities, might have to check side
impl DynamicEntity {

    pub fn horizontal_bounds_collision(&mut self) -> HorizontalCollision {

        let mut collision: HorizontalCollision = HorizontalCollision::None;

        match self.dx.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let bounds_collision: bool = u8::MAX - self.width - (self.dx as u8) < self.x;

                if bounds_collision {
                    collision = HorizontalCollision::Right;
                    self.x = u8::MAX - self.width;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.x <= (self.dx * -1) as u8;

                if bounds_collision {
                    collision = HorizontalCollision::Left;
                    self.x = 0;
                }

            }
        }

        return collision;

    }

    pub fn vertical_bounds_collision(&mut self) -> VerticalCollision {

        let mut collision: VerticalCollision = VerticalCollision::None;

        match self.dy.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let bounds_collision: bool = u8::MAX - self.height - (self.dy as u8) < self.y;

                if bounds_collision {
                    collision = VerticalCollision::Down;
                    self.y = u8::MAX - self.height;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.y <= (self.dy * -1) as u8;

                if bounds_collision {
                    collision = VerticalCollision::Up;
                    self.y = 0;
                }

            }
        }

        return collision;

    }

    pub fn hortizonal_static_collision(&mut self, other: &StaticEntity) -> HorizontalCollision {

        let mut collision: HorizontalCollision = HorizontalCollision::None;

        match self.dx.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let static_collision: bool = 
                    self.x + (self.dx as u8) < other.x + other.width &&
                    self.x + (self.dx as u8) + self.width > other.x &&
                    self.y < other.y + other.height &&
                    self.y + self.height > other.y;

                if static_collision {
                    collision = HorizontalCollision::Right;
                    self.x = other.x - self.width;
                }

            }
            std::cmp::Ordering::Less => {

                let static_collision: bool = 
                    self.x - ((self.dx * -1) as u8) < other.x + other.width &&
                    self.x - ((self.dx * -1) as u8) + self.width > other.x &&
                    self.y < other.y + other.height &&
                    self.y + self.height > other.y;

                if static_collision {
                    collision = HorizontalCollision::Left;
                    self.x = other.x + other.width;
                }

            }
        }

        return collision;

    }

    pub fn vertical_static_collision(&mut self, other: &StaticEntity) -> VerticalCollision {
        
        let mut collision: VerticalCollision = VerticalCollision::None;

        match self.dy.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let static_collision: bool = 
                    self.x < other.x + other.width &&
                    self.x + self.width > other.x &&
                    self.y + (self.dy as u8) < other.y + other.height &&
                    self.y + (self.dy as u8) + self.height > other.y;

                if static_collision {
                    collision = VerticalCollision::Down;
                    self.y = other.y - self.height;
                }

            }
            std::cmp::Ordering::Less => {

                let static_collision: bool = 
                    self.x < other.x + other.width &&
                    self.x + self.width > other.x &&
                    self.y - ((self.dy * -1) as u8) < other.y + other.height &&
                    self.y - ((self.dy * -1) as u8) + self.height > other.y;

                if static_collision {
                    collision = VerticalCollision::Up;
                    self.y = other.y + other.height;
                }

            }
        }

        return collision;

    }

}