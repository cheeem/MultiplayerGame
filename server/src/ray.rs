use crate::entity;

pub enum IntersectionType {
    User(usize),
    Platform,
}

pub struct Intersection {
    pub intersection_type: IntersectionType,
    pub time: f32,
}

pub struct Ray {
    origin_x: f32,
    origin_y: f32,
    direction_x: f32,
    direction_y: f32,
}

impl Ray {

    pub fn from_click_position(entity: &entity::Entity, x: f32, y: f32) -> Self {

        let origin_x: f32 = entity.x + entity.width * 0.5;
        let origin_y: f32 = entity.y + entity.height * 0.5;

        let distance_x: f32 = origin_x - x;
        let distance_y: f32 = origin_y - y;

        let magnitude: f32 = f32::sqrt(distance_x * distance_x + distance_y * distance_y);

        let direction_x: f32 = distance_x / magnitude;
        let direction_y: f32 = distance_y / magnitude; 

        return Self {
            origin_x,
            origin_y,
            direction_x, 
            direction_y,
        }

    }

    pub fn intersection(&self, entity: &entity::Entity, intersection_type: IntersectionType) -> Option<Intersection> {

        let mut min_time: f32 = f32::NEG_INFINITY;
        let mut max_time: f32 = f32::INFINITY;

        if self.direction_x != 0.0 {

            let time_x_1: f32 = (entity.x - self.origin_x) / self.direction_x;
            let time_x_2: f32 = (entity.x + entity.width - self.origin_x) / self.direction_x;

            min_time = min_time.max(time_x_1.min(time_x_2));
            max_time = max_time.min(time_x_1.max(time_x_2));

        };

        if self.direction_y != 0.0 {

            let time_y_1: f32 = (entity.y - self.origin_y) / self.direction_y;
            let time_y_2: f32 = (entity.y + entity.height - self.origin_y) / self.direction_y;

            min_time = min_time.max(time_y_1.min(time_y_2));
            max_time = max_time.min(time_y_1.max(time_y_2));

        };

        if min_time > max_time {
            return None;
        }

        return Some(Intersection { intersection_type, time: max_time });

    }

}