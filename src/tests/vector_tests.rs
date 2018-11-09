use vector::Vector;

#[test]
fn it_add_assigns() {
    // given
    let mut sut = Vector { dx: 3.0, dy: 4.0 };
    
    // when
    sut += Vector { dx: 9.5, dy: -3.5 };
    
    // then
    assert_eq!(sut.dx, 12.5);
    assert_eq!(sut.dy, 0.5);
}

#[test]
fn it_scalar_multiplies() {
    // given
    let sut = Vector { dx: 3.0, dy: 4.0 };

    // when
    let result = &sut * 3.0;

    // then
    assert_eq!(result.dx, 9.0);
    assert_eq!(result.dy, 12.0);
}

#[test]
fn it_scalar_divides() {
    // given
    let sut = Vector { dx: 3.0, dy: 12.0 };

    // when
    let result = &sut / 3.0;

    // then
    assert_eq!(result.dx, 1.0);
    assert_eq!(result.dy, 4.0);
}

#[test]
fn it_calculates_magnitude() {
    // given
    let sut = Vector { dx: 3.0, dy: 4.0 };

    // when
    let result = sut.magnitude();

    // then
    assert_eq!(result, 5.0);
}

#[test]
fn it_normalizes() {
    // given
    let sut = Vector { dx: 3.3, dy: 5.2 };

    // when
    match sut.normalized() {
        None => {
            assert!(false)
        },
        Some(result) => {
            // then
            assert!(result.magnitude() > 0.999999);
            assert!(result.magnitude() < 1.000001);
        }
    };
}

#[test]
fn it_does_not_normalize_zero_vector() {
    // given
    let sut = Vector::zero();

    // when
    let result = sut.normalized();
    
    // then
    assert_eq!(result, None);
    
}