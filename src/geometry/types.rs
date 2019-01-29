use std::cmp::PartialEq;
use std::fmt;
use std::ops::{Add, AddAssign, SubAssign, Div, Mul};

use geometry::types::ErrorKind::OutOfBounds;

use self::Quadrant::*;


// Point /////////////////////////////////////////////////////////////////////
//
// Coordinates in 2D space.

#[derive(Clone, PartialEq, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

// TODO: Test
impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// TODO: Test
impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<'a> Mul<f32> for &'a Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl<'a> Div<f32> for &'a Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Point::new(
            self.x / rhs,
            self.y / rhs
        )
    }
}

impl From<Vector> for Point {
    fn from(v: Vector) -> Self {
        Point::new(v.dx, v.dy)
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    pub fn zero() -> Point {
        Point::new(0.0, 0.0)
    }

    pub fn is_zero(&self) -> bool {
        self == &Point::zero()
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let difference = Vector::difference(self, other);
        difference.magnitude()
    }
}

// Vector ////////////////////////////////////////////////////////////////////
//
// Change of coordinates in 2D space.

#[derive(Clone, Debug)]
pub struct Vector {
    pub dx: f32,
    pub dy: f32,
}

impl PartialEq for Vector {
    fn eq(&self, other: &'_ Vector) -> bool {
        let e = 0.0000001;
        let x = (self.dx - other.dx).abs();
        let y = (self.dy - other.dy).abs();
        x < e && y < e
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
        }
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}

impl<'a> Mul<f32> for &'a Vector {
    type Output = Vector;

    fn mul(self, scalar: f32) -> Self::Output {
        Vector {
            dx: self.dx * scalar,
            dy: self.dy * scalar,
        }
    }
}

impl<'a> Div<f32> for &'a Vector {
    type Output = Vector;

    fn div(self, scalar: f32) -> Self::Output {
        Vector {
            dx: self.dx / scalar,
            dy: self.dy / scalar,
        }
    }
}

impl<'a> Mul for &'a Vector {
    type Output = f32;

    fn mul(self, rhs: &Vector) -> Self::Output {
        self.dx * rhs.dx + self.dy * rhs.dy
    }
}

impl Vector {
    pub fn new(dx: f32, dy: f32) -> Vector {
        Vector { dx, dy }
    }

    pub fn zero() -> Vector {
        Vector { dx: 0.0, dy: 0.0 }
    }

    pub fn difference(lhs: &Point, rhs: &Point) -> Vector {
        Vector {
            dx: (lhs.x - rhs.x),
            dy: (lhs.y - rhs.y),
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }
    
    pub fn normalized(&self) -> Option<Vector> {
        if self == &Vector::zero() { return None; }
        let magnitude = self.magnitude();

        Some(Vector {
            dx: self.dx / magnitude,
            dy: self.dy / magnitude,
        })
    }
}

// Quadrant //////////////////////////////////////////////////////////////////
//
// The four quadrants of a rectangle.

#[derive(Clone, PartialEq, Debug)]
pub enum Quadrant { NW(Square), NE(Square), SW(Square), SE(Square) }

impl Quadrant {
    pub fn space(&self) -> &Square {
        match self {
            NW(space) => space,
            NE(space) => space,
            SW(space) => space,
            SE(space) => space,
        }
    }

    fn contains(&self, point: &Point) -> bool {
        self.space().contains(point)
    }
}

// Square //////////////////////////////////////////////////////////////////////
//
// A rectangle whose origin denotes the position of the bottom left corner.

#[derive(Clone, PartialEq, Debug)]
pub struct Square {
    pub origin: Point,
    pub size: u32,
}

impl Square {
    /// Creates a new Square with size = 2^exponent.
    pub fn new(x: f32, y: f32, exponent: u32) -> Square {
        Square {
            origin: Point::new(x,y),
            size: u32::pow(2, exponent),
        }
    }

    // TODO: test
    pub fn is_unit_rect(&self) -> bool {
        self.size == 1
    }

    /// Returns the length of the hypotenuse.
    pub fn diameter(&self) -> f32 {
        let x = (self.size as f32).powi(2);
        (2.0 * x).sqrt()
    }

    /// Returns true if the given point is contained by self.
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.origin.x && point.y >= self.origin.y &&
            point.x <= self.upper_bound().x && point.y <= self.upper_bound().y
    }

