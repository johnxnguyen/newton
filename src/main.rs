use clap::App;
use clap::load_yaml;
use clap::value_t;

use newton::physics::types::Environment;
use newton::util::distribution::Loader;
use newton::util::write::DataWriter;
use std::time::Instant;
use pbr::ProgressBar;

// TODO: Option for environment size (S, M, L, or exp)
// TODO: Flag for brute force

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

    let mut progress = ProgressBar::new(frames as u64);
    progress.message("Frame ");
    progress.format("╢▌▌-╟");

    let stop_watch = StopWatch::start();

    for _ in 1..=frames {
        progress.inc();
        env.update();
    }

    let (secs, millis) = stop_watch.stop();
    println!("Total: {}.{} seconds.", secs, millis);
}

type SecsMillis = (u64, u32);

struct StopWatch {
    start: Instant
}

impl StopWatch {
    fn start() -> StopWatch {
        StopWatch {
            start: Instant::now()
        }
    }

    fn stop(&self) -> SecsMillis {
        let duration = self.start.elapsed();
        (duration.as_secs(), duration.subsec_millis())
    }
}
