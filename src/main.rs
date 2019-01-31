use newton::physics::types::Environment;
use newton::util::distribution::Loader;

use clap::App;
use clap::load_yaml;
use clap::value_t;

fn main() {
    let yaml = load_yaml!("../cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let path = matches.value_of("INPUT").unwrap();
    let frames = value_t!(matches, "FRAMES", u32).unwrap();

    let mut env = Environment::new();
    {
        let mut loader = Loader::new();
        env.bodies = loader.load_from_path(path).unwrap();
    }

    for x in 1..=frames {
        println!("frame: {}/{}", x, frames);
        env.update();
    }
}
