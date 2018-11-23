use super::types::Body;
use geometry::types::Vector;

// Gravity ///////////////////////////////////////////////////////////////////
//
// Newton's Law of Universal Gravitation.

// TODO: Needs testing
pub struct Gravity {
    g: f32,
    min_dist: f32,
}

impl Gravity {
    pub fn new(g: f32, min_dist: f32) -> Gravity {
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
