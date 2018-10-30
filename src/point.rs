use vector::Vector;

/**
 *  Coordinates in 2D space.
 */
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /**
     *  Calculate the distance between self and the given point.
     */
    pub fn distance_to(&self, other: &Point) -> f64 {
        let difference = Vector::difference(self, other);
        difference.magnitude()
    }
}
