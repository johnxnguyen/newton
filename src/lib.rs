#[cfg(test)]
mod tests;

pub mod point;
pub mod vector;
pub mod body;
pub mod field;

use field::Field;
use body::Body;
use point::Point;
use vector::Vector;
use std::i32;

#[no_mangle]
pub extern fn newton_new_field(g: f64) -> *mut Field {
    let field = Field { g, bodies: vec![] };
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