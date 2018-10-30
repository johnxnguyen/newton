
/**
 *  Coordinates in 2D space.
 */
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {

    pub fn distance_to(&self, other: &Point) -> f64 {
        let difference = Vector::difference(self, other);
        difference.magnitude()
    }
}

/**
 *  Vector in 2D space.
 */
pub struct Vector {
    pub dx: f64,
    pub dy: f64,
}

impl Vector {
    /**
     *  The difference vector between two points.
     */
    fn difference(lhs: &Point, rhs: &Point) -> Vector {
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

