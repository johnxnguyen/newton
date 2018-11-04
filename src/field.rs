use body::Body;
use vector::Vector;
use std::collections::HashMap;

pub struct Field {
    pub g: f64,
    pub solar_mass: f64,    // TODO: make sure non zero and positive
    pub min_dist: f64,
    pub max_dist: f64,
    pub bodies: Vec<Body>,
}

impl Drop for Field {
    fn drop(&mut self) {
        println!("A field has been deallocated.");
    }
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
        if b1.position == b2.position { return Vector::zero() }
        let difference = Vector::difference(&b2.position, &b1.position);
        let distance = difference.magnitude().min(self.max_dist).max(self.min_dist);
        let force = (self.g * b1.mass * b2.mass) / (distance * distance);
        let direction = difference.normalized();
        &direction * force
    }

    // TODO: try to refactor this into the method above. THe issue was to do with creating a solar body.
    /**
     *  The force exerted by the sun on the given body.
     */
    fn solar_force(&self, body: &Body) -> Vector {
        if self.solar_mass == 0.0 || body.position.is_origin() { return Vector::zero() }
        let difference = Vector { dx: -body.position.x as f64, dy: -body.position.y as f64 };
        let distance = difference.magnitude().min(self.max_dist).max(self.min_dist);
        let force = (self.g * self.solar_mass * body.mass) / (distance * distance);
        let direction = difference.normalized();
        &direction * force
    }
}
