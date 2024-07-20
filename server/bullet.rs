use crate::{ entity, game, platform, ray, user };

pub struct Bullet {
    pub user_idx: usize, 
    pub ray: ray::Ray, 
    pub end_x: f32,
    pub end_y: f32,
}

impl Bullet {

    pub fn from_click_position(user: &user::User, idx: usize, x: f32, y: f32) -> Self {

        let ray: ray::Ray = ray::Ray::from_click_position(&user.dynamic_entity.entity, x, y);

        println!("{x} {y} {} {}", ray.direction_x, ray.direction_y);

        //let magnitude: f32 = f32::max(game::BOUNDS_X_MAX, game::BOUNDS_Y_MAX) * std::f32::consts::SQRT_2;
        let magnitude: f32 = 100.0;

        let end_x: f32 = ray.origin_x + magnitude * ray.direction_x;
        let end_y: f32 = ray.origin_y + magnitude * ray.direction_y;
        
        Self { 
            user_idx: idx, 
            ray,
            end_x,
            end_y,
        }
        
    }

    pub fn tick(&mut self, users: &mut [Option<user::User>], platforms: &[platform::Platform]) {

        let mut intersection: Option<ray::Intersection> = None;
        let mut min_distance: f32 = f32::INFINITY;

        for idx in 0..users.len() {

            if idx == self.user_idx {
                continue;
            }

            let entity: &entity::Entity = match users[idx].as_ref() {
                Some(user) => &user.dynamic_entity.entity,
                None => continue,
            };

            let intersection_distance: Option<f32> = self.ray.intersection(entity);

            if let Some(distance) = intersection_distance {
                if distance < min_distance {
                    intersection = Some(ray::Intersection { distance, intersection_type: ray::IntersectionType::User(idx) });
                    min_distance = distance;
                }
            }

        }

        for entity in platforms.iter().map(|platform| &platform.entity) {

            let intersection_distance: Option<f32> = self.ray.intersection(entity);

            if let Some(distance) = intersection_distance {
                if distance < min_distance {
                    intersection = Some(ray::Intersection { distance, intersection_type: ray::IntersectionType::Platform });
                    min_distance = distance;
                }
            }

        }

        println!("{:?}\n", intersection.as_ref().map(|i| i.distance));

        if let Some(ray::Intersection { intersection_type, distance }) = intersection {
            
            self.set_end_position(distance);

            // testing grapplling idea here 
            // match users[self.user_idx].as_mut() {
            //     Some(user) => {
            //         // user.dynamic_entity.entity.x = self.end_x;
            //         // user.dynamic_entity.entity.y = self.end_y;

            //         let ticks: f32 = 5.0;

            //         user.dynamic_entity.dx = (self.end_x - user.dynamic_entity.entity.x) / ticks;
            //         user.dynamic_entity.dy = (self.end_y - user.dynamic_entity.entity.y) / ticks - game::GRAVITY * ticks; 
            //     }
            //     None => (),
            // }

            if let ray::IntersectionType::User(idx) = intersection_type {

                let shot_user: user::User = users[idx].take().unwrap();
                // respawn shot user
                users[idx] = Some(user::User::new(shot_user.idx, shot_user.send_to_client));

            }

        }

    }

    fn set_end_position(&mut self, magnitude: f32) {
        self.end_x = self.ray.origin_x + magnitude * self.ray.direction_x;
        self.end_y = self.ray.origin_y + magnitude * self.ray.direction_y;
    }

}