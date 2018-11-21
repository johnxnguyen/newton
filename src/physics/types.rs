use geometry::types::{Point, Vector};
use std::cmp::Eq;
use std::collections::HashMap;

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a moveable object in space.

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

pub struct Field {
    pub g: f32,
    pub min_dist: f32,
    pub max_dist: f32,
    pub bodies: HashMap<u32, Body>,
    sun: Option<Body>,
}

impl Field {
    pub fn new(g: f32, solar_mass: f32, min_dist: f32, max_dist: f32) -> Field {
        let sun = match solar_mass {
            // TODO: tidy this up
            x if x > 0.0 => Some(Body::new(0, solar_mass, Point::origin(), Vector::zero())),
            _ => None,
        };

        Field {
            g,
            sun,
            min_dist: min_dist.max(0.0),
            max_dist: max_dist.max(0.0),
            bodies: HashMap::new(),
        }
    }

    // TODO: Needs testing
    /**
     *  Update the state of the field by applying force on each of the bodies
     *  and updating their positions.
     */
    pub fn update(&mut self) {
        let force_map = self.force_map();

        // update each body
        for (id, body) in self.bodies.iter_mut() {
            match force_map.get(&id) {
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

        // for each body
        for (id, body) in self.bodies.iter() {
            let mut cumulative_force = Vector::zero();

            // combine the forces of all other bodies exerted on body
            for other in self.bodies.values() {
                cumulative_force += self.force_between(body, other);
            }

            // also for the sun, if it exists
            if let Some(ref sun) = self.sun {
                cumulative_force += self.force_between(body, &sun);
            }

            forces.insert(*id, cumulative_force);
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
        let distance = difference.magnitude().max(self.min_dist);
        let force = (self.g * b1.mass * b2.mass) / (distance * distance);

        let direction = match difference.normalized() {
            None => Vector::zero(),
            Some(normalized) => normalized,
        };

        &direction * force
    }
}



// Invironment ///////////////////////////////////////////////////////////////


pub struct Environment {
    pub bodies: HashMap<u32, Body>,
    pub fields: Vec<Field>,
}


impl Environment {
    pub fn update(&mut self) {

        // update each field
        for field in self.fields.iter_mut() {

            field.update()

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
