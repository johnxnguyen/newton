use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use geometry::types::Point;
use geometry::types::Vector;
use geometry::util::Transformation;
use physics::types::Mass;
use util::gens::*;

// Question: If I clone the gens, do they produce the same sequence?

// should define error type for useful feedback

// need to give back errors instead of unwrapping

// need to provide default values

pub struct Loader {
    mass_gens: HashMap<String, MassGen>,
    translation_gens: HashMap<String, TranslationGen>,
    velocity_gens: HashMap<String, VelocityGen>,
    rotation_gens: HashMap<String, RotationGen>,
    bodies: HashMap<String, Vec<Node>>,
    tree: DistributionTree,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            mass_gens: HashMap::new(),
            translation_gens: HashMap::new(),
            velocity_gens: HashMap::new(),
            rotation_gens: HashMap::new(),
            bodies: HashMap::new(),
            tree: DistributionTree::new(),
        }
    }

    fn docs(path: &str) -> Vec<Yaml> {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        YamlLoader::load_from_str(&contents).unwrap()
    }

    pub fn load(&mut self, path: &str) {
        let docs = Loader::docs(path);
        let doc = &docs[0];

        let gens = doc["gens"].as_vec().unwrap();
        self.parse_gens(gens);

        let bodies = doc["bodies"].as_vec().unwrap();
        self.parse_bodies(bodies);

        // this should only have one element
        let root_idx = self.parse_system(doc);
    }

    // Gen Parsing ///////////////////////////////////////////////////////////

    fn parse_gens(&mut self, gens: &Vec<Yaml>) {
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
                _ => panic!("Unknown generator type: {:?}", gen_type),
            };
        }
    }

    fn parse_mass_gen(&self, gen: &Yaml) -> MassGen {
        let min = gen["min"].as_f64().unwrap() as f32;
        let max = gen["max"].as_f64().unwrap() as f32;
        MassGen::new(min, max)
    }

    fn parse_translation_gen(&self, gen: &Yaml) -> TranslationGen {
        let x_min = gen["x"]["min"].as_f64().unwrap() as f32;
        let x_max = gen["x"]["max"].as_f64().unwrap() as f32;
        let y_min = gen["y"]["min"].as_f64().unwrap() as f32;
        let y_max = gen["y"]["max"].as_f64().unwrap() as f32;
        TranslationGen::new(x_min, x_max, y_min, y_max)
    }

    fn parse_rotation_gen(&self, gen: &Yaml) -> RotationGen {
        let min = gen["min"].as_f64().unwrap() as f32;
        let max = gen["max"].as_f64().unwrap() as f32;
        RotationGen::new_degrees(min, max)
    }

    fn parse_velocity_gen(&self, gen: &Yaml) -> VelocityGen {
        let dx_min = gen["dx"]["min"].as_f64().unwrap() as f32;
        let dx_max = gen["dx"]["max"].as_f64().unwrap() as f32;
        let dy_min = gen["dy"]["min"].as_f64().unwrap() as f32;
        let dy_max = gen["dy"]["max"].as_f64().unwrap() as f32;
        VelocityGen::new(dx_min, dx_max, dy_min, dy_max)
    }

    // Body Parsing //////////////////////////////////////////////////////////

    fn parse_bodies(&mut self, bodies: &Vec<Yaml>) {
        for body in bodies {
            let (name, nodes) = self.parse_body(body);
            self.bodies.insert(name, nodes);
        };
    }

    // how to handle missing keys and default values?
    fn parse_body(&self, body: &Yaml) -> (String, Vec<Node>) {
        let name = body["name"].as_str().unwrap();
        let num = body["num"].as_i64().unwrap_or(1); // should be positive

        let mut nodes: Vec<Node> = vec![];
        let mut mass = self.parse_mass(body);
        let mut vel = self.parse_velocity(body);
        let mut trans = self.parse_translation(body);
        let mut rot = self.parse_rotation(body);

        let mut nodes: Vec<Node> = Vec::new();

        for _ in 1..=num {
            let tvr = TVR(trans.generate(), vel.generate(), rot.generate());
            let node = Node::Body(tvr, mass.generate());
            nodes.push(node);
        }

        (String::from(name), nodes)
    }

    /// Returns the named mass gen if it exists, else creates one from concrete values.
    fn parse_mass(&self, object: &Yaml) -> Box<dyn Generator<Output=Mass>> {
        match object["mass"].as_str() {
            Some(gen_name) => {
                let gen = self.mass_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let raw = object["mass"].as_f64().unwrap() as f32;
                Box::new(Repeater::new(Mass::from(raw)))
            },
        }
    }

    // Returns the named translation gen if it exists, else creates one from concrete values.
    fn parse_translation(&self, object: &Yaml) -> Box<dyn Generator<Output=Point>> {
        match object["trans"].as_str() {
            Some(gen_name) => {
                let gen = self.translation_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let x = object["trans"]["x"].as_i64().unwrap() as f32;
                let y = object["trans"]["y"].as_i64().unwrap() as f32;
                Box::new(Repeater::new(Point::new(x, y)))
            },
        }
    }

    /// Returns the named velocity gen if it exists, else creates one from concrete values.
    fn parse_velocity(&self, object: &Yaml) -> Box<dyn Generator<Output=Vector>> {
        match object["vel"].as_str() {
            Some(gen_name) => {
                let gen = self.velocity_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let dx = object["vel"]["dx"].as_f64().unwrap() as f32;
                let dy = object["vel"]["dy"].as_f64().unwrap() as f32;
                Box::new(Repeater::new(Vector::new(dx, dy)))
            },
        }
    }

    /// Returns the named rotation gen if it exists, else creates one from concrete values.
    fn parse_rotation(&self, object: &Yaml) -> Box<dyn Generator<Output=f32>> {
        match object["rot"].as_str() {
            Some(gen_name) => {
                let gen = self.rotation_gens.get(gen_name).unwrap().clone();
                Box::new(gen)
            },
            None => {
                let rotation = object["rot"].as_f64().unwrap() as f32;
                Box::from(Repeater::new(rotation))
            },
        }
    }

    // System Parsing ////////////////////////////////////////////////////////////

    fn parse_system(&mut self, system: &Yaml) -> Vec<Index> {
        // check for reference to bodies
        match system["name"].as_str() {
            None => (),
            Some(name) => {
                let mut indices = vec![];
                for body in self.bodies.remove(name).unwrap() {
                    indices.push(self.tree.add_node(body));
                }

                return indices
            },
        }

        // transformation for the system
        let tvr: TVR;
        {
            let t = self.parse_translation(system).generate();
            let v = self.parse_velocity(system).generate();
            let r = self.parse_rotation(system).generate();
            tvr = TVR(t, v, r);
        }

        // parse the subsystems
        let mut subsystems: Vec<Index> = vec![];
        for subsystem in system["systems"].as_vec().unwrap() {
            let mut indices = self.parse_system(subsystem);
            subsystems.append(&mut indices);
        }

        // finally build the node & return its index
        let idx = self.tree.add_node(Node::System(tvr, subsystems));
        vec![idx]
    }
}

