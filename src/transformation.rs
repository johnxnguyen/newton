use vector::Vector;
use std::ops::Mul;

struct Transformation {
    a: Vector,
    b: Vector,
}

// TODO: Needs testing
impl<'a> Mul<Vector> for &'a Transformation {
    type Output = Vector;

    /**
     *  Transformation of a vector.
     */
    fn mul(self, rhs: Vector) -> Self::Output {
        &self.a * rhs.dx + &self.b * rhs.dy
    }
}

impl Transformation {

    // TODO: Needs testing
    fn rotation(theta: f64) -> Transformation {
        let (sin, cos) = theta.sin_cos();
        Transformation {
            a: Vector { dx: cos, dy: sin },
            b: Vector { dx: -sin, dy: cos },
        }
    }
}