use point::Point;
use vector::Vector;

pub struct Body {
    pub mass: f64,
    pub position: Point,
    pub velocity: Vector,
}

impl Body {
    /**
     *  Apply the given force to update the velocity vector.
     */
    fn apply_force(&mut self, force: Vector) {
        self.velocity += force / self.mass;
    }

    /**
     *  Cacluate the force exterted on self by the given body.
     */
    fn force_from(&self, other: &Body) -> Vector {
        let difference = Vector::difference(&self.position, &other.position);
        let distance = difference.magnitude();
        let force = (self.mass * other.mass) / (distance * distance);
        let direction = difference.normalized();
        direction * force
    }
}