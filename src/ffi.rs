use geometry::types::{Point};
use physics::types::{Body, Environment};

//////////////////////////////////////////////////////////////////////////////

#[repr(C)]
pub struct NewtonPoint {
    pub x: f32,
    pub y: f32,
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
pub extern "C" fn newton_new_environment() -> *mut Environment {
    let environment = Environment::new();
    let boxed = Box::new(environment);
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn newton_destroy_environment(environment: *mut Environment) {
    let _ = Box::from_raw(environment);
}

#[no_mangle]
pub unsafe extern "C" fn newton_distribute_bodies(
    environment: *mut Environment,
    num_bodies: u32,
    min_dist: f32,
    max_dist: f32,
    dy: f32,
) {
    let distributor = ::geometry::util::Distributor {
        num_bodies,
        min_dist,
        max_dist,
        dy,
    };

    let bodies = distributor.distribution();
    let environment = &mut *environment;
    environment.bodies = bodies;
}

#[no_mangle]
pub unsafe extern "C" fn newton_step(environment: *mut Environment) {
    let environment = &mut *environment;
    environment.update()
}

#[no_mangle]
pub unsafe extern "C" fn newton_body_pos(environment: *const Environment, id: u32) -> NewtonPoint {
    let environment = &*environment;
    match environment.bodies.get(id as usize) {
        Some(val) => NewtonPoint::from(&((val as &Body).position)),
        None => NewtonPoint {
            x: 0.0,
            y: 0.0,
        },
    }
}
