use transformation::Transformation;
use vector::Vector;
use std::f64::consts::FRAC_PI_2;

#[test]
fn it_transforms_a_vector() {
    // given
    let sut = Transformation {
        a: Vector { dx: 2.0, dy: 0.0 },
        b: Vector { dx: 0.0, dy: 2.0 },
    };

    // when
    let result = &sut * Vector { dx: 4.0, dy: -2.5 };

    // then
    assert_eq!(result, Vector { dx: 8.0, dy: -5.0 });
}

#[test]
fn it_rotates_a_vector() {
    // given
    let sut = Transformation::rotation(FRAC_PI_2);

    // when
    let result = &sut * Vector { dx: 1.0, dy: 0.0 };

    // then
    assert_eq!(result, Vector { dx: 0.0, dy: 1.0 });
}