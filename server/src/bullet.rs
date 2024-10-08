use crate::{ ray, room, user::{self, User} };

pub struct Bullet {
    pub user_idx: usize, 
    pub target_user_idx: usize,
    pub room_idx: usize,
    pub ray: ray::Ray,
}

pub struct BulletPath {
    pub origin_x: f32,
    pub origin_y: f32, 
    pub end_x: f32, 
    pub end_y: f32, 
}

impl Bullet {

    pub fn tick(&self, users: &mut [Option<user::User>]) -> BulletPath {

        let room: &room::Room = &room::ROOMS[self.room_idx];

        let mut intersection: Option<ray::Intersection> = None;
        let mut min_distance: f32 = f32::INFINITY;

        for idx in 0..users.len() {

            if idx == self.user_idx {
                continue;
            }

            let user: &user::User = match users[idx].as_ref() {
                Some(user) => user,
                None => continue,
            };

            if user.room_idx != self.room_idx {
                continue;
            }

            let intersection_distance: Option<f32> = self.ray.intersection(&user.dynamic_entity.entity);

            if let Some(distance) = intersection_distance {
                if distance < min_distance {
                    intersection = Some(ray::Intersection { distance, variant: ray::IntersectionVariant::User(idx) });
                    min_distance = distance;
                }
            }

        }

        for entity in room.platforms {

            let intersection_distance: Option<f32> = self.ray.intersection(entity);

            if let Some(distance) = intersection_distance {
                if distance < min_distance {
                    intersection = Some(ray::Intersection { distance, variant: ray::IntersectionVariant::Platform });
                    min_distance = distance;
                }
            }

        }

        let distance: f32 = match intersection {
            //None => f32::max(room.bounds.x_max, room.bounds.y_max) * std::f32::consts::SQRT_2,
            None => 200.0,
            Some(ray::Intersection { variant, distance }) => {

                if let ray::IntersectionVariant::User(idx) = variant {

                    if idx == self.target_user_idx {

                        // add code to take the target's target
                        
                        let killed_user: user::User = users[idx].take().unwrap();

                        users[self.user_idx].as_mut().unwrap().target_user_idx = killed_user.target_user_idx;
                        
                        // respawn shot user
                        //user.respawn(user.room_idx);
                        users[idx] = Some(user::User::new(
                            killed_user.idx, 
                            killed_user.room_idx, 
                            User::get_target_idx(users, idx), 
                            killed_user.send_to_client,
                        ));

                    }

                }

                distance

            }
        };

        BulletPath::from_bullet(self, distance)

    }

}

impl BulletPath {

    fn from_bullet(bullet: &Bullet, magnitude: f32) -> Self {

        let origin_x: f32 = bullet.ray.origin_x;
        let origin_y: f32 = bullet.ray.origin_y;

        let end_x: f32 = origin_x + magnitude * bullet.ray.direction_x;
        let end_y: f32 = origin_y + magnitude * bullet.ray.direction_y;

        Self {
            origin_x,
            origin_y,
            end_x,
            end_y,
        }

    }
}