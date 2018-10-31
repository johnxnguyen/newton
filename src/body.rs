use point::Point;
use vector::Vector;
use std::cmp::Eq;

// TODO: Needs testing

#[derive(Debug)]
pub struct Body {
    pub id: u32,
    pub mass: f64,
    pub position: Point,
    pub velocity: Vector,
}

impl Eq for Body {}

impl PartialEq for Body {
    /**
     *  Bodies are compared referentially.
     */
    fn eq(&self, other: &'_ Body) -> bool {
        self as *const _ == other as *const _
    }
}

impl Body {
    /**
     *  Apply the given force to update the velocity vector.
     */
    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass;
        self.position.x += force.dx.round() as i32;
        self.position.y += force.dy.round() as i32;
    }
}
