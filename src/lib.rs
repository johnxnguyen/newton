#[cfg(test)]
mod tests;
mod point;
mod vector;
mod body;
mod field;
mod transformation;
mod util;

extern crate rand;

use field::Field;
use body::Body;
use point::Point;
use vector::Vector;
use std::i32;

#[no_mangle]
pub extern fn newton_new_field(g: f64, solar_mass: f64, min_dist: f64, max_dist: f64) -> *mut Field {
    let field = Field { g, solar_mass, min_dist, max_dist, bodies: vec![] };
    let boxed = Box::new(field);
    println!("A field has been allocated");
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern fn newton_destroy_field(field: *mut Field) {
    let _ = Box::from_raw(field);
}

#[no_mangle]
pub unsafe extern fn newton_add_body(field: *mut Field, id: u32, mass: f64, x: i32, y: i32, dx: f64, dy: f64) {
    let body = Body {
        id,
        mass,
        position: Point { x, y },
        velocity: Vector { dx, dy },
    };

    println!("Body #{} is created.", id);
    let field = &mut *field;
    field.bodies.push(body);
}

#[no_mangle]
pub unsafe extern fn newton_distribute_bodies(field: *mut Field, num_bodies: u32, min_dist: u32, max_dist: u32, dy: f64) {
    let distributor = util::Distributor {
        num_bodies,
        min_dist,
        max_dist,
        dy,
    };

    let bodies = distributor.get();
    let field = &mut *field;
    field.bodies = bodies;
}

#[no_mangle]
pub unsafe extern fn newton_step(field: *mut Field) {
    let field = &mut *field;
    field.update()
}

#[no_mangle]
pub unsafe extern fn newton_body_x_pos(field: *const Field, id: u32) -> i32 {
    let field = &* field;
    match field.bodies.get(id as usize) {
        Some(val) => (val as &Body).position.x,
        None => i32::MAX
    }
}

#[no_mangle]
pub unsafe extern fn newton_body_y_pos(field: *const Field, id: u32) -> i32 {
    let field = &* field;
    match field.bodies.get(id as usize) {
        Some(val) => (val as &Body).position.y,
        None => i32::MAX
    }
}