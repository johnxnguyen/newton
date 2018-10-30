use util::Point;

#[test]
fn it_calculates_distance_from_origin() {
    // given
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 5, y: 0 };

    // when
    let result = p1.distance_to(&p2);

    // then
    assert_eq!(result, 5.0);
}

#[test]
fn it_calculates_distance_to_origin() {
    // given
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 0, y: -5 };

    // when
    let result = p2.distance_to(&p1);

    // then
    assert_eq!(result, 5.0);
}