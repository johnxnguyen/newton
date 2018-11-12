use geometry::vector::Vector;

/**
 *  Coordinates in 2D space.
 */
#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/**
 *  Point equality.
 */
impl PartialEq for Point {
    fn eq(&self, other: &'_ Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Point {
    /**
     *  The zero point.
     */
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

impl Point {
    /**
     *  Returns true if self is the origin.
     */
    pub fn is_origin(&self) -> bool {
        self == &Point::origin()
    }

    /**
     *  Calculate the distance between self and the given point.
     */
    pub fn distance_to(&self, other: &Point) -> f64 {
        let difference = Vector::difference(self, other);
        difference.magnitude()
    }
}
