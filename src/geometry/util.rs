use physics::types::Body;
use geometry::types::{ Point, Vector};
use rand::prelude::*;
use std::f64::consts::PI;
use std::ops::Mul;

// Transformation ////////////////////////////////////////////////////////////
//
// A 2D transformation matrix represented as a pair of transformed basis
// vectors.

// TODO: make this a data tuple
pub struct Transformation {
    pub a: Vector,
    pub b: Vector,
}

impl<'a> Mul<Vector> for &'a Transformation {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        &self.a * rhs.dx + &self.b * rhs.dy
    }
}

impl Transformation {
    pub fn rotation(radians: f64) -> Transformation {
        let (sin, cos) = radians.sin_cos();
        Transformation {
            a: Vector { dx: cos, dy: sin },
            b: Vector { dx: -sin, dy: cos },
        }
    }
}

// Distributor ///////////////////////////////////////////////////////////////
//
// // TODO: description

pub struct Distributor {
    pub num_bodies: u32,
    pub min_dist: u32,
    pub max_dist: u32,
    pub dy: f64,
}

impl Distributor {
    pub fn distribution(&self) -> Vec<Body> {
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

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::types::Vector;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn it_transforms_a_vector() {
        // given
        let sut = Transformation {
            a: Vector { dx: 2.0, dy: 0.0 },
            b: Vector { dx: 0.0, dy: 2.0 },
        };

        // when
        let result = &sut * Vector { dx: 4.0, dy: -2.5 };

        // then
        assert_eq!(result, Vector { dx: 8.0, dy: -5.0 });
    }

    #[test]
    fn it_rotates_a_vector() {
        // given
        let sut = Transformation::rotation(FRAC_PI_2);

        // when
        let result = &sut * Vector { dx: 1.0, dy: 0.0 };

        // then
        assert_eq!(result, Vector { dx: 0.0, dy: 1.0 });
    }
}