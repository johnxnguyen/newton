use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use geometry::types::Point;
use geometry::types::Vector;
use geometry::util::Transformation;
use physics::types::Mass;
use util::gens::Generator;
use util::gens::MassGen;
use util::gens::RadialGen;
use util::gens::Repeater;
use util::gens::RotationGen;
use util::gens::UniformGen;
use util::gens::VelocityGen;
use util::gens::TranslationGen;

// Question: If I clone the gens, do they produce the same sequence?
// TODO: Replace distance_gens with translation_gens
pub struct Loader {
    mass_gens:        HashMap<String, MassGen>,
    translation_gens: HashMap<String, TranslationGen>,
    velocity_gens:    HashMap<String, VelocityGen>,
    rotation_gens:    HashMap<String, RotationGen>,
    radials_gens:     HashMap<String, RadialGen>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            mass_gens:        HashMap::new(),
            translation_gens: HashMap::new(),
            velocity_gens:    HashMap::new(),
            rotation_gens:    HashMap::new(),
            radials_gens:     HashMap::new(),
        }
    }

    pub fn load(&mut self, path: &str) {
        let docs = Loader::docs(path);
        let doc = &docs[0];

        // should define error type for useful feedback

        // need to give back errors instead of unwrapping

        // this could be refactored into a method
        let gens = doc["gens"].as_vec().unwrap();

        for gen in gens {
            let name = gen["name"].as_str().unwrap().to_owned();
            let gen_type = gen["type"].as_str().unwrap();

            match gen_type {
                "mass" => {
                    let mass_gen = self.parse_mass_gen(gen);
                    self.mass_gens.insert(name, mass_gen);
                },
                "translation" => {
                    let translation_gen = self.parse_translation_gen(gen);
                    self.translation_gens.insert(name, translation_gen);
                },
                "velocity" => {
                    let velocity_gen = self.parse_velocity_gen(gen);
                    self.velocity_gens.insert(name, velocity_gen);
                },
                "rotation" => {
                    let rotation_gen = self.parse_rotation_gen(gen);
                    self.rotation_gens.insert(name, rotation_gen);
                },
                "radial" => {
                    let radial_gen = self.parse_radial_gen(gen);
                    self.radials_gens.insert(name, radial_gen);
                },
                _ => panic!("Unknown generator type: {:?}", gen_type),
            };
        }

        println!("mass gens: {:?}", self.mass_gens.len());
        println!("trans gens: {:?}", self.translation_gens.len());
        println!("vel gens: {:?}", self.velocity_gens.len());
        println!("rot gens: {:?}", self.rotation_gens.len());
        println!("radials gens: {:?}", self.radials_gens.len());

        // now we create body nodes
        let bods = doc["bodies"].as_vec().unwrap();

        for bod in bods {
            let name = bod["name"].as_str().unwrap().to_owned();
            // this should be positive
            let num = bod["num"].as_i64().unwrap();
        }
    }

    fn docs(path: &str) -> Vec<Yaml> {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        YamlLoader::load_from_str(&contents).unwrap()
    }

    // Gen Parsing ///////////////////////////////////////////////////////////

    fn parse_mass_gen(&self, gen: &Yaml) -> MassGen {
        let low = gen["low"].as_f64().unwrap() as f32;
        let high = gen["high"].as_f64().unwrap() as f32;
        MassGen::new(low, high)
    }

    fn parse_translation_gen(&self, gen: &Yaml) -> TranslationGen {
        let x_min = gen["x"]["min"].as_i64().unwrap() as f32;
        let x_max = gen["x"]["max"].as_i64().unwrap() as f32;
        let y_min = gen["y"]["min"].as_i64().unwrap() as f32;
        let y_max = gen["y"]["max"].as_i64().unwrap() as f32;
        TranslationGen::new(x_min, x_max, y_min, y_max)
    }

    fn parse_rotation_gen(&self, gen: &Yaml) -> RotationGen {
        let low = gen["low"].as_f64().unwrap() as f32;
        let high = gen["high"].as_f64().unwrap() as f32;
        RotationGen::new_degrees(low, high)
    }

    fn parse_velocity_gen(&self, gen: &Yaml) -> VelocityGen {
        let vel_min = gen["vel"]["min"].as_f64().unwrap() as f32;
        let vel_max = gen["vel"]["max"].as_f64().unwrap() as f32;
        VelocityGen::new(0.0, 0.0, vel_min, vel_max)
    }

    fn parse_radial_gen(&self, gen: &Yaml) -> RadialGen {
        let translation = self.parse_translation_gen(gen);
        let rotation = RotationGen::new_degrees(0.0, 360.0);
        let velocity = self.parse_velocity_gen(gen);
        RadialGen::new(translation, rotation, velocity)
    }

    // Body Parsing //////////////////////////////////////////////////////////

    // how to handle missing keys and default values?
    fn parse_bod(&self, bod: &Yaml) -> (String, Vec<Node>) {
        let name = bod["name"].as_str().unwrap();
        let num = bod["num"].as_i64().unwrap_or(1); // should be positive

        let mut nodes: Vec<Node> = vec![];
        let mass = self.parse_body_mass(bod);
        let vel = self.parse_body_velocity(bod);
        let trans = self.parse_body_translation(bod);
        let rot = self.parse_body_rotation(bod);

        // radial gen?
        // might not even need radial gen anymore

        // make the nodes here
        (String::new(), vec![Node::Body(Point::zero(), Vector::zero(), 0.0)])
    }

    /// Returns the named mass gen if it exists, else creates one from concrete values.
    fn parse_body_mass(&self, body: &Yaml) -> Box<dyn Generator<Output=Mass>> {
        match body["mass"].as_str() {
            Some(gen_name) => {
                let gen = self.mass_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let raw = body["mass"].as_f64().unwrap() as f32;
                Box::new(Repeater::new(Mass::from(raw)))
            },
        }
    }

    /// Returns the named velocity gen if it exists, else creates one from concrete values.
    fn parse_body_velocity(&self, body: &Yaml) -> Box<dyn Generator<Output=Vector>> {
        match body["vel"].as_str() {
            Some(gen_name) => {
                let gen = self.velocity_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let dx = body["vel"]["dx"].as_f64().unwrap() as f32;
                let dy = body["vel"]["dy"].as_f64().unwrap() as f32;
                Box::new(Repeater::new(Vector::new(dx, dy)))
            },
        }
    }

    /// Returns the named rotation gen if it exists, else creates one from concrete values.
    fn parse_body_rotation(&self, body: &Yaml) -> Box<dyn Generator<Output=f32>> {
        match body["rot"].as_str() {
            Some(gen_name) => {
                let gen = self.rotation_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let rotation = body["rot"].as_f64().unwrap() as f32;
                Box::from(Repeater::new(rotation))
            },
        }
    }

    // Returns the named translation gen if it exists, else creates one from concrete values.
    fn parse_body_translation(&self, body: &Yaml) -> Box<dyn Generator<Output=Point>> {
        match body["trans"].as_str() {
            Some(gen_name) => {
                let gen = self.translation_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let x = body["trans"]["x"].as_i64().unwrap() as f32;
                let y  = body["trans"]["y"].as_i64().unwrap() as f32;
                Box::new(Repeater::new(Point::new(x, y)))
            },
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

enum Node {
    // translation, velocity, subsystems
    System(Point, Vector, Vec<Index>),
    // position, velocity, mass
    Body(Point, Vector, f32),
}

type Index = u32;

struct DistributionTree {
    nodes: Vec<Index>
}

impl DistributionTree {
    fn new() -> DistributionTree {
        DistributionTree { nodes: vec![] }
    }
}
