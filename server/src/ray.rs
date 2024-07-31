use crate::entity;

pub enum IntersectionVariant {
    User(usize),
    Platform,
}

pub struct Intersection {
    pub variant: IntersectionVariant,
    pub distance: f32,
}

pub struct Ray {
    pub origin_x: f32,
    pub origin_y: f32,
    pub direction_x: f32,
    pub direction_y: f32,
}

impl Ray {

    pub fn from_entity_and_position(entity: &entity::Entity, x: f32, y: f32) -> Self {

        let origin_x: f32 = entity.x + entity.width * 0.5;
        let origin_y: f32 = entity.y + entity.height * 0.5;

        let distance_x: f32 = x - origin_x;
        let distance_y: f32 = y - origin_y;

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

    pub fn intersection(&self, entity: &entity::Entity) -> Option<f32> {

        let mut min_distance: f32 = f32::NEG_INFINITY;
        let mut max_distance: f32 = f32::INFINITY;

        if self.direction_x != 0.0 {

            let distance_x_1: f32 = (entity.x - self.origin_x) / self.direction_x;
            let distance_x_2: f32 = (entity.x + entity.width - self.origin_x) / self.direction_x;

            min_distance = min_distance.max(distance_x_1.min(distance_x_2));
            max_distance = max_distance.min(distance_x_1.max(distance_x_2));

        };

        if self.direction_y != 0.0 {

            let distance_y_1: f32 = (entity.y - self.origin_y) / self.direction_y;
            let distance_y_2: f32 = (entity.y + entity.height - self.origin_y) / self.direction_y;

            min_distance = min_distance.max(distance_y_1.min(distance_y_2));
            max_distance = max_distance.min(distance_y_1.max(distance_y_2));

        };

        if min_distance > max_distance {
            return None;
        }

        if max_distance < 0.0 {
            return None;
        }

        return Some(max_distance);

    }

}