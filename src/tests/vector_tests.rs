use point::Vector;

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
    let mut sut = Vector { dx: 3.3, dy: 5.2 };

    // when
    sut.normalize();

    // then
    assert!(sut.magnitude() > 0.999999);
    assert!(sut.magnitude() < 1.000001);
}