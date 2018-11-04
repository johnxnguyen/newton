use vector::Vector;

/**
 *  Coordinates in 2D space.
 */
#[derive(Debug)]
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

    // TODO: needs testing
    /**
     *  Returns true if self is the origin.
     */
    pub fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

// TODO: needs testing
impl PartialEq for Point {
    fn eq(&self, other: &'_ Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