    /// Returns a partition of self in the order northeast, northwest,
    /// southeast, southwest. If the width and height of self are even,
    /// then the four quadrants are of equal size. Otherwise the boundaries
    /// of the quadrants is shifted so their widths and heights remain
    /// integers. This eliminates gaps in the coverage of the quadrants.
    pub fn quadrants(&self) -> (Quadrant, Quadrant, Quadrant, Quadrant) {
        assert!(!self.is_unit_rect(), "Cannot split rect with minimal dimension.");

        let (x, y) = (self.origin.x, self.origin.y);
        let size = self.size >> 1;

        let sw = Square { origin: Point::new(x, y), size };
        let se = Square { origin: Point::new(x + size as f32, y), size };
        let nw = Square { origin: Point::new(x, y + size as f32), size };
        let ne = Square { origin: Point::new(x + size as f32, y + size as f32), size };
        (NW(nw), NE(ne), SW(sw), SE(se))
    }

    /// Returns the quadrant for the given point. If the point is not
    /// contained by any quadrant, an OutOfBounds error is returned.
    pub fn quadrant(&self, point: &Point) -> Result<Quadrant, Error> {
        let q = self.quadrants();
        if      q.0.contains(point) { Ok(q.0) }
        else if q.1.contains(point) { Ok(q.1) }
        else if q.2.contains(point) { Ok(q.2) }
        else if q.3.contains(point) { Ok(q.3) }
        else                        { Err(Error(OutOfBounds)) }
    }

    fn upper_bound(&self) -> Point {
        Point {
            x: self.origin.x + self.size as f32,
            y: self.origin.y + self.size as f32,
        }
    }
}

// Error /////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug)]
pub struct Error(ErrorKind);
impl Error {
    pub fn kind(&self) -> ErrorKind { self.0 }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ErrorKind {
    OutOfBounds
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutOfBounds => write!(f, "Cannot compare a point with a rect that does not contain it.")
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance_from_origin() {
        // given
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 5.0, y: 0.0 };

        // when
        let result = p1.distance_to(&p2);

        // then
        assert_eq!(result, 5.0);
    }

    #[test]
    fn point_distance_to_origin() {
        // given
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 0.0, y: -5.0 };

        // when
        let result = p2.distance_to(&p1);

        // then
        assert_eq!(result, 5.0);
    }

    #[test]
    fn point_addition() {
        // given
        let sut = Point::new(-4.6, 7.5);

        // when
        let result = sut + Point::new(-8.8, -6.5);

        // then
        assert_eq!(Point::new(-13.4, 1.0), result);
    }

    #[test]
    fn point_scalar_multiplication() {
        // given
        let sut = Point::new(5.5, 2.0);

        // when
        let result = &sut * -3.5;

        // then
        assert_eq!(Point::new(-19.25, -7.0), result);
    }

    #[test]
    fn point_scalar_division() {
        // given
        let sut = Point::new(-6.2, 14.8);

        // when
        let result = &sut / 2.0;

        // then
        assert_eq!(Point::new(-3.1, 7.4), result);
    }

    // Vector ////////////////////////////////////////////////////////////////

    #[test]
    fn vector_add_assigns() {
        // given
        let mut sut = Vector { dx: 3.0, dy: 4.0 };

        // when
        sut += Vector { dx: 9.5, dy: -3.5 };

        // then
        assert_eq!(Vector { dx: 12.5, dy: 0.5 }, sut);
    }

    #[test]
    fn vector_scalar_multiplies() {
        // given
        let sut = Vector { dx: 3.0, dy: 4.0 };

        // when
        let result = &sut * 3.0;

        // then
        assert_eq!(result, Vector { dx: 9.0, dy: 12.0 })
    }

    #[test]
    fn vector_scalar_divides() {
        // given
        let sut = Vector { dx: 3.0, dy: 12.0 };

        // when
        let result = &sut / 3.0;

        // then
        assert_eq!(result, Vector { dx: 1.0, dy: 4.0 });
    }

    #[test]
    fn vector_inner_product() {
        // given
        let a = Vector { dx: 3.4, dy: -4.9 };
        let b = Vector { dx: 10.0, dy: 6.3 };

        // when
        let result = &a * &b;

        // then
        assert!((result - 3.13).abs() < 0.00001);
    }

