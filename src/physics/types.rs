use geometry::types::{Point, Vector};
use std::cmp::Eq;
use std::collections::HashMap;

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a moveable object in space.

#[derive(Debug)]
pub struct Body {
    pub id: u32,
    pub mass: f64,
    pub position: Point,
    pub velocity: Vector,
}

impl Eq for Body {}

impl PartialEq for Body {
    fn eq(&self, other: &'_ Body) -> bool {
        // Bodies are compared referentially.
        self as *const _ == other as *const _
    }
}

impl Body {
    pub fn new(id: u32, mass: f64, position: Point, velocity: Vector) -> Body {
        if mass <= 0.0 {
            panic!("A body's mass must be greater than 0. Got {}", mass);
        }
        Body {
            id,
            mass,
            position,
            velocity,
        }
    }

    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass;
        self.position.x += self.velocity.dx.round() as i32;
        self.position.y += self.velocity.dy.round() as i32;
    }
}

// Field /////////////////////////////////////////////////////////////////////
//
//

pub struct Field {
    pub g: f64,
    pub solar_mass: f64, // TODO: make sure non zero and positive
    pub min_dist: f64,
    pub max_dist: f64,
    pub bodies: Vec<Body>,
}

impl Field {
    // TODO: Needs testing
    /**
     *  Update the state of the field by applying force on each of the bodies
     *  and updating their positions.
     */
    pub fn update(&mut self) {
        let force_map = self.force_map();

        // now the idea is to iterate through each body and update
        // its velocity, then its position.

        for body in self.bodies.iter_mut() {
            match force_map.get(&body.id) {
                Some(force) => body.apply_force(force),
                None => (),
            }
        }
    }

    // TODO: Needs testing
    /**
     *  A mapping of Body ids to their corresponding received forces
     *  for the current state. This uses a brute force approach with O(n^2).
     */
    fn force_map(&self) -> HashMap<u32, Vector> {
        // store the forces for each body
        let mut forces: HashMap<u32, Vector> = HashMap::new();

        for body in self.bodies.iter() {
            let mut cumulative_force = Vector::zero();

            // combine the forces of all other bodies exerted on body
            for other in self.bodies.iter() {
                if body != other {
                    cumulative_force += self.force_between(body, other);
                }
            }

            // add solar force
            cumulative_force += self.solar_force(body);

            forces.insert(body.id, cumulative_force);
        }

        forces
    }

    // TODO: Needs testing
    /**
     *  The force exerted mutually between the given bodies.
     */
    fn force_between(&self, b1: &Body, b2: &Body) -> Vector {
        // Bad things happen if both bodies occupy the same space.
        if b1.position == b2.position {
            return Vector::zero();
        }
        let difference = Vector::difference(&b2.position, &b1.position);
        let distance = difference.magnitude().min(self.max_dist).max(self.min_dist);
        let force = (self.g * b1.mass * b2.mass) / (distance * distance);

        let direction = match difference.normalized() {
            None => Vector::zero(),
            Some(normalized) => normalized,
        };

        &direction * force
    }

    // TODO: try to refactor this into the method above. THe issue was to do with creating a solar body.
    /**
     *  The force exerted by the sun on the given body.
     */
    fn solar_force(&self, body: &Body) -> Vector {
        if self.solar_mass == 0.0 || body.position.is_origin() {
            return Vector::zero();
        }
        let difference = Vector {
            dx: -body.position.x as f64,
            dy: -body.position.y as f64,
        };
        let distance = difference.magnitude().min(self.max_dist).max(self.min_dist);
        let force = (self.g * self.solar_mass * body.mass) / (distance * distance);

        let direction = match difference.normalized() {
            None => Vector::zero(),
            Some(normalized) => normalized,
        };

        &direction * force
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::types::{Point, Vector};

    #[test]
    #[should_panic(expected = "A body's mass must be greater than 0.")]
    fn body_with_zero_mass() {
        // given
        Body::new(0, 0.0, Point::origin(), Vector::zero());
    }

    #[test]
    #[should_panic(expected = "A body's mass must be greater than 0.")]
    fn body_with_negative_mass() {
        // given
        Body::new(0, -10.0, Point::origin(), Vector::zero());
    }

    #[test]
    fn body_has_referential_equivalence() {
        // given
        let b1 = Body {
            id: 0,
            mass: 1.0,
            position: Point { x: 1, y: 2 },
            velocity: Vector::zero(),
        };

        let b2 = Body {
            id: 0,
            mass: 1.0,
            position: Point { x: 1, y: 2 },
            velocity: Vector::zero(),
        };

        // then
        assert_eq!(b1, b1);
        assert_ne!(b1, b2);
    }

    #[test]
    fn body_applies_force() {
        // given
        let mut sut = Body {
            id: 0,
            mass: 2.0,
            position: Point { x: 1, y: 2 },
            velocity: Vector { dx: -2.0, dy: 5.0 },
        };

        let force = Vector { dx: 2.6, dy: -3.2 };

        // when
        sut.apply_force(&force);

        // then
        assert_eq!(sut.velocity, Vector { dx: -0.7, dy: 3.4 });
        assert_eq!(sut.position, Point { x: 0, y: 5 });
    }
}
