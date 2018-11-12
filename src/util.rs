use body::Body;
use vector::Vector;
use point::Point;
use transformation::Transformation;
use rand::prelude::*;
use std::f64::consts::PI;

pub struct Distributor {
    pub num_bodies: u32,
    pub min_dist: u32,
    pub max_dist: u32,
    pub dy: f64,
}

impl Distributor {
    /**
     *  Returns a distribution of bodies.
     */
    pub fn get(&self) -> Vec<Body> {
        let mut result: Vec<Body> = vec![];
        let mut angle_rand = thread_rng();
        let mut dist_rand = thread_rng();

        for i in 0..self.num_bodies {
            let angle = angle_rand.gen_range(0.0, 2.0 * PI);
            let dist = dist_rand.gen_range(self.min_dist, self.max_dist);

            let trans = Transformation::rotation(angle);
            let position = &trans * Vector { dx: dist as f64, dy: 0.0 };
            let velocity = &trans * Vector { dx: 0.0, dy: self.dy };

            // create body
            let body = Body {
                id: i,
                mass: 0.1,
                position: Point { x: position.dx as i32, y: position.dy as i32 },
                velocity: velocity,
            };

            result.push(body);
        }

        result
    }
}