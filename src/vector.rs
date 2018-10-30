use point::Point;
use std::ops::{ Mul, Div };

/**
 *  Vector in 2D space.
 */
pub struct Vector {
    pub dx: f64,
    pub dy: f64,
}

impl Mul<f64> for Vector {
    type Output = Vector;

    /**
     *  Scalar multiplication.
     */
    fn mul(self, rhs: f64) -> Self::Output {
        Vector {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    /**
     *  Scalar division.
     */
    fn div(self, rhs: f64) -> Self::Output {
        Vector {
            dx: self.dx / rhs,
            dy: self.dy / rhs,
        }
    }
}

impl Vector {
    /**
     *  The difference vector between two points.
     */
    pub fn difference(lhs: &Point, rhs: &Point) -> Vector {
        Vector {
            dx: (lhs.x - rhs.x) as f64,
            dy: (lhs.y - rhs.y) as f64,
        }
    }
}

impl Vector {
    /**
     *  The magnitude of a vector is the length of its hypotenuse.
     */
    pub fn magnitude(&self) -> f64 {
        ((self.dx * self.dx + self.dy * self.dy) as f64).sqrt()
    }

    /**
     *  Normalizing a vector into the unit vector.
     */
    pub fn normalize(&mut self) {
        let magnitude = self.magnitude();
        self.dx = self.dx / magnitude;
        self.dy = self.dy / magnitude;
    }
}
