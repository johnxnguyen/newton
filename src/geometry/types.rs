use std::fmt;
use std::cmp::PartialEq;
use std::ops::{Add, AddAssign, Div, Mul};

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

// Size //////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Size {
        if width == 0 || height == 0 { panic!("A size's width and/or height must be positive."); }
        Size { width, height }
    }
}

// Quadrant //////////////////////////////////////////////////////////////////
//
// The four quadrants of a rectangle.
//TODO: make this a struct
#[derive(Clone, PartialEq, Debug)]
pub enum Quadrant { NW(Rect), NE(Rect), SW(Rect), SE(Rect) }

impl Quadrant {
    pub fn space(&self) -> &Rect {
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

// Rect //////////////////////////////////////////////////////////////////////
//
// A rectangle whose origin denotes the position of the bottom left corner.

#[derive(Clone, PartialEq, Debug)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: u32, height: u32) -> Rect {
        Rect {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
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
        let split = |n: u32| {
            let half = n >> 1;
            if n & 1 == 0 { (half, half) } else { (half, half + 1) }
        };

        let (x, y) = (self.origin.x, self.origin.y);
        let (w1, w2) = split(self.size.width);
        let (h1, h2) = split(self.size.height);

        let sw = Rect::new(x, y, w1, h1);
        let se = Rect::new(x + w1 as f32, y, w2, h1);
        let nw = Rect::new(x, y + h1 as f32, w1, h2);
        let ne = Rect::new(x + w1 as f32, y + h1 as f32, w2, h2);
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
            x: self.origin.x + self.size.width as f32,
            y: self.origin.y + self.size.height as f32,
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

    // Vector ////////////////////////////////////////////////////////////////

    #[test]
    fn vector_add_assigns() {
        // given
        let mut sut = Vector { dx: 3.0, dy: 4.0 };

        // when
        sut += Vector { dx: 9.5, dy: -3.5 };

        // then
        assert_eq!(sut, Vector { dx: 12.5, dy: 0.5 });
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
        assert_eq!(Vector { dx: 3.0, dy: 4.0 }.magnitude(), 5.0)
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
        assert_eq!(Vector::zero().normalized(), None)
    }

    // Size //////////////////////////////////////////////////////////////////

    #[test]
    #[should_panic(expected = "A size's width and/or height must be positive.")]
    fn size_non_positive_width() {
        // given, when , then
        Size::new(0, 1);
    }

    #[test]
    #[should_panic(expected = "A size's width and/or height must be positive.")]
    fn size_non_positive_height() {
        // given, when , then
        Size::new(10, 0);
    }

    // Rect //////////////////////////////////////////////////////////////////

    #[test]
    #[should_panic(expected = "A size's width and/or height must be positive.")]
    fn rect_non_positive_size() {
        // given, when , then
        Rect::new(-1.0, 1.0, 1, 0);
    }

    #[test]
    fn rect_even_quadrants() {
        // given
        let sut = Rect::new(0.0, 0.0, 6, 8);

        // when
        let (nw, ne, sw, se) = sut.quadrants();

        // then
        assert_eq!(nw, NW(Rect::new(0.0, 4.0, 3, 4)));
        assert_eq!(ne, NE(Rect::new(3.0, 4.0, 3, 4)));
        assert_eq!(sw, SW(Rect::new(0.0, 0.0, 3, 4)));
        assert_eq!(se, SE(Rect::new(3.0, 0.0, 3, 4)));
    }

    #[test]
    fn rect_uneven_quadrants() {
        // given
        let sut = Rect::new(0.0, 0.0, 5, 5);

        // when
        let (nw, ne, sw, se) = sut.quadrants();

        // then
        assert_eq!(nw, NW(Rect::new(0.0, 2.0, 2, 3)));
        assert_eq!(ne, NE(Rect::new(2.0, 2.0, 3, 3)));
        assert_eq!(sw, SW(Rect::new(0.0, 0.0, 2, 2)));
        assert_eq!(se, SE(Rect::new(2.0, 0.0, 3, 2)));
    }

    #[test]
    fn rect_contains_point() {
        // given
        let sut = Rect::new(0.0, 0.0, 10, 5);

        // then
        assert!(sut.contains(&Point::new(0.0, 0.0)));
        assert!(sut.contains(&Point::new(3.0, 3.0)));
        assert!(sut.contains(&Point::new(10.0, 5.0)));

        assert!(!sut.contains(&Point::new(-0.0001, 0.0)));
        assert!(!sut.contains(&Point::new(10.0000, 5.00001)));
        assert!(!sut.contains(&Point::new(14.0, 5.01)));
    }

    #[test]
    fn rect_which_quadrant() {
        // given
        let sut = Rect::new(0.0, 0.0, 5, 5);
        let (nw, ne, sw, se) = sut.quadrants();

        // then (bottom left of each quadrant)
        assert_eq!(sut.quadrant(&Point::new(0.0, 2.5)), Ok(nw.clone()));
        assert_eq!(sut.quadrant(&Point::new(2.5, 2.5)), Ok(ne.clone()));
        assert_eq!(sut.quadrant(&Point::new(0.0, 0.0)), Ok(sw.clone()));
        assert_eq!(sut.quadrant(&Point::new(2.5, 0.0)), Ok(se.clone()));

        // then (top right of each quadrant)
        assert_eq!(sut.quadrant(&Point::new(2.0, 5.0)), Ok(nw.clone()));
        assert_eq!(sut.quadrant(&Point::new(5.0, 5.0)), Ok(ne.clone()));
        assert_eq!(sut.quadrant(&Point::new(2.0, 1.0)), Ok(sw.clone()));
        assert_eq!(sut.quadrant(&Point::new(5.0, 1.0)), Ok(se.clone()));

        // then (anywhere in quadrant)
        assert_eq!(sut.quadrant(&Point::new(0.3, 2.9)), Ok(nw.clone()));
        assert_eq!(sut.quadrant(&Point::new(2.6, 4.2)), Ok(ne.clone()));
        assert_eq!(sut.quadrant(&Point::new(1.0, 2.0)), Ok(nw.clone()));
        assert_eq!(sut.quadrant(&Point::new(3.7, 2.4)), Ok(ne.clone()));

        // then
        assert_eq!(sut.quadrant(&Point::new(-2.5, 5.0)), Err(Error(OutOfBounds)));
        assert_eq!(sut.quadrant(&Point::new(5.4, 0.4)),  Err(Error(OutOfBounds)));
        assert_eq!(sut.quadrant(&Point::new(2.5, -4.0)), Err(Error(OutOfBounds)));
        assert_eq!(sut.quadrant(&Point::new(4.0, 6.5)),  Err(Error(OutOfBounds)));
    }
}
