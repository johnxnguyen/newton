use body::Body;
use vector::Vector;

pub struct Field {
    pub g: f64,
    pub bodies: Vec<Body>,
}

impl Field {

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
