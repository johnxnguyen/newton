use std::f32::consts::PI;

use rand::distributions::Uniform;
use rand::Rng;
use rand::thread_rng;
use rand::ThreadRng;

use geometry::types::Point;
use geometry::types::Vector;
use geometry::util::Transformation;
use physics::types::Mass;

// Generator /////////////////////////////////////////////////////////////////
//
// A type conforming to Generator produces an infinite stream of objects.

pub trait Generator {
    type Output;
    fn generate(&mut self) -> Self::Output;
}

// Repeater //////////////////////////////////////////////////////////////////
//
// A simple generator that generates a single output.

pub struct Repeater<T> {
    value: T,
}

impl<T> Repeater<T> {
    pub fn new(value: T) -> Repeater<T> {
        Repeater { value }
    }
}

impl<T> Generator for Repeater<T> where T: Clone {
    type Output = T;
    fn generate(&mut self) -> Self::Output {
        self.value.clone()
    }
}

// UniformGen ////////////////////////////////////////////////////////////////
//
// Uniformly generates random f32 within a closed range of values.

#[derive(Debug)]
pub struct UniformGen {
    distribution: Uniform<f32>,
    rand: ThreadRng,
}

impl UniformGen {
    pub fn new(low: f32, high: f32) -> UniformGen {
        UniformGen {
            distribution: Uniform::new_inclusive(low, high),
            rand: thread_rng(),
        }
    }
}

impl Generator for UniformGen {
    type Output = f32;
    fn generate(&mut self) -> Self::Output {
        self.rand.sample(self.distribution)
    }
}

// MassGen ///////////////////////////////////////////////////////////////////
//
// Uniformly generates random Mass within a closed range.

#[derive(Debug)]
pub struct MassGen {
    gen: UniformGen,
}

impl MassGen {
    pub fn new(low: f32, high: f32) -> MassGen {
        if low <= 0.0 || high <= 0.0 {
            panic!("MassGen requires positive range. Got [{}, {}]", low, high);
        }

        MassGen { gen: UniformGen::new(low, high) }
    }
}

impl Generator for MassGen {
    type Output = Mass;
    fn generate(&mut self) -> Self::Output {
        Mass::from(self.gen.generate())
    }
}

// RotationGen ///////////////////////////////////////////////////////////////
//
// Uniformly generates random angles (in radians) within a closed range.

#[derive(Debug)]
pub struct RotationGen {
    gen: UniformGen,
}

impl RotationGen {
    pub fn new_radians(low: f32, high: f32) -> RotationGen {
        let (low, high) = RotationGen::normalize(low, high);
        RotationGen { gen: UniformGen::new(low, high) }
    }

    pub fn new_degrees(low: f32, high: f32) -> RotationGen {
        let low = RotationGen::radians(low);
        let high = RotationGen::radians(high);
        RotationGen::new_radians(low, high)
    }

    fn radians(degrees: f32) -> f32 {
        degrees * PI / 180.0
    }

    fn normalize(mut low: f32, mut high: f32) -> (f32, f32) {
        let pi_2 = 2.0 * PI;

        // add 2PI to low until it it exceeds 0
        while low + pi_2 <= 0.0 { low += pi_2; }

        // subtract 2PI from high until it is <= 0
        while  high - pi_2 > 0.0 { high -= pi_2; }

        (low, high)
    }
}

impl Generator for RotationGen {
    type Output = f32;
    fn generate(&mut self) -> Self::Output {
        self.gen.generate()
    }
}

// VelocityGen ///////////////////////////////////////////////////////////////
//
// Uniformly generates random velocities within closed ranges.

#[derive(Debug)]
pub struct VelocityGen {
    dx: UniformGen,
    dy: UniformGen,
}

impl VelocityGen {
    pub fn new(dx_low: f32, dx_high: f32, dy_low: f32, dy_high: f32) -> VelocityGen {
        VelocityGen {
            dx: UniformGen::new(dx_low, dx_high),
            dy: UniformGen::new(dy_low, dy_high),
        }
    }
}

impl Generator for VelocityGen {
    type Output = Vector;

    fn generate(&mut self) -> Self::Output {
        Vector {
            dx: self.dx.generate(),
            dy: self.dy.generate(),
        }
    }
}

// RadialGen /////////////////////////////////////////////////////////////////
//
// Generates positions and velocities radially around the origin.

#[derive(Debug)]
pub struct RadialGen {
    distance: UniformGen,
    rotation: RotationGen,
    velocity: VelocityGen,
}

impl RadialGen {
    pub fn new(distance: UniformGen, rotation: RotationGen, velocity: VelocityGen) -> RadialGen {
        RadialGen { distance, rotation, velocity }
    }
}

