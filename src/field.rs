use body::Body;
use vector::Vector;
use std::collections::HashMap;

pub struct Field {
    pub g: f64,
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
                None => ()
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

            forces.insert(body.id, cumulative_force);
        }

        forces
    }

    // TODO: Needs testing
    /**
     *  The force exerted mutually between the given bodies.
     */
    fn force_between(&self, b1: &Body, b2: &Body) -> Vector {
        let difference = Vector::difference(&b1.position, &b2.position);
        let distance = difference.magnitude();
        let force = (self.g * b1.mass * b2.mass) / (distance * distance);
        let direction = difference.normalized();
        &direction * force
    }
}
