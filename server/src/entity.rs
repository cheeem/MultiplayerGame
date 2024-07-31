use crate::room;

pub enum CollisionVariant<'a> {
    Bounds,
    User(&'a Entity),
    Platform(&'a Entity),
    Door(&'a room::Door),
}

#[derive(Debug)]
pub enum HorizontalCollisionDirection {
    Left,
    Right,
}

#[derive(Debug)]
pub enum VerticalCollisionDirection {
    Up,
    Down,
}

pub struct HorizontalCollision<'a> {
    pub variant: CollisionVariant<'a>, 
    pub direction: HorizontalCollisionDirection, 
    pub time: f32,
}

pub struct VerticalCollision<'a> {
    pub variant: CollisionVariant<'a>,
    pub direction: VerticalCollisionDirection, 
    pub time: f32,
}

#[derive(Debug)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct DynamicEntity {
    pub entity: Entity,
    pub dx: f32,
    pub dy: f32,
    pub weight: f32,
}

impl DynamicEntity {

    pub fn horizontal_bounds_collision(&self, bounds: &'static room::Bounds) -> Option<HorizontalCollision<'static>> {

        match self.dx.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Greater) => {

                let bounds_collision: bool = bounds.x_max - self.entity.width - self.dx < self.entity.x;

                if bounds_collision {
                    return Some(HorizontalCollision { 
                        variant: CollisionVariant::Bounds,
                        direction: HorizontalCollisionDirection::Right,
                        time: f32::INFINITY,
                    })
                }

                None

            }
            Some(std::cmp::Ordering::Less) => {

                let bounds_collision: bool = self.entity.x <= self.dx * -1.0;

                if bounds_collision {
                    return Some(HorizontalCollision {
                        variant: CollisionVariant::Bounds,
                        direction: HorizontalCollisionDirection::Left,
                        time: f32::INFINITY,
                    })
                } 

                None

            }
            _ => None,
        }

    }

    pub fn vertical_bounds_collision(&self, bounds: &'static room::Bounds) -> Option<VerticalCollision<'static>> {

        match self.dy.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Greater) => {

                let bounds_collision: bool = bounds.y_max - self.entity.height - self.dy < self.entity.y;

                if bounds_collision {
                    return Some(VerticalCollision {
                        variant: CollisionVariant::Bounds,
                        direction: VerticalCollisionDirection::Down,
                        time: f32::INFINITY,
                    })
                }

                None

            }
            Some(std::cmp::Ordering::Less) => {

                let bounds_collision: bool = self.entity.y <= self.dy * -1.0;

                if bounds_collision {
                    return Some(VerticalCollision {
                        variant: CollisionVariant::Bounds,
                        direction: VerticalCollisionDirection::Up,
                        time: f32::INFINITY,
                    })
                } 

                None

            }
            _ => None,
        }

    }

    // pub fn horizonal_static_collision(&mut self, other: &StaticEntity) -> HorizontalCollision {

    //     let mut collision: HorizontalCollision = HorizontalCollision::None;

    //     match self.dx.cmp(&0) {
    //         std::cmp::Ordering::Equal => (), 
    //         std::cmp::Ordering::Greater => {

    //             let static_collision: bool = 
    //                 self.x + (self.dx as u8) < other.x + other.width &&
    //                 self.x + (self.dx as u8) + self.width > other.x &&
    //                 self.y < other.y + other.height &&
    //                 self.y + self.height > other.y;

    //             if static_collision {
    //                 collision = HorizontalCollision::Right;
    //                 self.x = other.x - self.width;
    //             }

    //         }
    //         std::cmp::Ordering::Less => {

    //             let static_collision: bool = 
    //                 self.x - ((self.dx * -1) as u8) < other.x + other.width &&
    //                 self.x - ((self.dx * -1) as u8) + self.width > other.x &&
    //                 self.y < other.y + other.height &&
    //                 self.y + self.height > other.y;

    //             if static_collision {
    //                 collision = HorizontalCollision::Left;
    //                 self.x = other.x + other.width;
    //             }

    //         }
    //     }

    //     return collision;

    // }

    // pub fn vertical_static_collision(&mut self, other: &StaticEntity) -> VerticalCollision {
        
    //     let mut collision: VerticalCollision = VerticalCollision::None;

    //     match self.dy.cmp(&0) {
    //         std::cmp::Ordering::Equal => (), 
    //         std::cmp::Ordering::Greater => {

    //             let static_collision: bool = 
    //                 self.x < other.x + other.width &&
    //                 self.x + self.width > other.x &&
    //                 self.y + (self.dy as u8) < other.y + other.height &&
    //                 self.y + (self.dy as u8) + self.height > other.y;

    //             if static_collision {
    //                 collision = VerticalCollision::Down;
    //                 self.y = other.y - self.height;
    //             }

    //         }
    //         std::cmp::Ordering::Less => {

    //             let static_collision: bool = 
    //                 self.x < other.x + other.width &&
    //                 self.x + self.width > other.x &&
    //                 self.y - ((self.dy * -1) as u8) < other.y + other.height &&
    //                 self.y - ((self.dy * -1) as u8) + self.height > other.y;

    //             if static_collision {
    //                 collision = VerticalCollision::Up;
    //                 self.y = other.y + other.height;
    //             }

    //         }
    //     }

    //     return collision;

    // }

    pub fn swept_collision<'a, 'b>(&'a self, other: &'b Entity) -> (f32, Option<HorizontalCollisionDirection>, Option<VerticalCollisionDirection>) {

        let (x_entry_time, x_exit_time) = if self.dx == 0.0 {
            
            if self.entity.x < other.x + other.width && other.x < self.entity.x + self.entity.width {
                (
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                )
            } else {
                return (
                    f32::INFINITY, 
                    None,
                    None,
                )
            }

        } else {
            
            let (x_entry_distance, x_exit_distance) = if self.dx > 0.0 {
                (
                    other.x - (self.entity.x + self.entity.width),
                    other.x + other.width - self.entity.x,
                )
            } else {
                (
                    self.entity.x - (other.x + other.width),
                    self.entity.x + self.entity.width - other.x,
                )
            };

            (
                x_entry_distance / self.dx.abs(),
                x_exit_distance / self.dx.abs(),
            )

        };

        let (y_entry_time, y_exit_time) = if self.dy == 0.0 {

            if self.entity.y < other.y + other.height && other.y < self.entity.y + self.entity.height {
                (
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                )
            } else {
                return (
                    f32::INFINITY,
                    None,
                    None,
                )
            }

        } else {

            let (y_entry_distance, y_exit_distance) = if self.dy > 0.0 {
                (
                    other.y - (self.entity.y + self.entity.height),
                    other.y + other.height - self.entity.y,
                )
            } else {
                (
                    self.entity.y - (other.y + other.height),
                    self.entity.y + self.entity.height - other.y,
                )
            };

            (
                y_entry_distance / self.dy.abs(),
                y_exit_distance / self.dy.abs(),
            )

        };

        if x_entry_time > y_exit_time || y_entry_time > x_exit_time {
            return (
                f32::INFINITY,
                None,
                None,
            )
        }

        let entry_time: f32 = f32::max(x_entry_time, y_entry_time);

        if entry_time < 0.0 || entry_time > 1.0 {
            return (
                entry_time,
                None,
                None,
            )
        }

        if x_entry_time > y_entry_time {
            if self.dx > 0.0 {
                (
                    entry_time, 
                    Some(HorizontalCollisionDirection::Right), 
                    None
                )
            } else {
                (
                    entry_time, 
                    Some(HorizontalCollisionDirection::Left), 
                    None
                )
            }
        } else {
            if self.dy > 0.0 {
                (
                    entry_time, 
                    None, 
                    Some(VerticalCollisionDirection::Down)
                )
            } else {
                (
                    entry_time, 
                    None, 
                    Some(VerticalCollisionDirection::Up)
                )
            }
        }

    }

}