    #[test]
    fn vector_magnitude() {
        // given, when, then
        assert_eq!(5.0, Vector { dx: 3.0, dy: 4.0 }.magnitude())
    }

    #[test]
    fn vector_normalize() {
        // given
        let sut = Vector { dx: 3.3, dy: 5.2 };

        // when
        let result = sut.normalized().unwrap();

        // then
        assert!((result.magnitude() - 1.0).abs() < 0.0000001);
    }

    #[test]
    fn vector_does_not_normalize_if_zero() {
        // given, when, then
        assert_eq!(None, Vector::zero().normalized())
    }

    // Square //////////////////////////////////////////////////////////////////

    #[test]
    fn rect_diameter() {
        // given
        let sut = Square::new(0.0, 0.0, 2);

        // when
        let result = sut.diameter();

        // then
        assert_eq!(5.65685424949, result);
    }

    #[test]
    fn rect_quadrants() {
        // given
        let sut = Square::new(4.0, 2.0, 2);

        // when
        let (nw, ne, sw, se) = sut.quadrants();

        // then
        assert_eq!(NW(Square::new(4.0, 4.0, 1)), nw);
        assert_eq!(NE(Square::new(6.0, 4.0, 1)), ne);
        assert_eq!(SW(Square::new(4.0, 2.0, 1)), sw);
        assert_eq!(SE(Square::new(6.0, 2.0, 1)), se);
    }

    #[test]
    #[should_panic(expected = "Cannot split rect with minimal dimension.")]
    fn rect_quadrants_of_unit_rect() {
        // given
        let sut = Square::new(0.0, 0.0, 0);

        // when, then
        sut.quadrants();
    }

    #[test]
    fn rect_contains_point() {
        // given
        let sut = Square::new(0.0, 0.0, 5);

        // then
        assert!(sut.contains(&Point::new(0.0, 0.0)));
        assert!(sut.contains(&Point::new(3.0, 3.0)));
        assert!(sut.contains(&Point::new(10.0, 5.0)));

        assert!(!sut.contains(&Point::new(-0.0001, 0.0)));
        assert!(!sut.contains(&Point::new(33.0000, 5.00001)));
        assert!(!sut.contains(&Point::new(1.0, 40.01)));
    }

    #[test]
    fn rect_which_quadrant() {
        // given
        let sut = Square::new(0.0, 0.0, 3);
        let (nw, ne, sw, se) = sut.quadrants();

        // then (bottom left of each quadrant)
        assert_eq!(Ok(nw.clone()), sut.quadrant(&Point::new(0.0, 4.0)));
        assert_eq!(Ok(ne.clone()), sut.quadrant(&Point::new(5.0, 5.0)));
        assert_eq!(Ok(sw.clone()), sut.quadrant(&Point::new(1.0, 0.0)));
        assert_eq!(Ok(se.clone()), sut.quadrant(&Point::new(5.0, 0.0)));

        // then (top right of each quadrant)
        assert_eq!(Ok(nw.clone()), sut.quadrant(&Point::new(2.0, 5.0)));
        assert_eq!(Ok(ne.clone()), sut.quadrant(&Point::new(5.0, 5.0)));
        assert_eq!(Ok(sw.clone()), sut.quadrant(&Point::new(2.0, 1.0)));
        assert_eq!(Ok(se.clone()), sut.quadrant(&Point::new(5.0, 1.0)));

        // then (anywhere in quadrant)
        assert_eq!(Ok(nw.clone()), sut.quadrant(&Point::new(3.0, 4.1)));
        assert_eq!(Ok(ne.clone()), sut.quadrant(&Point::new(4.3, 5.1)));
        assert_eq!(Ok(nw.clone()), sut.quadrant(&Point::new(1.0, 6.0)));
        assert_eq!(Ok(ne.clone()), sut.quadrant(&Point::new(6.3, 6.8)));

        // then
        assert_eq!(Err(Error(OutOfBounds)), sut.quadrant(&Point::new(-2.5, 5.0)));
        assert_eq!(Err(Error(OutOfBounds)), sut.quadrant(&Point::new(8.4, 0.4)));
        assert_eq!(Err(Error(OutOfBounds)), sut.quadrant(&Point::new(2.5, -4.0)));
        assert_eq!(Err(Error(OutOfBounds)), sut.quadrant(&Point::new(4.0, 9.5)),);
    }
}
