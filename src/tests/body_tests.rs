use body::Body;
use point::Point;
use vector::Vector;

#[test]
fn it_has_referential_equivalence() {
    // given
    let b1 = Body {
        id: 0,
        mass: 1.0,
        position: Point { x: 1, y: 2 },
        velocity: Vector::zero(),
    };

    let b2 = Body {
        id: 0,
        mass: 1.0,
        position: Point { x: 1, y: 2 },
        velocity: Vector::zero(),
    };

    // then
    assert_eq!(b1, b1);
    assert_ne!(b1, b2);
}

#[test]
fn it_applies_force() {
    // given
    let mut sut = Body {
        id: 0,
        mass: 2.0,
        position: Point { x: 1, y: 2 },
        velocity: Vector { dx: -2.0, dy: 5.0 },
    };

    let force = Vector { dx: 2.6, dy: -3.2 };
    
    // when
    sut.apply_force(&force);
        
    // then
    assert_eq!(sut.velocity.dx, -0.7);
    assert_eq!(sut.velocity.dy, 3.4);
    assert_eq!(sut.position.x, 0);
    assert_eq!(sut.position.y, 5);
}