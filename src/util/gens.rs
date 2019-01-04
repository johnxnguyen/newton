use rand::distributions::Uniform;
use rand::Rng;
use rand::thread_rng;
use rand::ThreadRng;

use physics::types::Mass;

// MassGen ///////////////////////////////////////////////////////////////////
//
// Uniformly generates random Mass within a closed range of f32 values.

struct MassGen {
    distribution: Uniform<f32>,
    rand: ThreadRng,
}

impl MassGen {
    fn new(low: f32, high: f32) -> MassGen {
        if low <= 0.0 || high <= 0.0 {
            panic!("MassGen requires positive range. Got [{}, {}]", low, high);
        }
        MassGen {
            distribution: Uniform::new_inclusive(low, high),
            rand: thread_rng(),
        }
    }

    fn next(&mut self) -> Mass {
        Mass::from(self.rand.sample(self.distribution))
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use physics::types::Mass;
    use util::gens::MassGen;

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
    #[should_panic]
    fn mass_gen_panics_on_invalid_range() {
        // when
        MassGen::new(2.0, 1.0);
    }

    #[test]
    fn mass_gen_generates() {
        // given
        let mut sut = MassGen::new(1.0, 2.0);
        let within_range = |m: Mass| m.value() >= 1.0 && m.value() <= 2.0;

        // then
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
        assert!(within_range(sut.next()));
    }
}