//////////////////////////////////////////////////////////////////////////////

type Index = u32;
struct TVR(Point, Vector, f32);

enum Node {
    System(TVR, Vec<Index>),
    Body(TVR, Mass),
}

struct DistributionTree {
    nodes: Vec<Node>
}

impl DistributionTree {
    fn new() -> DistributionTree {
        DistributionTree { nodes: vec![] }
    }

    fn add_node(&mut self, node: Node) -> Index {
        self.nodes.push(node);
        (self.nodes.len() - 1) as u32
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use yaml_rust::Yaml;
    use yaml_rust::YamlLoader;

    use util::distribution::Loader;
    use util::gens::Generator;
    use util::gens::MassGen;

    fn yaml(raw: &str) -> Yaml {
        match YamlLoader::load_from_str(raw) {
            Ok(_) => {
                // all good here
            },
            Err(_) => {
                // oh no!
                panic!("OH NO!")
            },
        }

        let mut docs = YamlLoader::load_from_str(raw).unwrap();
        docs.remove(0)
    }

    // TODO: Should test panic cases too. And defaults!

    #[test]
    fn loader_parse_mass_gen() {
        // given
        let sut = Loader::new();
        let input = "
        name: MASSATRON 1000
        type: mass
        min: 0.1
        max: 0.3";

        // when
        let mut result = sut.parse_mass_gen(&yaml(input));

        // then
        assert!(result.generate().value() > 0.1);
        assert!(result.generate().value() < 0.3);
    }

    #[test]
    fn loader_parse_translation_gen() {
        // given
        let sut = Loader::new();
        let input = "
        name: THE TRANSLATOR
        type: translation
        x: {min: -10.0, max: 10.0}
        y: {min:  10.0, max: 20.0}";

        // when
        let mut result = sut.parse_translation_gen(&yaml(input));

        // then
        let point = result.generate();
        assert!(point.x >= -10.0);
        assert!(point.x <= 10.0);
        assert!(point.y >= 10.0);
        assert!(point.y <= 20.0);
    }

    #[test]
    fn loader_parse_velocity_gen() {
        // given
        let sut = Loader::new();
        let input = "
        name: VELOCIRAPTOR
        type: velocity
        dx: {min: -10.0, max: 5.0}
        dy: {min:  5.0, max: 10.0}";

        // when
        let mut result = sut.parse_velocity_gen(&yaml(input));

        // then
        let velocity = result.generate();
        assert!(velocity.dx >= -10.0);
        assert!(velocity.dx <= 5.0);
        assert!(velocity.dy >= 5.0);
        assert!(velocity.dy <= 10.0);
    }

    #[test]
    fn loader_parse_rotation_gen() {
        // given
        let sut = Loader::new();
        let input = "
        name: ROLY POLY
        type: rotation
        min: 90.0
        max: 180.0";

        // when
        let mut result = sut.parse_rotation_gen(&yaml(input));

        // then
        let rotation = result.generate();
        assert!(rotation >= PI / 2.0);
        assert!(rotation <= PI);
    }
}