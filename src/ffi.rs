use geometry::types::{Point, Vector};
use physics::types::{Body, Field};
use std::i32;

//////////////////////////////////////////////////////////////////////////////

#[repr(C)]
pub struct NewtonPoint {
    pub x: i32,
    pub y: i32,
}

impl NewtonPoint {
    fn from(point: &Point) -> NewtonPoint {
        NewtonPoint {
            x: point.x,
            y: point.y,
        }
    }
}

#[no_mangle]
pub extern "C" fn newton_new_field(
    g: f64,
    solar_mass: f64,
    min_dist: f64,
    max_dist: f64,
) -> *mut Field {
    let field = Field::new(g, solar_mass, min_dist, max_dist);
    let boxed = Box::new(field);
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn newton_destroy_field(field: *mut Field) {
    let _ = Box::from_raw(field);
}

#[no_mangle]
pub unsafe extern "C" fn newton_add_body(
    field: *mut Field,
    id: u32,
    mass: f64,
    x: i32,
    y: i32,
    dx: f64,
    dy: f64,
) {
    let body = Body::new(id, mass, Point { x, y }, Vector { dx, dy });
    let field = &mut *field;
    field.bodies.insert(id, body);
}

#[no_mangle]
pub unsafe extern "C" fn newton_distribute_bodies(
    field: *mut Field,
    num_bodies: u32,
    min_dist: u32,
    max_dist: u32,
    dy: f64,
) {
    let distributor = ::geometry::util::Distributor {
        num_bodies,
        min_dist,
        max_dist,
        dy,
    };

    let bodies = distributor.distribution();
    let field = &mut *field;
    field.bodies = bodies;
}

#[no_mangle]
pub unsafe extern "C" fn newton_step(field: *mut Field) {
    let field = &mut *field;
    field.update()
}

#[no_mangle]
pub unsafe extern "C" fn newton_body_pos(field: *const Field, id: u32) -> NewtonPoint {
    let field = &*field;
    match field.bodies.get(&id) {
        Some(val) => NewtonPoint::from(&((val as &Body).position)),
        None => NewtonPoint {
            x: 0,
            y: 0,
        },
    }
}
