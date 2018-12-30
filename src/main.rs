extern crate newton;

use newton::physics::types::Environment;
use newton::geometry::util::Distributor;

fn main() {
    let mut env = Environment::new();
    let distributor = Distributor {
        num_bodies: 1000,
        min_dist: 50.0,
        max_dist: 250.0,
        dy: 10.0
    };

    env.bodies = distributor.distribution();

    for _ in 0..150 {
        env.update();
    }
}