use std::time::Instant;

use clap::{App, load_yaml, value_t};
use pbr::ProgressBar;

use newton::physics::types::Environment;
use newton::physics::field::*;
use newton::util::distribution::Loader;
use newton::util::write::DataWriter;

// TODO: Option for environment size (S, M, L, or exp)

fn main() {
    // Initialize CLI
    let yaml = load_yaml!("../cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    // Get args/opts/flags
    let path = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let frames = value_t!(matches, "FRAMES", u32).unwrap();
    let brute_force = matches.is_present("BRUTEFORCE");

    // Configure progress bar
    let mut progress = ProgressBar::new(frames as u64);
    progress.message("Frame ");
    progress.format("╢▌▌-╟");

    // Configure the environment
    let mut fields: Vec<Box<dyn Field>> = vec![];
    let writer = DataWriter::new(output);

    if brute_force {
        fields.push(Box::from(BruteForceField::new()));
    } else {
        fields.push(Box::from(BHField::new()));
    }

    let mut env = Environment::new(fields, writer);

    {
        let mut loader = Loader::new();
        env.bodies = loader.load_from_path(path).unwrap();
    }

    // Run the simulation
    // --------------------------------------------------------------------

    let stop_watch = StopWatch::start();

    for _ in 1..=frames {
        progress.inc();
        env.update();
    }

    let (secs, millis) = stop_watch.stop();
    println!("Total: {}.{} seconds.", secs, millis);
}

// STOPWATCH /////////////////////////////////////////////////////////////////

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
