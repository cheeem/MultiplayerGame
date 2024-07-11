pub enum EntityType {
    User, 
    Platform,
}

#[derive(Debug)]
pub enum CollisionEntity<'a> {
    Bounds,
    User(&'a Entity),
    Platform(&'a Entity),
}

#[derive(Debug)]
pub enum HorizontalCollision<'a> {
    None,
    Left(CollisionEntity<'a>),
    Right(CollisionEntity<'a>),
}

#[derive(Debug)]
pub enum VerticalCollision<'a> {
    None,
    Up(CollisionEntity<'a>),
    Down(CollisionEntity<'a>),
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

impl EntityType {
    fn to_collision_entity<'a, 'b>(&'a self, entity: &'b Entity) -> CollisionEntity<'b> {
        match self {
            Self::User => CollisionEntity::User(entity), 
            Self::Platform => CollisionEntity::Platform(entity), 
        }
    }
}

impl<'a> CollisionEntity<'a> {
    pub fn entity(&self) -> Option<&Entity> {
        match self {
            Self::Bounds => None,
            Self::User(entity) => Some(entity),
            Self::Platform(entity) => Some(entity),
        }
    }
}

impl<'a> HorizontalCollision<'a> {
    pub fn is_some(&self) -> bool {
        match self{
            Self::None => false,
            _ => true,
        }
    }
}

impl<'a> VerticalCollision<'a> {
    pub fn is_some(&self) -> bool {
        match self{
            Self::None => false,
            _ => true,
        }
    }
}

impl DynamicEntity {

    pub fn horizontal_bounds_collision(&self) -> HorizontalCollision<'static> {

        let mut collision: HorizontalCollision = HorizontalCollision::None;

        match self.dx.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Greater) => {

                let bounds_collision: bool = 255.0 - self.entity.width - self.dx < self.entity.x;

                if bounds_collision {
                    collision = HorizontalCollision::Right(CollisionEntity::Bounds);
                }

            }
            Some(std::cmp::Ordering::Less) => {

                let bounds_collision: bool = self.entity.x <= self.dx * -1.0;

                if bounds_collision {
                    collision = HorizontalCollision::Left(CollisionEntity::Bounds);
                }

            }
            _ => (),
        }

        return collision;

    }

    pub fn vertical_bounds_collision(&self) -> VerticalCollision<'static> {

        let mut collision: VerticalCollision = VerticalCollision::None;

        match self.dy.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Greater) => {

                let bounds_collision: bool = 255.0 - self.entity.height - self.dy < self.entity.y;

                if bounds_collision {
                    collision = VerticalCollision::Down(CollisionEntity::Bounds);
                }

            }
            Some(std::cmp::Ordering::Less) => {

                let bounds_collision: bool = self.entity.y <= self.dy * -1.0;

                if bounds_collision {
                    collision = VerticalCollision::Up(CollisionEntity::Bounds);
                }

            }
            _ => (),
        }

        return collision;

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

    pub fn swept_collision<'a, 'b>(&'a self, other: &'b Entity, entity_type: EntityType) -> (f32, HorizontalCollision<'b>, VerticalCollision<'b>) {

        let horizontal_collision: HorizontalCollision = HorizontalCollision::None;
        let vertical_collision: VerticalCollision = VerticalCollision::None;
        let entry_time: f32 = 1.0;

        let (x_entry_time, x_exit_time) = if self.dx == 0.0 {
            
            if self.entity.x < other.x + other.width && other.x < self.entity.x + self.entity.width {
                (
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                )
            } else {
                return (
                    entry_time, 
                    horizontal_collision,
                    vertical_collision,
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
                    entry_time,
                    horizontal_collision,
                    vertical_collision,
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
                entry_time,
                horizontal_collision,
                vertical_collision,
            )
        }

        let entry_time: f32 = if x_entry_time > y_entry_time {
            x_entry_time
        } else {
            y_entry_time
        };

        if entry_time < 0.0 || entry_time > 1.0 {
            return (
                entry_time,
                horizontal_collision,
                vertical_collision,
            )
        }

        let (horizontal_collision, vertical_collision) = if x_entry_time > y_entry_time {
            if self.dx > 0.0 {
                // (-1, 0)
                (HorizontalCollision::Right(entity_type.to_collision_entity(other)), VerticalCollision::None)
            } else {
                // (1, 0)
                (HorizontalCollision::Left(entity_type.to_collision_entity(other)), VerticalCollision::None)
            }
        } else {
            if self.dy > 0.0 {
                // (0, -1)
                (HorizontalCollision::None, VerticalCollision::Down(entity_type.to_collision_entity(other)))
            } else {
                // (0, 1)
                (HorizontalCollision::None, VerticalCollision::Up(entity_type.to_collision_entity(other)))
            }
        };

        return (
            entry_time,
            horizontal_collision,
            vertical_collision,
        )

    }

    // pub fn swept_collision(&self, other: &Entity) -> (f32, HorizontalCollision, VerticalCollision) {

    //     let (x_entry_distance, x_exit_distance) = if self.dx > 0.0 { 
    //         (
    //             other.x - (self.entity.x + self.entity.width),
    //             (other.x + other.width) - self.entity.x,
    //         )
    //     } else {
    //         (
    //             (other.x + other.width) - self.entity.x,
    //             other.x - (self.entity.x + self.entity.width),
    //         ) 
    //     };

    //     let (y_entry_distance, y_exit_distance) = if self.dy > 0.0 { 
    //         (
    //             other.y - (self.entity.y + self.entity.height), 
    //             (other.y + other.height) - self.entity.y,
    //         )
    //     } else { 
    //         (
    //             (other.y + other.height) - self.entity.y,
    //             other.y - (self.entity.y + self.entity.height),
    //         )  
    //     };

    //     let (mut x_entry_time, x_exit_time) = if self.dx == 0.0 { 
    //         (
    //             std::f32::NEG_INFINITY,
    //             std::f32::INFINITY,
    //         )
    //     } else {
    //         (
    //             x_entry_distance / self.dx,
    //             x_exit_distance / self.dx,
    //         )
    //     };

    //     let (mut y_entry_time, y_exit_time) = if self.dy == 0.0 {
    //         (
    //             std::f32::NEG_INFINITY,
    //             std::f32::INFINITY,
    //         )
    //     } else { 
    //         (
    //             y_entry_distance / self.dy,
    //             y_exit_distance / self.dy, 
    //         )
    //     };

    //     if x_entry_time > 1.0 {
    //         x_entry_time = f32::NEG_INFINITY;
    //     }
        
    //     if y_entry_time > 1.0 {
    //         y_entry_time = f32::NEG_INFINITY;
    //     }

    //     let entry_time: f32 = if x_entry_time > y_entry_time {
    //         x_entry_time
    //     } else {
    //         y_entry_time
    //     };

    //     let exit_time: f32 = if x_exit_time < y_exit_time {
    //         x_exit_time
    //     } else {
    //         y_exit_time
    //     };

    //     let no_collision: bool = 
    //         (entry_time > exit_time) ||
    //         (x_entry_time < 0.0 && y_entry_time < 0.0) || 
    //         // Check that the bounding box started overlapped or not.
    //         (x_entry_time < 0.0 && (self.entity.x + self.entity.width < other.x || self.entity.x > other.x + other.width)) ||
    //         // Check that the bounding box started overlapped or not.
    //         (y_entry_time < 0.0 && (self.entity.y + self.entity.height < other.y || self.entity.y > other.y + other.height));

    //     // let no_collision: bool = entry_time > exit_time || x_entry_time < 0.0 && y_entry_time < 0.0 || x_entry_time > 1.0 || y_entry_time > 1.0;

    //     if no_collision {
    //         return (
    //             1.0,
    //             HorizontalCollision::None,
    //             VerticalCollision::None,
    //         );
    //     }

    //     let mut horizontal_collision: HorizontalCollision = HorizontalCollision::None;
    //     let mut vertical_collision: VerticalCollision = VerticalCollision::None;

    //     if x_entry_time > y_entry_time {

    //         horizontal_collision = if x_entry_distance > 0.0 {
    //             HorizontalCollision::Right(other)
    //         } else {
    //             HorizontalCollision::Left(other)
    //         }

    //     } else {

    //         vertical_collision = if y_entry_distance > 0.0 {
    //             VerticalCollision::Down(other)
    //         } else {
    //             VerticalCollision::Up(other)
    //         }
            
    //     }

    //     return (
    //         entry_time,
    //         horizontal_collision,
    //         vertical_collision,
    //     );

    // }

}