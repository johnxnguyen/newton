extern crate newton;

use newton::physics::types::Environment;
use newton::geometry::util::Distributor;
use newton::util::distribution::Loader;

fn main() {
    let mut loader = Loader::new();
    loader.load("config.yaml");
//    let mut env = Environment::new();
//    let distributor = Distributor {
//        num_bodies: 5000,
//        min_dist: 200.0,
//        max_dist: 250.0,
//        dy: 10.0
//    };
//
//    env.bodies = distributor.distribution();
//
//    let upper = 500;
//    for x in 1..=upper {
//        println!("frame: {}/{}", x, upper);
//        env.update();
//    }
}