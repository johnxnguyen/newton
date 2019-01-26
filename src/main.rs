extern crate newton;

use newton::physics::types::Environment;
use newton::geometry::util::Distributor;
use newton::util::distribution::Loader;

fn main() {
    let mut loader = Loader::new();
    loader.load_path("Radial5000.yaml");
    let mut env = Environment::new();
    env.bodies = loader.bodies();

    let upper = 5;
    for x in 1..=upper {
        println!("frame: {}/{}", x, upper);
        env.update();
    }
}