use std::ops::{ Add, AddAssign, Mul, Div };
use point::Point;

/**
 *  Vector in 2D space.
 */
#[derive(Debug)]
pub struct Vector {
    pub dx: f64,
    pub dy: f64,
}

/**
 *  Vector equality.
 */
impl PartialEq for Vector {
    fn eq(&self, other: &'_ Vector) -> bool {
        let e = 0.0000001;
        let x = (self.dx - other.dx).abs();
        let y = (self.dy - other.dy).abs();
        x < e && y < e
    }
}

/**
 *  Vector addition.
 */
impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

/**
 *  Vector add assign.
 */
impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}

/**
 *  Scalar multiplication.
 */
impl<'a> Mul<f64> for &'a Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
        }
    }
}

/**
 *  Scalar division.
 */
impl<'a> Div<f64> for &'a Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Self::Output {
        Vector {
            dx: self.dx / rhs,
            dy: self.dy / rhs,
        }
    }
}

// TODO: Needs testing
/**
 *  Inner product.
 */
impl<'a> Mul for &'a Vector {
    type Output = f64;

    fn mul(self, rhs: &Vector) -> Self::Output {
        self.dx * rhs.dx + self.dy * rhs.dy
    }
}

impl Vector {
    /**
     *  The zero vector.
     */
    pub fn zero() -> Vector {
        Vector { dx: 0.0, dy: 0.0 }
    }

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
     *  Normalized copy of self.
     */
    pub fn normalized(&self) -> Option<Vector> {
        if self == &Vector::zero() { return None }
        let magnitude = self.magnitude();

        Some(Vector {
            dx: self.dx / magnitude,
            dy: self.dy / magnitude
        })
    }
}
