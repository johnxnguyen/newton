use rand::distributions::Uniform;
use rand::Rng;
use rand::thread_rng;
use rand::ThreadRng;

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

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use physics::types::Mass;
    use util::gens::MassGen;
    use util::gens::UniformGen;

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
}
