use newton::physics::types::Environment;
use newton::util::distribution::Loader;

fn main() {
    let mut loader = Loader::new();
    let bodies = loader.load_from_path("configurations/Radial5000.yaml").unwrap();
    let mut env = Environment::new();
    env.bodies = bodies;

    let upper = 5;
    for x in 1..=upper {
        println!("frame: {}/{}", x, upper);
        env.update();
    }
}