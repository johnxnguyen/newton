use geometry::types::{Point, Vector};
use std::ops::Mul;

// Transformation ////////////////////////////////////////////////////////////
//
// A 2D transformation matrix represented as a pair of transformed basis
// vectors.

pub struct Transformation(Vector, Vector);

impl<'a> Mul<Vector> for &'a Transformation {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Self::Output {
        &self.0 * rhs.dx + &self.1 * rhs.dy
    }
}

impl<'a> Mul<Point> for &'a Transformation {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        Point::from(&self.0 * rhs.x + &self.1 * rhs.y)
    }
}

impl Transformation {
    pub fn rotation(radians: f32) -> Transformation {
        let (sin, cos) = radians.sin_cos();
        Transformation(Vector { dx: cos, dy: sin }, Vector { dx: -sin, dy: cos })
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::types::Vector;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn it_transforms_a_vector() {
        // given
        let sut = Transformation(Vector { dx: 2.0, dy: 0.0 }, Vector { dx: 0.0, dy: 2.0 });

        // when
        let result = &sut * Vector { dx: 4.0, dy: -2.5 };

        // then
        assert_eq!(result, Vector { dx: 8.0, dy: -5.0 });
    }

    #[test]
    fn it_rotates_a_vector() {
        // given
        let sut = Transformation::rotation(FRAC_PI_2);

        // when
        let result = &sut * Vector { dx: 1.0, dy: 0.0 };

        // then
        assert_eq!(result, Vector { dx: 0.0, dy: 1.0 });
    }
}
