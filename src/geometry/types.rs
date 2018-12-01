use std::ops::{Add, AddAssign, Div, Mul};
use std::cmp::PartialEq;

// Point /////////////////////////////////////////////////////////////////////
//
// Coordinates in 2D space.

#[derive(PartialEq, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }

    pub fn is_origin(&self) -> bool {
        self == &Point::origin()
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let difference = Vector::difference(self, other);
        difference.magnitude()
    }
}

// Vector ////////////////////////////////////////////////////////////////////
//
// Change of coordinates in 2D space.

#[derive(Debug)]
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
        if self == &Vector::zero() {
            return None;
        }
        let magnitude = self.magnitude();

        Some(Vector {
            dx: self.dx / magnitude,
            dy: self.dy / magnitude,
        })
    }
}

// Rectangle /////////////////////////////////////////////////////////////////
#[derive(PartialEq, Debug)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(PartialEq, Debug)]
pub struct Rect {
    // the leftmost coordination system of the rectangle
    pub origin: Point,
    // the width and high of the rectangle
    pub size: Size,
}

impl Rect {
    fn quarter_sized(&self) -> Rect {
        Rect{
            origin: Point{
                x: self.origin.x,
                y: self.origin.y,
            },
            size: Size{
                width: self.size.width/ 2.0,
                height: self.size.height/2.0,
            }
        }
    }

    pub fn quadrants(&self) -> (Rect, Rect, Rect, Rect) {

        //the upper left rectangle
        let mut upper_left = self.quarter_sized();

        //the upper right rectangle
        let mut upper_right = self.quarter_sized();

        upper_right.origin.x = upper_right.origin.x + upper_right.size.width;

        //the lower right rectangle
        let mut lower_right = self.quarter_sized();
        lower_right.origin.x = lower_right.origin.x + lower_right.size.width;
        lower_right.origin.y = lower_right.origin.y + lower_right.size.height;

        //the lower left rectangle
        let mut lower_left = self.quarter_sized();
        lower_left.origin.y = lower_left.origin.y + lower_left.size.height;

        let rects= (upper_left, upper_right, lower_right, lower_left);

        rects
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
        match sut.normalized() {
            None => panic!(),
            Some(result) => {
                // then
                assert!((result.magnitude() - 1.0).abs() < 0.0000001);
            }
        };
    }

    #[test]
    fn vector_does_not_normalize_if_zero() {
        // given, when, then
        assert_eq!(Vector::zero().normalized(), None)
    }

    #[test]
    fn rect_quadrant() {
        // given
        let mut rect = Rect { origin: Point { x: 0.0, y: 0.0 },
                              size: Size { width: 6.0, height: 8.0 } };

        // when

        let (upper_left, upper_right,
             lower_right, lower_left) = rect.quadrants();

        let upper_left_test = Rect{origin: Point{x: 0.0, y:0.0},
                              size: Size {width:3.0, height: 4.0}};
        let upper_right_test = Rect{origin: Point{x: 3.0, y:0.0},
                              size: Size {width:3.0, height: 4.0}};
        let lower_right_test = Rect{origin: Point{x: 3.0, y:4.0},
                              size: Size {width:3.0, height: 4.0}};
        let lower_left_test = Rect{origin: Point{x: 0.0, y:4.0},
                              size: Size {width:3.0, height: 4.0}};

        // then
        assert_eq!(upper_left, upper_left_test);
        assert_eq!(upper_right, upper_right_test);
        assert_eq!(lower_right, lower_right_test);
        assert_eq!(lower_left, lower_left_test);
    }
}
