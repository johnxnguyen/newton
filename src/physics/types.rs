use std::cmp::Eq;
use std::fmt;

use geometry::types::{Point, Vector};

use super::force::{Attractor, Gravity};

// Environment ///////////////////////////////////////////////////////////////
//
// An environment represents a space in which bodies interact with fields.

// TODO: when passing the bodies to the field, we need to return them back in the same order.
pub struct Environment {
    pub bodies: Vec<Body>,
    pub fields: Vec<Box<Field>>,
}

impl Environment {
    pub fn new() -> Environment {
        let field = BruteForceField::new();
        Environment {
            bodies: vec![],
            fields: vec![Box::from(field)],
        }
    }

    pub fn update(&mut self) {
        for field in self.fields.iter() {
            let forces = field.forces(&self.bodies[..]);

            for (body, force) in self.bodies.iter_mut().zip(forces.iter()) {
                body.apply_force(force);
            }
        }
    }
}

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a movable object in space.

#[derive(Clone, Debug)]
pub struct Body {
    pub mass: f32, // TODO: make this a type with validation (for positive values)
    pub position: Point,
    pub velocity: Vector,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<>) -> Result<(), fmt::Error> {
        write!(f, "M({}) P({}, {}) V({}, {})",
               self.mass,
               self.position.x, self.position.y,
               self.velocity.dx, self.velocity.dy)
    }
}

impl Eq for Body {}

impl PartialEq for Body {
    fn eq(&self, other: &'_ Body) -> bool {
        // Bodies are compared referentially.
        self as *const _ == other as *const _
    }
}

impl Body {
    pub fn new(mass: f32, position: Point, velocity: Vector) -> Body {
        if mass <= 0.0 {
            panic!("A body's mass must be greater than 0. Got {}", mass);
        }
        Body {
            mass,
            position,
            velocity,
        }
    }

    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass;
        self.position.x += self.velocity.dx;
        self.position.y += self.velocity.dy;
    }
    
    pub fn weighted_position(&self) -> Point {
        Point::new(self.mass * self.position.x, self.mass * self.position.y)
    }
}

// Field /////////////////////////////////////////////////////////////////////
//
// A field represents an instance of space in which bodies are affected by
// gravitational force.

pub trait Field {
    fn forces(&self, bodies: &[Body]) -> Vec<Vector>;
}

// BruteForceField ///////////////////////////////////////////////////////////
//
// Brute force gravitation calculation between n bodies. For every body,
// calculate the gravitational force with every other body directly.

pub struct BruteForceField {
    force: Gravity,
    sun: Option<Attractor>,
}

impl Field for BruteForceField {
    fn forces(&self, bodies: &[Body]) -> Vec<Vector> {
        let mut result: Vec<Vector> = vec![];

        for body in bodies {
            let mut cumulative_force = Vector::zero();

            for other in bodies {
                cumulative_force += self.force.between(body, other);
            }

            if let Some(ref sun) = self.sun {
                cumulative_force += sun.force(body);
            }

            result.push(cumulative_force);
        }

        result
    }
}

impl BruteForceField {
    pub fn new() -> BruteForceField {
        BruteForceField {
            force: Gravity::new(1.0, 4.0),
            sun: Some(Attractor::new(10000.0, Point::zero(), 1.0, 4.0)),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use geometry::types::{Point, Vector};

    use super::*;

    #[test]
    #[should_panic(expected = "A body's mass must be greater than 0.")]
    fn body_with_zero_mass() {
        // given
        Body::new(0.0, Point::zero(), Vector::zero());
    }

    #[test]
    #[should_panic(expected = "A body's mass must be greater than 0.")]
    fn body_with_negative_mass() {
        // given
        Body::new(-10.0, Point::zero(), Vector::zero());
    }

    #[test]
    fn body_has_referential_equivalence() {
        // given
        let b1 = Body {
            mass: 1.0,
            position: Point { x: 1.0, y: 2.0 },
            velocity: Vector::zero(),
        };

        let b2 = Body {
            mass: 1.0,
            position: Point { x: 1.0, y: 2.0 },
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
            mass: 2.0,
            position: Point { x: 1.0, y: 2.0 },
            velocity: Vector { dx: -2.0, dy: 5.0 },
        };

        let force = Vector { dx: 3.0, dy: -3.0 };

        // when
        sut.apply_force(&force);

        // then
        assert_eq!(sut.velocity, Vector { dx: -0.5, dy: 3.5 });
        assert_eq!(sut.position, Point { x: 0.5, y: 5.5 });
    }
    
    #[test]
    fn body_weighted_position() {
        // given, then
        let sut = Body::new(3.7, Point::new(4.6, 7.5), Vector::zero());
        assert_eq!(Point::new(17.02, 27.75), sut.weighted_position());

        // given, then
        let sut = Body::new(2.1, Point::new(-24.6, -9.0), Vector::zero());
        assert_eq!(Point::new(-51.66, -18.9), sut.weighted_position());

        // given, then
        let sut = Body::new(14.5, Point::zero(), Vector::zero());
        assert_eq!(Point::zero(), sut.weighted_position());
    }
}
