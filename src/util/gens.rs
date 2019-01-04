use std::f32::consts::PI;

use rand::distributions::Uniform;
use rand::Rng;
use rand::thread_rng;
use rand::ThreadRng;

use geometry::types::Vector;
use physics::types::Mass;

// UniformGen ////////////////////////////////////////////////////////////////
//
// Uniformly generates random f32 within a closed range of values.

struct UniformGen {
    distribution: Uniform<f32>,
    rand: ThreadRng,
}

impl UniformGen {
    fn new(low: f32, high: f32) -> UniformGen {
        UniformGen {
            distribution: Uniform::new_inclusive(low, high),
            rand: thread_rng(),
        }
    }

    fn next(&mut self) -> f32 {
        self.rand.sample(self.distribution)
    }
}

// MassGen ///////////////////////////////////////////////////////////////////
//
// Uniformly generates random Mass within a closed range.

struct MassGen {
    gen: UniformGen,
}

impl MassGen {
    fn new(low: f32, high: f32) -> MassGen {
        if low <= 0.0 || high <= 0.0 {
            panic!("MassGen requires positive range. Got [{}, {}]", low, high);
        }

        MassGen { gen: UniformGen::new(low, high) }
    }

    fn next(&mut self) -> Mass {
        Mass::from(self.gen.next())
    }
}

// RotationGen ///////////////////////////////////////////////////////////////
//
// Uniformly generates random angles (in radians) within a closed range.

struct RotationGen {
    gen: UniformGen,
}

impl RotationGen {
    fn new_radians(low: f32, high: f32) -> RotationGen {
        let (low, high) = RotationGen::normalize(low, high);
        RotationGen { gen: UniformGen::new(low, high) }
    }

    fn new_degrees(low: f32, high: f32) -> RotationGen {
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

    fn next(&mut self) -> f32 {
        self.gen.next()
    }
}

// VelocityGen ///////////////////////////////////////////////////////////////
//
// Uniformly generates random velocities within closed ranges.

struct VelocityGen {
    dx: UniformGen,
    dy: UniformGen,
}

impl VelocityGen {
    fn new(dx_low: f32, dx_high: f32, dy_low: f32, dy_high: f32) -> VelocityGen {
        VelocityGen {
            dx: UniformGen::new(dx_low, dx_high),
            dy: UniformGen::new(dy_low, dy_high),
        }
    }

    fn next(&mut self) -> Vector {
        Vector {
            dx: self.dx.next(),
            dy: self.dy.next(),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use geometry::types::Vector;
    use physics::types::Mass;
    use util::gens::MassGen;
    use util::gens::RotationGen;
    use util::gens::UniformGen;
    use util::gens::VelocityGen;

    #[test]
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
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
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
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
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
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
    }

    #[test]
    fn rotation_gen_from_degrees_generates() {
        // given
        let mut sut = RotationGen::new_degrees(90.0, 180.0);
        let within_range = |r| r >= 0.5 * PI && r <= PI;

        // then
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
    }

    #[test]
    fn velocity_gen_generates() {
        // given
        let mut sut = VelocityGen::new(-1.0, 1.0, 2.0, 3.0);
        let within_range = |v: Vector| {
            v.dx >= -1.0 && v.dx <= 1.0 && v.dy >= 2.0 && v.dy <= 3.0
        };

        // then
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
    }
}
