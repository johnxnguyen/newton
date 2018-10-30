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
}