impl Generator for RadialGen {
    type Output = (Point, Vector);
    fn generate(&mut self) -> <Self as Generator>::Output {
        let t = Transformation::rotation(self.rotation.generate());
        let point = Point::from(&t * Vector::new(self.distance.generate(), 0.0));
        let velocity =  &t * self.velocity.generate();
        (point, velocity)
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use geometry::types::Point;
    use geometry::types::Vector;
    use physics::types::Mass;
    use util::gens::*;

    #[test]
    fn repeater_generates() {
        // given
        let mut sut = Repeater::new(Point::new(1.0, 2.0));

        // then
        let expected = Point::new(1.0, 2.0);
        assert_eq!(expected, sut.generate());
        assert_eq!(expected, sut.generate());
        assert_eq!(expected, sut.generate());
        assert_eq!(expected, sut.generate());
    }

    #[should_panic]
    fn uniform_gen_panics_on_invalid_range() {
        // when
        UniformGen::new(2.0, 1.0);
    }

    #[test]
    fn uniform_gen_generates() {
        // given
        let mut sut = UniformGen::new(1.0, 2.0);
        let within_range = |n: f32| n >= 1.0 && n <= 2.0;

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    #[should_panic]
    fn mass_gen_panics_on_non_positive_low() {
        // when
        MassGen::new(-2.0, 2.0);
    }

    #[test]
    #[should_panic]
    fn mass_gen_panics_on_non_positive_high() {
        // when
        MassGen::new(1.0, 0.0);
    }

    #[test]
    fn mass_gen_generates() {
        // given
        let mut sut = MassGen::new(1.0, 2.0);
        let within_range = |n: Mass| n.value() >= 1.0 && n.value() <= 2.0;

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    fn rotation_gen_radians() {
        assert_eq!(0.0, RotationGen::radians(0.0));
        assert_eq!(0.5 * PI, RotationGen::radians(90.0));
        assert_eq!(PI, RotationGen::radians(180.0));
        assert_eq!(1.5 * PI, RotationGen::radians(270.0));
        assert_eq!(2.0 * PI, RotationGen::radians(360.0));
    }

    #[test]
    fn rotation_gen_normalizes() {
        // given
        let low = -17.3 * PI;
        let high = 44.8 * PI;

        // when
        let (low, high) = RotationGen::normalize(low, high);

        // then
        assert_eq!(-4.0840707, low);
        assert_eq!(2.5132432, high);
    }

    #[test]
    fn rotation_gen_generates() {
        // given
        let mut sut = RotationGen::new_radians(0.5 * PI, PI);
        let within_range = |r| r >= 0.5 * PI && r <= PI;

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    fn rotation_gen_from_degrees_generates() {
        // given
        let mut sut = RotationGen::new_degrees(90.0, 180.0);
        let within_range = |r| r >= 0.5 * PI && r <= PI;

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    fn velocity_gen_generates() {
        // given
        let mut sut = VelocityGen::new(-1.0, 1.0, 2.0, 3.0);
        let within_range = |v: Vector| {
            v.dx >= -1.0 && v.dx <= 1.0 && v.dy >= 2.0 && v.dy <= 3.0
        };

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    fn radial_gen_generates() {
        // given
        let distance = UniformGen::new(100.0, 200.0);
        let rotation = RotationGen::new_radians(0.0, PI);
        let velocity = VelocityGen::new(1.0, 2.0, 3.0, 4.0);
        let mut sut = RadialGen::new(distance, rotation, velocity);

        let within_range = |(p, v): (Point, Vector)| {
            let dist_to_origin = p.distance_to(&Point::zero());
            let dist_in_range = dist_to_origin >= 100.0 && dist_to_origin <= 200.0;
            let y_is_positive = p.y >= 0.0;
            let v_min = v.magnitude() >= Vector::new(1.0, 2.0).magnitude();
            let v_max = v.magnitude() <= Vector::new(3.0, 4.0).magnitude();

            dist_in_range && y_is_positive && v_min && v_max
        };

        // then
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
        assert!(within_range(sut.generate()));
    }

    #[test]
    fn radial_gen_rotates_velocity() {
        // given
        let distance = UniformGen::new(100.0, 200.0);
        let rotation = RotationGen::new_radians(0.0, PI);
        let velocity = VelocityGen::new(0.0, 0.0, 1.0, 2.0);
        let mut sut = RadialGen::new(distance, rotation, velocity);

        // then
        for _ in 0..5 {
            // since the distance extends only in the positive x axis before rotation
            // and the velocity extends only in the positive y axis before rotation,
            // they are orthogonal before rotation. Therefore, if they are rotated
            // together, they must remain orthogonal.
            let (p, v) = sut.generate();
            let p = Vector::new(p.x, p.y);
            let dot_product = &p * &v;

            // to account for floating point inaccuracies.
            assert!(dot_product.abs() >= 0.0);
            assert!(dot_product.abs() <= 0.0001);
        }
    }
}
