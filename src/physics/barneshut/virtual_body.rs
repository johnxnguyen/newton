use std::fmt;

use crate::geometry::Point;
use crate::geometry::Vector;
use crate::physics::Body;

// VirtualBody ///////////////////////////////////////////////////////////////
//
// A virtual body represents an amalgamation of real bodies. Its mass is the
// total sum of the collected masses and its position is the total sum of mass
// weighted positions. To obtain a copy with the position centered on its
// mass, call the `centered()` method.

#[derive(Clone, PartialEq, Debug)]
pub struct VirtualBody {
    pub mass: f32,
    pub position: Point,
}

impl fmt::Display for VirtualBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let v = self.centered();
        write!(f, "{:?}, ({:?}, {:?})", v.mass, v.position.x, v.position.y)?;
        Ok(())
    }
}

impl From<Body> for VirtualBody {
    fn from(body: Body) -> Self {
        VirtualBody {
            mass: body.mass.value(),
            position: &body.position * body.mass.value(),
        }
    }
}

impl VirtualBody {
    pub fn to_body(self) -> Body {
        Body::new(self.mass, self.position.clone(), Vector::zero())
    }

    pub fn new(mass: f32, x: f32, y: f32) -> VirtualBody {
        VirtualBody {
            mass,
            position: Point::new(x, y),
        }
    }

    pub fn zero() -> VirtualBody {
        VirtualBody::new(0.0, 0.0, 0.0)
    }

    pub fn centered(&self) -> VirtualBody {
        debug_assert!(self.mass > 0.0, "Mass must be positive. Got {}", self.mass);
        VirtualBody {
            mass: self.mass,
            position: &self.position / self.mass,
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_body_centered() {
        // given, then
        let sut = VirtualBody::new(2.5, 5.0, 7.5);
        assert_eq!(Point::new(2.0, 3.0), sut.centered().position);

        // given, then
        let sut = VirtualBody::new(2.4, -24.6, -4.8);
        assert_eq!(Point::new(-10.25, -2.0), sut.centered().position);

        // given, then
        let sut = VirtualBody::new(14.5, 0.0, 0.0);
        assert_eq!(Point::zero(), sut.centered().position);
    }

    #[test]
    #[should_panic]
    fn virtual_body_centered_zero_mass() {
        // given, when
        VirtualBody::new(0.0, 5.0, 7.5).centered();
    }
}
