use std::cmp::Eq;
use std::fmt;

use uuid::Uuid;

use geometry::types::{Point, Vector};
use geometry::types::Square;
use physics::barneshut::BHTree;
use util::write::DataWriter;

use super::force::{Attractor, Gravity};

// Mass //////////////////////////////////////////////////////////////////////
//
// Simple wrapper type that can only hold a positive floating point value.

#[derive(PartialEq, Copy, Clone)]
pub struct Mass(f32);

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}

impl From<f32> for Mass {
    fn from(m: f32) -> Self {
        Mass::new(m)
    }
}

impl Mass {
    pub fn new(m: f32) -> Mass {
        if m <= 0.0 { panic!("A mass must be greater than 0. Got {}", m); }
        Mass(m)
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

// Environment ///////////////////////////////////////////////////////////////
//
// An environment represents a space in which bodies interact with fields.

pub struct Environment {
    pub bodies: Vec<Body>,
    pub fields: Vec<Box<Field>>,
    writer: DataWriter,
}

impl Environment {
    pub fn new() -> Environment {
        let field = BHField::new();
        Environment {
            bodies: vec![],
            fields: vec![Box::from(field)],
            writer: DataWriter::new("data"),
        }
    }

    pub fn update(&mut self) {
        for field in self.fields.iter() {
            let forces = field.forces(&self.bodies[..]);

            for (body, force) in self.bodies.iter_mut().zip(forces.iter()) {
                body.apply_force(force);
            }
        }

        for body in self.bodies.iter_mut() {
            body.apply_velocity();
        }

        let points = self.bodies.iter().map(|b| b.position.clone()).collect();
        self.writer.write(points);
    }
}

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a movable object in space.

#[derive(Debug)]
pub struct Body {
    id: Uuid,
    pub mass: Mass,
    pub position: Point,
    pub velocity: Vector,
}

impl Clone for Body {
    fn clone(&self) -> Self {
        Body:: new(self.mass.value(), self.position.clone(), self.velocity.clone())
    }
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
        self.id == other.id
    }
}

impl Body {
    pub fn new(mass: f32, position: Point, velocity: Vector) -> Body {
        Body {
            id: Uuid::new_v4(),
            mass: Mass::from(mass),
            position,
            velocity,
        }
    }

    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass.value();
    }

    pub fn apply_velocity(&mut self) {
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

// BHField ///////////////////////////////////////////////////////////////////

struct BHField {
    space: Square,
    force: Gravity,
    sun: Attractor,
}

impl Field for BHField {
    fn forces(&self, bodies: &[Body]) -> Vec<Vector> {
        let mut result: Vec<Vector> = vec![];
        let mut tree = BHTree::new(self.space.clone());

        for body in bodies {
            tree.add(body.clone());
        }

        for body in bodies {
            let mut f = tree.virtual_bodies(body).iter().fold(Vector::zero(), |acc, n| {
                acc + self.force.between(body, &n.to_body())
            });

            f += self.sun.force(body);
            result.push(f);
        };

        result
    }
}

impl BHField {
    pub fn new() -> BHField {
        BHField {
            space: Square::new(-1920.0, -1080.0, 20),
            force: Gravity::new(1.0, 4.0),
            sun: Attractor::new(10000.0, Point::zero(), 2.5, 4.0),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use geometry::types::{Point, Vector};

    use super::*;

    #[test]
    #[should_panic(expected = "A mass must be greater than 0.")]
    fn body_with_zero_mass() {
        // given
        Body::new(0.0, Point::zero(), Vector::zero());
    }

    #[test]
    #[should_panic(expected = "A mass must be greater than 0.")]
    fn body_with_negative_mass() {
        // given
        Body::new(-10.0, Point::zero(), Vector::zero());
    }

    #[test]
    fn body_has_referential_equivalence() {
        // given
        let b1 = Body::new(1.0, Point::new(1.0, 2.0), Vector::zero());
        let b2 = b1.clone();

        // then
        assert_eq!(b1, b1);
        assert_ne!(b1, b2);
    }

    #[test]
    fn body_applies_force() {
        // given
        let mut sut = Body::new(2.0, Point::new(1.0, 2.0), Vector::new(-2.0, 5.0));
        let force = Vector { dx: 3.0, dy: -3.0 };

        // when
        sut.apply_force(&force);

        // then
        assert_eq!(Vector::new(-0.5, 3.5), sut.velocity);
        assert_eq!(Point::new(1.0, 2.0), sut.position);
    }

    #[test]
    fn body_applies_velocity() {
        // given
        let mut sut = Body::new(2.0, Point::new(1.0, 2.0), Vector::new(-2.0, 5.0));

        // when
        sut.apply_velocity();

        // then
        assert_eq!(Point::new(-1.0, 7.0), sut.position);
    }
}
