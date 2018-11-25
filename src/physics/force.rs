use super::types::Body;
use geometry::types::{Point, Vector};

// Gravity ///////////////////////////////////////////////////////////////////
//
// Newton's Law of Universal Gravitation.

pub struct Gravity {
    g: f32,
    min_dist: f32,
}

impl Gravity {
    pub fn new(g: f32, min_dist: f32) -> Gravity {
        if min_dist <= 0.0 {
            panic!("The minimum gravitational distance \
            must be greater than 0. Got {}", min_dist);
        }
        Gravity {
            g,
            min_dist,
        }
    }

    pub fn between(&self, b1: &Body, b2: &Body) -> Vector {
        // Force is undefined for two bodies that occupy the same space.
        if b1.position == b2.position {
            return Vector::zero();
        }

        let difference = Vector::difference(&b2.position, &b1.position);
        let distance = difference.magnitude().max(self.min_dist);
        let force = (self.g * b1.mass * b2.mass) / (distance * distance);

        let direction = match difference.normalized() {
            None => Vector::zero(),
            Some(normalized) => normalized,
        };

        &direction * force
    }
}

// Attractor /////////////////////////////////////////////////////////////////
//
// Gravitational attraction to a point.

pub struct Attractor {
    body: Body,
    gravity: Gravity,
}

impl Attractor {
    pub fn new(mass: f32, point: Point, g: f32, min_dist: f32) -> Attractor {
        Attractor {
            body: Body::new(mass, point, Vector::zero()),
            gravity: Gravity::new(g, min_dist),
        }
    }

    pub fn force(&self, body: &Body) -> Vector {
        self.gravity.between(body, &self.body)
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{Gravity, Attractor, Body};
    use geometry::types::{Point, Vector};

    #[test]
    #[should_panic(expected = "The minimum gravitational distance must be greater than 0.")]
    fn gravity_with_negative_minimum_distance() {
        // given
        Gravity::new(1.0, -2.0);
    }

    #[test]
    fn gravity_calculates_force() {
        // given
        let sut = Gravity::new(1.5, 4.0);

        let b1 = Body::new(
            1.0,
            Point { x: 1.0, y: 2.0},
            Vector::zero()
        );

        let b2 = Body::new(
            2.0,
            Point { x: -3.5, y: 0.0},
            Vector::zero()
        );

        // when
        let result = sut.between(&b1, &b2);

        // then
        assert_eq!(result, Vector { dx: -0.1130488514, dy: -0.0502439339});
    }

    #[test]
    fn gravity_obeys_minimum_distance() {
        // given
        let sut = Gravity::new(1.5, 4.0);

        let b1 = Body::new(
            1.0,
            Point { x: 1.0, y: 2.0},
            Vector::zero()
        );

        let b2 = Body::new(
            2.0,
            Point { x: 2.0, y: 2.5},
            Vector::zero()
        );

        assert!(b1.position.distance_to(&b2.position) < 4.0);

        // when
        let result = sut.between(&b1, &b2);

        // then
        let result_if_dist_was_4 = Vector { dx: 0.1677050983, dy: 0.0838525491};
        assert_eq!(result, result_if_dist_was_4);
    }

    #[test]
    fn gravity_is_undefined_for_bodies_with_equal_position() {
        // given
        let sut = Gravity::new(1.5, 4.0);

        let b1 = Body::new(
            1.0,
            Point { x: 1.0, y: 2.0},
            Vector::zero()
        );

        let b2 = Body::new(
            2.0,
            Point { x: 1.0, y: 2.0},
            Vector::zero()
        );

        // when
        let result = sut.between(&b1, &b2);

        // then
        assert_eq!(result, Vector::zero());
    }

    #[test]
    fn attractor_calculates_force() {
        // given
        let sut = Attractor::new(
            100.0,
            Point::origin(),
            2.3,
            1.0
        );

        let body = Body::new(
            3.8,
            Point { x: 1.0, y: 2.0},
            Vector::zero()
        );

        // when
        let result = sut.force(&body);

        // then
        assert_eq!(result, Vector { dx: -78.17293649, dy: -156.345873});
    }
}
