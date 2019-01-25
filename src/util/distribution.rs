use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use geometry::types::Point;
use geometry::types::Vector;
use geometry::util::Transformation;
use physics::types::Mass;
use util::distribution::Error::*;
use util::gens::*;

// Question: If I clone the gens, do they produce the same sequence?

// should define error type for useful feedback

// need to give back errors instead of unwrapping

// need to provide default values

// Error /////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub enum Error {
    MissingKey(String),
    ExpectedType(String),
    UnknownReference(String),
    InvalidValue(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MissingKey(key) => write!(f, "Missing required key: {}.", key),
            ExpectedType(which) => write!(f, "Expected type {}", which),
            UnknownReference(name) => write!(f, "Unknown reference: {}", name),
            InvalidValue(which) => write!(f, "Invalid value: {}", which),
        }
    }
}

// Loader ////////////////////////////////////////////////////////////////////

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

    // TODO: error handling here
    fn docs(path: &str) -> Vec<Yaml> {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        YamlLoader::load_from_str(&contents).unwrap()
    }

    /// Attempts to extract the value for the given key.
    fn get_value<'a>(&self, object: &'a Yaml, key: &str) -> Result<&'a Yaml, Error> {
        let value = &object[key];
        if value.is_badvalue() {
            Err(MissingKey(String::from(key)))
        } else {
            Ok(value)
        }
    }

    /// Attempts to get the integer number at the given key for the given object.
    fn get_int(&self, object: &Yaml, key: &str) -> Result<i32, Error> {
        let value = self.get_value(object, key)?;
        match value.as_i64() {
            Some(result) => Ok(result as i32),
            None => Err(ExpectedType(key.to_owned() + ": Integer")),
        }
    }

    /// Returns either the integer number at the given key for the given object, or the
    /// default value provide if they key is not found.
    fn get_int_or(&self, object: &Yaml, key: &str, default: i32) -> Result<i32, Error> {
        let value = match self.get_value(object, key) {
            Ok(value) => value,
            Err(_) => return Ok(default),
        };
        match value.as_i64() {
            Some(result) => Ok(result as i32),
            None => Err(ExpectedType(key.to_owned() + ": Integer")),
        }
    }

    /// Attempts to get the real number at the given key for the given object.
    fn get_real(&self, object: &Yaml, key: &str) -> Result<f32, Error> {
        let value = self.get_value(object, key)?;
        match value.as_f64() {
            Some(result) => Ok(result as f32),
            None => Err(ExpectedType(key.to_owned() + ": Real")),
        }
    }

    /// Returns either the real number at the given key for the given object, or the
    /// default value provide if they key is not found.
    fn get_real_or(&self, object: &Yaml, key: &str, default: f32) -> Result<f32, Error> {
        let value = match self.get_value(object, key) {
            Ok(value) => value,
            Err(_) => return Ok(default),
        };
        match value.as_f64() {
            Some(result) => Ok(result as f32),
            None => Err(ExpectedType(key.to_owned() + ": Real")),
        }
    }

    /// Attempts to get the string at the given key for the given object.
    fn get_string(&self, object: &Yaml, key: &str) -> Result<String, Error> {
        let value = self.get_value(object, key)?;
        match value.as_str() {
            Some(result) => Ok(result.to_owned()),
            None => Err(ExpectedType(key.to_owned() + ": String")),
        }
    }

    /// Attempts to get the vector at the given key for the given object.
    fn get_vec<'a>(&self, object: &'a Yaml, key: &str) -> Result<&'a Vec<Yaml>, Error> {
        let value = self.get_value(object, key)?;
        match value.as_vec() {
            Some(result) => Ok(result),
            None => Err(ExpectedType(key.to_owned() + ": Array")),
        }
    }

    // need to be able to check if accessing a key produces a bad value.
    // then need to check if casting to type fails.
    pub fn load(&mut self, path: &str) -> Result<(), Error> {
        // TODO: propagate errors here
        let docs = Loader::docs(path);
        let doc = &docs[0];

        let gens = self.get_vec(doc, "gens")?;
        self.parse_gens(gens)?;

        let bodies = self.get_vec(doc, "bodies")?;
        self.parse_bodies(bodies)?;

        // TODO: Check this should only have one element
        let root_idx = self.parse_system(doc)?;
        Ok(())
    }

    // Gen Parsing ///////////////////////////////////////////////////////////

    /// Parses each generate description in the given list and stores them
    /// in the corresponding hash map of self.
    fn parse_gens(&mut self, gens: &Vec<Yaml>) -> Result<(), Error> {
        for gen in gens {
            let name = self.get_string(gen, "name")?;
            let gen_type = self.get_string(gen, "type")?;

            match gen_type.as_str() {
                "mass" => {
                    let mass_gen = self.parse_mass_gen(gen)?;
                    self.mass_gens.insert(name, mass_gen);
                },
                "translation" => {
                    let translation_gen = self.parse_translation_gen(gen)?;
                    self.translation_gens.insert(name, translation_gen);
                },
                "velocity" => {
                    let velocity_gen = self.parse_velocity_gen(gen)?;
                    self.velocity_gens.insert(name, velocity_gen);
                },
                "rotation" => {
                    let rotation_gen = self.parse_rotation_gen(gen)?;
                    self.rotation_gens.insert(name, rotation_gen);
                },
                _ => return Err(InvalidValue(gen_type)),
            };
        }
        Ok(())
    }

    /// Parses the mass generator description.
    fn parse_mass_gen(&self, gen: &Yaml) -> Result<MassGen, Error> {
        let min = self.get_real(gen, "min")?;
        let max = self.get_real(gen, "max")?;
        Ok(MassGen::new(min, max))
    }

    /// Parses the translation generator description.
    fn parse_translation_gen(&self, gen: &Yaml) -> Result<TranslationGen, Error> {
        let x = self.get_value(gen, "x")?;
        let y = self.get_value(gen, "y")?;
        let x_min = self.get_real(x, "min")?;
        let x_max = self.get_real(x, "max")?;
        let y_min = self.get_real(y, "min")?;
        let y_max = self.get_real(y, "max")?;
        Ok(TranslationGen::new(x_min, x_max, y_min, y_max))
    }

    /// Parses the velocity generator description.
    fn parse_velocity_gen(&self, gen: &Yaml) -> Result<VelocityGen, Error> {
        let dx = self.get_value(gen, "dx")?;
        let dy = self.get_value(gen, "dy")?;
        let dx_min = self.get_real(dx, "min")?;
        let dx_max = self.get_real(dx, "max")?;
        let dy_min = self.get_real(dy, "min")?;
        let dy_max = self.get_real(dy, "max")?;
        Ok(VelocityGen::new(dx_min, dx_max, dy_min, dy_max))
    }

    /// Parses the rotation generator description.
    fn parse_rotation_gen(&self, gen: &Yaml) -> Result<RotationGen, Error> {
        let min = self.get_real(gen, "min")?;
        let max = self.get_real(gen, "max")?;
        Ok(RotationGen::new_degrees(min, max))
    }

    // Property Parsing //////////////////////////////////////////////////////

    // TODO: Refactor duplicate logic
    // TODO: Could check for key existence?

    /// Returns the named mass gen if it exists, else creates one from concrete values.
    fn parse_mass(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Mass>>, Error> {
        // check for gen reference
        match self.get_string(object, "m") {
            Ok(gen_name) => {
                // look it up
                return match self.mass_gens.get(gen_name.as_str()) {
                    None => Err(UnknownReference(gen_name)),
                    Some(gen) => Ok(Box::new(gen.clone())),
                }
            },
            _ => (),
        }

        // get concrete value
        let mass = self.get_real(object, "m")?;
        Ok(Box::new(Repeater::new(Mass::from(mass))))
    }

    /// Returns the named translation gen if it exists, else creates one from concrete values,
    /// else provides default value of (0.0, 0.0).
    fn parse_translation(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Point>>, Error> {
        // check for gen reference
        match self.get_string(object, "t") {
            Ok(gen_name) => {
                // look it up
                return match self.translation_gens.get(gen_name.as_str()) {
                    None => Err(UnknownReference(gen_name)),
                    Some(gen) => Ok(Box::new(gen.clone())),
                }
            },
            _ => (),
        }

        // get concrete values
        let translation = match self.get_value(object, "t") {
            Err(error) => match error {
                // provide default
                MissingKey(_) => Point::zero(),
                _ => return Err(error),
            },
            Ok(value) => {
                let x = self.get_real(value, "x")?;
                let y = self.get_real(value, "y")?;
                Point::new(x, y)
            },
        };

        Ok(Box::new(Repeater::new(translation)))
    }

    /// Returns the named velocity gen if it exists, else creates one from concrete values,
    /// else provides default value of (0.0, 0.0).
    fn parse_velocity(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Vector>>, Error> {
        // check for gen reference
        match self.get_string(object, "v") {
            Ok(gen_name) => {
                // look it up
                return match self.velocity_gens.get(gen_name.as_str()) {
                    None => Err(UnknownReference(gen_name)),
                    Some(gen) => Ok(Box::new(gen.clone())),
                }
            },
            _ => (),
        }

        // get concrete values
        let velocity = match self.get_value(object, "v") {
            Err(error) => match error {
                // provide default
                MissingKey(_) => Vector::zero(),
                _ => return Err(error),
            },
            Ok(value) => {
                let dx = self.get_real(value, "dx")?;
                let dy = self.get_real(value, "dy")?;
                Vector::new(dx, dy)
            },
        };

        Ok(Box::new(Repeater::new(velocity)))
    }

    /// Returns the named rotation gen if it exists, else creates one from concrete values
    /// else provides default value of 0.0.
    fn parse_rotation(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=f32>>, Error> {
        // check for gen reference
        match self.get_string(object, "r") {
            Ok(gen_name) => {
                // look it up
                return match self.rotation_gens.get(gen_name.as_str()) {
                    None => Err(UnknownReference(gen_name)),
                    Some(gen) => Ok(Box::new(gen.clone())),
                }
            },
            _ => (),
        }

        // get concrete values
        let rotation = self.get_real_or(object, "r", 0.0)?;
        Ok(Box::new(Repeater::new(rotation)))
    }

    // Body Parsing //////////////////////////////////////////////////////////

    /// Parses each body description in the given list and stores them in
    /// the bodies hash map of self.
    fn parse_bodies(&mut self, bodies: &Vec<Yaml>) -> Result<(), Error> {
        for body in bodies {
            let (name, nodes) = self.parse_body(body)?;
            self.bodies.insert(name, nodes);
        };
        Ok(())
    }

    /// Parses the given body description.
    fn parse_body(&self, body: &Yaml) -> Result<(String, Vec<Node>), Error> {
        let name = self.get_string(body, "name")?;
        let num = self.get_int_or(body, "num", 1)?;

        if num < 1 {
            return Err(InvalidValue(String::from("num must be greater than 1")));
        }

        let mut nodes: Vec<Node> = vec![];
        let mut mass = self.parse_mass(body)?;
        let mut trans = self.parse_translation(body)?;
        let mut vel = self.parse_velocity(body)?;
        let mut rot = self.parse_rotation(body)?;

        let mut nodes: Vec<Node> = Vec::new();

        for _ in 1..=num {
            let tvr = TVR(trans.generate(), vel.generate(), rot.generate());
            let node = Node::Body(tvr, mass.generate());
            nodes.push(node);
        }

        Ok((String::from(name), nodes))
    }

    // System Parsing ////////////////////////////////////////////////////////////

    /// Parses the given system description.
    fn parse_system(&mut self, system: &Yaml) -> Result<Vec<Index>, Error> {
        // check for reference to bodies
        match self.get_string(system, "name") {
            Ok(name) => {
                // look it up
                return match self.bodies.remove(name.as_str()) {
                    None => Err(UnknownReference(name)),
                    Some(bodies) => {
                        let mut indices = vec![];
                        for body in bodies {
                            indices.push(self.tree.add_node(body));
                        }
                        return Ok(indices)
                    },
                }
            },
            Err(_) => {},
        }

        // transformation for the system
        let tvr: TVR;
        {
            let t = self.parse_translation(system)?.generate();
            let v = self.parse_velocity(system)?.generate();
            let r = self.parse_rotation(system)?.generate();
            tvr = TVR(t, v, r);
        }

        // parse the subsystems
        let mut subsystems: Vec<Index> = vec![];
        for subsystem in self.get_vec(system, "systems")? {
            let mut indices = self.parse_system(subsystem)?;
            subsystems.append(&mut indices);
        }

        // finally build the node & return its index
        let idx = self.tree.add_node(Node::System(tvr, subsystems));
        Ok(vec![idx])
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

    use super::Error::*;

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

    // GET VALUES ////////////////////////////////////////////////////////////

    #[test]
    fn loader_get_value() {
        // given
        let sut = Loader::new();
        let object = yaml("key: value");

        // when
        let result = sut.get_value(&object, "key");

        // then
        assert!(result.is_ok());
    }

    #[test]
    fn loader_get_value_missing_key() {
        // given
        let sut = Loader::new();
        let object = yaml("bar: value");

        // when
        let result = sut.get_value(&object, "foo");

        // then
        assert_eq!(Err(MissingKey(String::from("foo"))), result);
    }

    #[test]
    fn loader_get_int() {
        // given
        let sut = Loader::new();
        let object = yaml("num: 42");

        // when
        let result = sut.get_int(&object, "num").unwrap();

        // then
        assert_eq!(42, result);
    }

    #[test]
    fn loader_get_int_invalid_type() {
        // given
        let sut = Loader::new();
        let object = yaml("num: 42.3");

        // when
        let result = sut.get_int(&object, "num");

        // then
        assert_eq!(Err(ExpectedType(String::from("num: Integer"))), result);
    }

    #[test]
    fn loader_get_int_or() {
        // given
        let sut = Loader::new();
        let object = yaml("foo: 23");

        // when
        let result = sut.get_int_or(&object, "bar", 42).unwrap();

        // then
        assert_eq!(42, result);
    }

    #[test]
    fn loader_get_real() {
        // given
        let sut = Loader::new();
        let object = yaml("num: 3.14");

        // when
        let result = sut.get_real(&object, "num").unwrap();

        // then
        assert_eq!(3.14, result);
    }

    #[test]
    fn loader_get_real_invalid_type() {
        // given
        let sut = Loader::new();
        let object = yaml("num: 42");

        // when
        let result = sut.get_real(&object, "num");

        // then
        assert_eq!(Err(ExpectedType(String::from("num: Real"))), result);
    }

    #[test]
    fn loader_get_real_or() {
        // given
        let sut = Loader::new();
        let object = yaml("foo: 2.17");

        // when
        let result = sut.get_real_or(&object, "bar", 3.14).unwrap();

        // then
        assert_eq!(3.14, result);
    }

    #[test]
    fn loader_get_string() {
        // given
        let sut = Loader::new();
        let object = yaml("name: bob");

        // when
        let result = sut.get_string(&object, "name").unwrap();

        // then
        assert_eq!(String::from("bob"), result);
    }

    #[test]
    fn loader_get_string_invalid_type() {
        // given
        let sut = Loader::new();
        let object = yaml("name: 42");

        // when
        let result = sut.get_string(&object, "name");

        // then
        assert_eq!(Err(ExpectedType(String::from("name: String"))), result);
    }

    #[test]
    fn loader_get_vec() {
        // given
        let sut = Loader::new();
        let input = "
        nums:
          - 1
          - 2";

        let object = yaml(input);

        // when
        let result = sut.get_vec(&object, "nums").unwrap();

        // then
        assert_eq!(2, result.len());
    }

    #[test]
    fn loader_get_vec_invalid_type() {
        // given
        let sut = Loader::new();
        let object = yaml("nums: 42");

        // when
        let result = sut.get_vec(&object, "nums");

        // then
        assert_eq!(Err(ExpectedType(String::from("nums: Array"))), result);
    }

    // Gen Parsing ///////////////////////////////////////////////////////////

    #[test]
    fn loader_parse_gens() {
        // given
        let mut sut = Loader::new();
        let input = "
        gens:
          -
            name: m
            type: mass
            min: 0.1
            max: 0.3
          -
            name: t
            type: translation
            x: {min: -10.0, max: 10.0}
            y: {min: -10.0, max: 10.0}
          -
            name: v
            type: velocity
            dx: {min: -10.0, max: 10.0}
            dy: {min: -10.0, max: 10.0}
          -
            name: r
            type: rotation
            min: 0.1
            max: 0.3";

        let object = yaml(input);
        let gens = sut.get_vec(&object, "gens").unwrap();

        // when
        let result = sut.parse_gens(gens).unwrap();

        // then
        assert_eq!(1, sut.mass_gens.len());
        assert!(sut.mass_gens.get("m").is_some());

        assert_eq!(1, sut.translation_gens.len());
        assert!(sut.translation_gens.get("t").is_some());

        assert_eq!(1, sut.velocity_gens.len());
        assert!(sut.velocity_gens.get("v").is_some());

        assert_eq!(1, sut.rotation_gens.len());
        assert!(sut.rotation_gens.get("r").is_some());
    }

    #[test]
    fn loader_parse_gens_invalid_value() {
        // given
        let mut sut = Loader::new();
        let input = "
        gens:
          -
            name: m
            type: unexpected
            min: 0.1
            max: 0.3";

        let object = yaml(input);
        let gens = sut.get_vec(&object, "gens").unwrap();

        // when
        let result = sut.parse_gens(gens);

        // then
        assert_eq!(Err(InvalidValue(String::from("unexpected"))), result);
    }

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
        let mut result = sut.parse_mass_gen(&yaml(input)).unwrap();

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
        let mut result = sut.parse_translation_gen(&yaml(input)).unwrap();

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
        let mut result = sut.parse_velocity_gen(&yaml(input)).unwrap();

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
        let mut result = sut.parse_rotation_gen(&yaml(input)).unwrap();

        // then
        let rotation = result.generate();
        assert!(rotation >= PI / 2.0);
        assert!(rotation <= PI);
    }

    // Property Parsing //////////////////////////////////////////////////////
    

    // Body Parsing //////////////////////////////////////////////////////////
}