use physics::types::Body;
use geometry::types::{ Point, Vector};
use rand::prelude::*;
use std::f64::consts::PI;
use std::ops::Mul;

// Transformation ////////////////////////////////////////////////////////////
//
// A 2D transformation matrix represented as a pair of transformed basis
// vectors.

pub struct Transformation(Vector, Vector);

impl<'a> Mul<Vector> for &'a Transformation {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        &self.0 * rhs.dx + &self.1 * rhs.dy
    }
}

impl Transformation {
    pub fn rotation(radians: f64) -> Transformation {
        let (sin, cos) = radians.sin_cos();
        Transformation(
            Vector { dx: cos, dy: sin },
            Vector { dx: -sin, dy: cos },
        )
    }
}

// Distributor ///////////////////////////////////////////////////////////////
//
// A helper object to distribute bodies in space with velcoity. The
// distribution is uses parameterized randomization.

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
        let sut = Transformation(
            Vector { dx: 2.0, dy: 0.0 },
            Vector { dx: 0.0, dy: 2.0 },
        );

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