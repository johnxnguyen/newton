use geometry::types::{Point, Vector};
use super::force::{Gravity, Attractor};
use std::cmp::Eq;

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a movable object in space.

#[derive(Debug)]
pub struct Body {
    pub id: u32,
    pub mass: f32,
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
    pub fn new(id: u32, mass: f32, position: Point, velocity: Vector) -> Body {
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
        self.position.x += self.velocity.dx;
        self.position.y += self.velocity.dy;
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

pub struct BruteForceField {
    force: Gravity,
    sun: Option<Attractor>,
}

impl BruteForceField {
    pub fn new() -> BruteForceField {

        BruteForceField {
            force: Gravity::new(1.0, 4.0),
            sun: Some(Attractor::new(10000.0, Point::origin(), 1.0, 4.0)),
        }
    }
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

// Environment ///////////////////////////////////////////////////////////////
//
// An environment represents a space in which bodies interact with fields.

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
            position: Point { x: 1.0, y: 2.0 },
            velocity: Vector::zero(),
        };

        let b2 = Body {
            id: 0,
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
            id: 0,
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
}
