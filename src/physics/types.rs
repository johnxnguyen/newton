use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt;

use uuid::Uuid;

use geometry::types::{Point, Vector};
use util::DataWriter;

use super::force::{Attractor, Gravity};

// Mass //////////////////////////////////////////////////////////////////////
//
// Simple wrapper type that can only hold a positive floating point value.

#[derive(Copy, Clone)]
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
    writer: DataWriter,
    pub bodies: Option<HashMap<Uuid, Body>>,
    pub fields: Vec<Box<Field>>,
}

impl Environment {
    pub fn new() -> Environment {
        let field = BruteForceField::new();
        Environment {
            writer: DataWriter::new("data"),
            bodies: Some(HashMap::new()),
            fields: vec![Box::from(field)],
        }
    }

    pub fn step(&mut self) {
        self.update();

        let points = self.bodies
            .as_ref()
            .unwrap()
            .values()
            .map(|b| b.position.clone())
            .collect();

        self.writer.write(points);
    }

    pub fn update(&mut self) {
        let mut bodies = self.bodies.take().unwrap();

        for field in self.fields.iter() {
            bodies = field.apply(bodies);
        }

        for body in bodies.values_mut() {
            body.apply_velocity();
        }

        self.bodies = Some(bodies);
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

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass.value();
    }

    pub fn apply_velocity(&mut self) {
        self.position.x += self.velocity.dx;
        self.position.y += self.velocity.dy;
    }
    
//    pub fn weighted_position(&self) -> Point {
//        Point::new(self.mass.value() * self.position.x, self.mass.value() * self.position.y)
//    }
}

// Field /////////////////////////////////////////////////////////////////////
//
// A field represents an instance of space in which bodies are affected by
// gravitational force.

pub trait Field {
    /// Takes ownership of the given bodies, applies force vectors to each
    /// body, returning ownership upon completion.
    fn apply(&self, bodies: HashMap<Uuid, Body>) -> HashMap<Uuid, Body>;
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
    fn apply(&self, mut bodies: HashMap<Uuid, Body>) -> HashMap<Uuid, Body> {
        let mut result: HashMap<Uuid, Vector> = HashMap::new();
        {
            // generate force map
            for body in bodies.values() {
                let mut cumulative_force = Vector::zero();

                for other in bodies.values() {
                    cumulative_force += self.force.between(body, other);
                }

                if let Some(ref sun) = self.sun {
                    cumulative_force += sun.force(body);
                }

                result.insert(body.id, cumulative_force);
            }
        }
        {
            // apply forces
            for (id, force) in result {
                let body = bodies.get_mut(&id).expect("Body not found.");
                body.apply_force(&force);
            }
        }
        bodies
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
