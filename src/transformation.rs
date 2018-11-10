use vector::Vector;
use std::ops::Mul;

/**
 *  A 2D transformation matrix represented as a pair of
 *  vectors (the transformed basis vectors).
 */
pub struct Transformation {
    pub a: Vector,
    pub b: Vector,
}

/**
 *  Transformation of a vector.
 */
impl<'a> Mul<Vector> for &'a Transformation {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        &self.a * rhs.dx + &self.b * rhs.dy
    }
}

impl Transformation {
    /**
     *  Rotation transformation.
     */
    pub fn rotation(radians: f64) -> Transformation {
        let (sin, cos) = radians.sin_cos();
        Transformation {
            a: Vector { dx: cos, dy: sin },
            b: Vector { dx: -sin, dy: cos },
        }
    }
}