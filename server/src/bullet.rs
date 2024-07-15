use crate::{ entity, platform, ray, user };

pub struct Bullet {
    pub user_idx: usize, 
    pub ray: ray::Ray, 
}

impl Bullet {

    pub fn tick(&self, users: &mut [Option<user::User>], platforms: &[platform::Platform]) {

        let mut closest_intersection: Option<ray::Intersection> = None;
        let mut min_time: f32 = f32::NEG_INFINITY;

        for idx in 0..users.len() {

            if idx == self.user_idx {
                continue;
            }

            let user: Option<&user::User> = users[idx].as_ref();
            
            if user.is_none() {
                continue;
            }

            let entity: &entity::Entity = &user.unwrap().dynamic_entity.entity;

            let intersection: Option<ray::Intersection> = self.ray.intersection(entity, ray::IntersectionType::User(idx));

            if let Some(ray::Intersection { time, .. }) = intersection {
                if time > min_time {
                    closest_intersection = intersection;
                    min_time = time;
                }
            }

        }

        for entity in platforms.iter().map(|platform| &platform.entity) {

            let intersection: Option<ray::Intersection> = self.ray.intersection(entity, ray::IntersectionType::Platform);

            if let Some(ray::Intersection { time, .. }) = intersection {
                if time > min_time {
                    closest_intersection = intersection;
                    min_time = time;
                }
            }

        }

        if let Some(ray::Intersection { intersection_type, .. }) = closest_intersection {
            if let ray::IntersectionType::User(idx) = intersection_type {
                // kill user
                users[idx] = None;
            }
        }

    }

}