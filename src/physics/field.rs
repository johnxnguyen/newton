use crate::geometry::types::{Square, Vector};

use super::barneshut::BHTree;
use super::force::Gravity;
use super::types::Body;

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
}

impl Field for BruteForceField {
    fn forces(&self, bodies: &[Body]) -> Vec<Vector> {
        let mut result: Vec<Vector> = vec![];

        for body in bodies {
            let mut cumulative_force = Vector::zero();

            for other in bodies {
                cumulative_force += self.force.between(body, other);
            }

            result.push(cumulative_force);
        }

        result
    }
}

impl Default for BruteForceField {
    fn default() -> Self {
        BruteForceField {
            force: Gravity::new(1.0, 4.0),
        }
    }
}

impl BruteForceField {
    pub fn new() -> BruteForceField {
        Self::default()
    }
}

// BHField ///////////////////////////////////////////////////////////////////

// TODO: I want to be able to mark a body as unmoveable.

pub struct BHField {
    space: Square,
    force: Gravity,
}

impl Field for BHField {
    fn forces(&self, bodies: &[Body]) -> Vec<Vector> {
        let mut result: Vec<Vector> = vec![];
        let mut tree = BHTree::new(self.space.clone());

        for body in bodies {
            tree.add(body.clone());
        }

        for body in bodies {
            let f = tree.virtual_bodies(body).iter().fold(Vector::zero(), |acc, n| {
                acc + self.force.between(body, &n)
            });

            result.push(f);
        };

        result
    }
}

impl BHField {
    pub fn new() -> BHField {
        BHField {
            space: Square::new(-2048.0, -2048.0, 12),
            force: Gravity::new(1.0, 4.0),
        }
    }
}