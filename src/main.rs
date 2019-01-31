use clap::App;
use clap::load_yaml;
use clap::value_t;

use newton::physics::types::Environment;
use newton::util::distribution::Loader;
use newton::util::write::DataWriter;

fn main() {
    let yaml = load_yaml!("../cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let path = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let frames = value_t!(matches, "FRAMES", u32).unwrap();

    let writer = DataWriter::new(output);
    let mut env = Environment::new(writer);

    {
        let mut loader = Loader::new();
        env.bodies = loader.load_from_path(path).unwrap();
    }

    for x in 1..=frames {
        println!("frame: {}/{}", x, frames);
        env.update();
    }
}
