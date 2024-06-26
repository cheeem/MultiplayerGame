pub enum HorizontalCollision {
    None,
    Left,
    Right,
}

pub enum VerticalCollision {
    None,
    Up,
    Down,
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

#[derive(Debug)]
pub struct StaticEntity {
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
}

impl DynamicEntity {

    pub fn horizontal_bounds_collision(&mut self) -> HorizontalCollision {

        let mut collision: HorizontalCollision = HorizontalCollision::None;

        match self.dx.cmp(&0) {
            std::cmp::Ordering::Equal => (), 
            std::cmp::Ordering::Greater => {

                let bounds_collision: bool = u8::MAX - self.width - (self.dx as u8) < self.x;

                if bounds_collision {
                    collision = HorizontalCollision::Right;
                    self.x = u8::MAX - self.width + 1;
                    self.dx = 0;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.x <= (self.dx * -1) as u8;

                if bounds_collision {
                    collision = HorizontalCollision::Left;
                    self.x = 0;
                    self.dx = 0;
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
                    self.y = u8::MAX - self.height + 1;
                    self.dy = 0;
                }

            }
            std::cmp::Ordering::Less => {

                let bounds_collision: bool = self.y <= (self.dy * -1) as u8;

                if bounds_collision {
                    collision = VerticalCollision::Up;
                    self.y = 0;
                    self.dy = 0;
                }

            }
        }

        return collision;

    }

    fn aabb_collision(&self, other: &DynamicEntity) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

}

impl StaticEntity {

    fn aabb_collision(&self, other: &DynamicEntity) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

}