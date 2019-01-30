use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::result;
use std::slice::Iter;

use yaml_rust::ScanError;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use crate::geometry::types::Point;
use crate::geometry::types::Vector;
use crate::geometry::util::Transformation;
use crate::physics::types::Body;
use crate::physics::types::Mass;
use crate::util::distribution::Error::*;
use crate::util::gens::*;

// Question: If I clone the gens, do they produce the same sequence?

// Loader ////////////////////////////////////////////////////////////////////
//
// A Loader is responsible for parsing a system configuration file and
// creating the Body objects described within. The input is a Yaml file that
// defines 1) various types of property generators, 2) how to use these
// generators to create body objects, and 3) the kinetic and spacial between
// between of bodies as a hierarchy of systems.

#[derive(Default)]
pub struct Loader {
    tree: DistributionTree,
    bodies: HashMap<String, Vec<Node>>,
    mass_gens: HashMap<String, MassGen>,
    translation_gens: HashMap<String, TranslationGen>,
    velocity_gens: HashMap<String, VelocityGen>,
    rotation_gens: HashMap<String, RotationGen>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader::default()
    }

    pub fn load_from_path(&mut self, path: &str) -> Result<Vec<Body>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.load(contents)
    }

    pub fn load(&mut self, config: String) -> Result<Vec<Body>> {
        let docs = YamlLoader::load_from_str(&config)?;
        let doc = &docs[0];

        // parse gens if defined
        match self.get_vec(doc, "gens") {
            Ok(gens) => self.parse_gens(gens)?,
            Err(error) => match error {
                Error::MissingKey(_) => (),
                _ => return Err(error),
            },
        };

        let bodies = self.get_vec(doc, "bodies")?;
        self.parse_bodies(bodies)?;

        self.parse_system(doc)?;
        Ok(self.tree.bodies())
    }

    // Accessors /////////////////////////////////////////////////////////////

    /// Attempts to extract the value for the given key.
    fn get_value<'a>(&self, object: &'a Yaml, key: &str) -> Result<&'a Yaml> {
        let value = &object[key];
        if value.is_badvalue() {
            Err(MissingKey(String::from(key)))
        } else {
            Ok(value)
        }
    }

    /// Returns either the integer number at the given key for the given object, or the
    /// default value provide if they key is not found.
    fn get_int_or(&self, object: &Yaml, key: &str, default: i32) -> Result<i32> {
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
    fn get_real(&self, object: &Yaml, key: &str) -> Result<f32> {
        let value = self.get_value(object, key)?;
        match value.as_f64() {
            Some(result) => Ok(result as f32),
            None => Err(ExpectedType(key.to_owned() + ": Real")),
        }
    }

    /// Returns either the real number at the given key for the given object, or the
    /// default value provide if they key is not found.
    fn get_real_or(&self, object: &Yaml, key: &str, default: f32) -> Result<f32> {
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
    fn get_string(&self, object: &Yaml, key: &str) -> Result<String> {
        let value = self.get_value(object, key)?;
        match value.as_str() {
            Some(result) => Ok(result.to_owned()),
            None => Err(ExpectedType(key.to_owned() + ": String")),
        }
    }

    /// Attempts to get the vector at the given key for the given object.
    fn get_vec<'a>(&self, object: &'a Yaml, key: &str) -> Result<&'a Vec<Yaml>> {
        let value = self.get_value(object, key)?;
        match value.as_vec() {
            Some(result) => Ok(result),
            None => Err(ExpectedType(key.to_owned() + ": Array")),
        }
    }

    // Gen Parsing ///////////////////////////////////////////////////////////

    /// Parses each generate description in the given list and stores them
    /// in the corresponding hash map of self.
    fn parse_gens(&mut self, gens: &[Yaml]) -> Result<()> {
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
    fn parse_mass_gen(&self, gen: &Yaml) -> Result<MassGen> {
        let min = self.get_real(gen, "min")?;
        let max = self.get_real(gen, "max")?;
        Ok(MassGen::new(min, max))
    }

    /// Parses the translation generator description.
    fn parse_translation_gen(&self, gen: &Yaml) -> Result<TranslationGen> {
        let x = self.get_value(gen, "x")?;
        let y = self.get_value(gen, "y")?;
        let x_min = self.get_real(x, "min")?;
        let x_max = self.get_real(x, "max")?;
        let y_min = self.get_real(y, "min")?;
        let y_max = self.get_real(y, "max")?;
        Ok(TranslationGen::new(x_min, x_max, y_min, y_max))
    }

    /// Parses the velocity generator description.
    fn parse_velocity_gen(&self, gen: &Yaml) -> Result<VelocityGen> {
        let dx = self.get_value(gen, "dx")?;
        let dy = self.get_value(gen, "dy")?;
        let dx_min = self.get_real(dx, "min")?;
        let dx_max = self.get_real(dx, "max")?;
        let dy_min = self.get_real(dy, "min")?;
        let dy_max = self.get_real(dy, "max")?;
        Ok(VelocityGen::new(dx_min, dx_max, dy_min, dy_max))
    }

    /// Parses the rotation generator description.
    fn parse_rotation_gen(&self, gen: &Yaml) -> Result<RotationGen> {
        let min = self.get_real(gen, "min")?;
        let max = self.get_real(gen, "max")?;
        Ok(RotationGen::new_degrees(min, max))
    }

    // Property Parsing //////////////////////////////////////////////////////

    // TODO: Refactor duplicate logic
    // TODO: Could check for key existence?

    /// Returns the named mass gen if it exists, else creates one from concrete values.
    fn parse_mass(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Mass>>> {
        // check for gen reference
        if let Ok(gen_name) = self.get_string(object, "m") {
            // look it up
            return match self.mass_gens.get(gen_name.as_str()) {
                None => Err(UnknownReference(gen_name)),
                Some(gen) => Ok(Box::new(gen.clone())),
            }
        }

        // get concrete value
        let mass = self.get_real(object, "m")?;
        Ok(Box::new(Repeater::new(Mass::from(mass))))
    }

    /// Returns the named translation gen if it exists, else creates one from concrete values,
    /// else provides default value of (0.0, 0.0).
    fn parse_translation(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Point>>> {
        // check for gen reference
        if let Ok(gen_name) = self.get_string(object, "t") {
            // look it up
            return match self.translation_gens.get(gen_name.as_str()) {
                None => Err(UnknownReference(gen_name)),
                Some(gen) => Ok(Box::new(gen.clone())),
            }
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
    fn parse_velocity(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=Vector>>> {
        // check for gen reference
        if let Ok(gen_name) = self.get_string(object, "v") {
            // look it up
            return match self.velocity_gens.get(gen_name.as_str()) {
                None => Err(UnknownReference(gen_name)),
                Some(gen) => Ok(Box::new(gen.clone())),
            }
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
    fn parse_rotation(&self, object: &Yaml) -> Result<Box<dyn Generator<Output=f32>>> {
        // check for gen reference
        if let Ok(gen_name) = self.get_string(object, "r") {
            // look it up
            return match self.rotation_gens.get(gen_name.as_str()) {
                None => Err(UnknownReference(gen_name)),
                Some(gen) => Ok(Box::new(gen.clone())),
            }
        }

        // get concrete values
        let rotation = self.get_real_or(object, "r", 0.0)?;
        Ok(Box::new(Repeater::new(rotation.to_radians())))
    }

    // Body Parsing //////////////////////////////////////////////////////////

    /// Parses the given body description.
    fn parse_body(&self, body: &Yaml) -> Result<(String, Vec<Node>)> {
        let name = self.get_string(body, "name")?;
        let num = self.get_int_or(body, "num", 1)?;

        if num < 1 {
            return Err(InvalidValue(String::from("num must be greater than 1")));
        }

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

        Ok((name, nodes))
    }

    /// Parses each body description in the given list and stores them in
    /// the bodies hash map of self.
    fn parse_bodies(&mut self, bodies: &[Yaml]) -> Result<()> {
        for body in bodies {
            let (name, nodes) = self.parse_body(body)?;
            self.bodies.insert(name, nodes);
        };
        Ok(())
    }

    // System Parsing ////////////////////////////////////////////////////////////

    /// Parses the given system description.
    fn parse_system(&mut self, system: &Yaml) -> Result<Vec<Index>> {
        // check for reference to bodies
        if let Ok(name) = self.get_string(system, "name") {
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

// TVR ///////////////////////////////////////////////////////////////////////
//
// Small helper struct to contain Translation, Velocity, and Rotation data.

#[derive(Clone, PartialEq, Debug)]
struct TVR(Point, Vector, f32);

impl Default for TVR {
    fn default() -> Self {
        TVR(Point::zero(), Vector::zero(), 0.0)
    }
}

// Node //////////////////////////////////////////////////////////////////////
//
// Each node in the tree has the three tvr properties. System nodes contain a
//  list of the indices of subsystem nodes, whereas body nodes have a mass.

type Index = usize;

#[derive(Clone, PartialEq, Debug)]
enum Node {
    System(TVR, Vec<Index>),
    Body(TVR, Mass),
}

// DistributionTree //////////////////////////////////////////////////////////
//
// This tree is used to relate together nodes derived from a system
// configuration file. Once the nodes are related, one can traverse the tree
// to create the Body objects given the various data stored at each node.

#[derive(Default, Debug)]
struct DistributionTree {
    nodes: Vec<Node>
}

impl DistributionTree {
    /// Adds the given node to the tree and return its index.
    fn add_node(&mut self, node: Node) -> Index {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Traverses the tree and returns the bodies derived from it.
    fn bodies(&mut self) -> Vec<Body> {
        // start at the root node
        let start = vec![self.nodes.len() - 1];
        // stores the children indices and tvr data for visited nodes.
        let mut stack: Vec<(Iter<Index>, TVR)> = vec![];
        let mut bodies: Vec<Body> = vec![];

        // As we descend the tree, we must accumulate the tvr data.
        // First we accumulate rotation, so we can rotate the new position
        // and velocity. These new values are added to the previous values,
        // so that they are accumulated too.
        let merge = |prev: &TVR, curr: &TVR| -> TVR {
            let rotation = prev.2 + curr.2;
            let transform = Transformation::rotation(rotation);
            let (position, velocity) = (&transform * curr.0.clone(), &transform * curr.1.clone());
            let (position, velocity) = (prev.0.clone() + position, prev.1.clone() + velocity);
            TVR(position, velocity, rotation)
        };

        stack.push((start.iter(), TVR::default()));

        // there are potentially systems to inspect
        while let Some((mut systems, prev_tvr)) = stack.pop() {
            // there is a system to inspect
            if let Some(next) = systems.next() {
                match &self.nodes[*next] {
                    // it's a body
                    Node::Body(curr_tvr, mass) => {
                        let new_tvr = merge(&prev_tvr, curr_tvr);
                        bodies.push(Body::new(mass.value(), new_tvr.0, new_tvr.1));
                        stack.push((systems, prev_tvr));
                    },
                    // it's a system
                    Node::System(curr_tvr, subsystems) => {
                        let new_tvr = merge(&prev_tvr, curr_tvr);
                        stack.push((systems, prev_tvr));
                        stack.push((subsystems.iter(), new_tvr));
                    },
                };
            };
        };

        bodies
    }
}

// Error /////////////////////////////////////////////////////////////////////

pub type Result<T> = result::Result<T, Error>;

#[derive(PartialEq, Clone, Debug)]
pub enum Error {
    MissingKey(String),
    ExpectedType(String),
    UnknownReference(String),
    InvalidValue(String),
    IOError,
    InvalidYaml,
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        IOError
    }
}

impl From<ScanError> for Error {
    fn from(_: ScanError) -> Self {
        InvalidYaml
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MissingKey(key) => write!(f, "Missing required key: {}.", key),
            ExpectedType(which) => write!(f, "Expected type {}", which),
            UnknownReference(name) => write!(f, "Unknown reference: {}", name),
            InvalidValue(which) => write!(f, "Invalid value: {}", which),
            IOError => write!(f, "IO error: could not open/read file"),
            InvalidYaml => write!(f, "Invalid yaml: could not parse file"),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use yaml_rust::Yaml;
    use yaml_rust::YamlLoader;

    use crate::geometry::types::Point;
    use crate::geometry::types::Vector;
    use crate::physics::types::Mass;
    use crate::util::distribution::Loader;
    use crate::util::distribution::Node::*;
    use crate::util::distribution::TVR;
    use crate::util::gens::Generator;
    use crate::util::gens::MassGen;
    use crate::util::gens::RotationGen;
    use crate::util::gens::TranslationGen;
    use crate::util::gens::VelocityGen;

    use super::Error::*;

    fn yaml(raw: &str) -> Yaml {
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
        let result = sut.parse_gens(gens);

        // then
        assert!(result.is_ok());

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

    #[test]
    fn loader_parse_mass_concrete_value() {
        // given
        let sut = Loader::new();
        let object = yaml("m: 1.0");

        // when
        let mut result = sut.parse_mass(&object).unwrap();

        // then
        assert_eq!(1.0, result.generate().value());
    }

    #[test]
    fn loader_parse_mass_missing_value() {
        // given
        let sut = Loader::new();
        let object = yaml("foo: bar");

        // when
        let result = sut.parse_mass(&object).err().unwrap();

        // then
        assert_eq!(MissingKey(String::from("m")), result);
    }

    #[test]
    fn loader_parse_mass_gen_reference() {
        // given
        let mut sut = Loader::new();
        sut.mass_gens.insert(String::from("massa"), MassGen::new(6.0, 6.3));
        let object = yaml("m: massa");

        // when
        let mut result = sut.parse_mass(&object).unwrap();

        // then
        let mass = result.generate().value();
        assert!(mass >= 6.0 && mass <= 6.3);
    }

    #[test]
    fn loader_parse_mass_unknown_reference() {
        // given
        let sut = Loader::new();
        let object = yaml("m: massa");

        // when
        let result = sut.parse_mass(&object).err().unwrap();

        // then
        assert_eq!(UnknownReference(String::from("massa")), result);
    }

    #[test]
    fn loader_parse_translation_concrete_value() {
        // given
        let sut = Loader::new();
        let object = yaml("t: {x: 1.2, y: 3.4}");

        // when
        let mut result = sut.parse_translation(&object).unwrap();

        // then
        assert_eq!(Point::new(1.2, 3.4), result.generate());
    }

    #[test]
    fn loader_parse_translation_default_value() {
        // given
        let sut = Loader::new();
        let object = yaml("m: 0.2");

        // when
        let mut result = sut.parse_translation(&object).unwrap();

        // then
        assert_eq!(Point::zero(), result.generate());
    }

    #[test]
    fn loader_parse_translation_gen_reference() {
        // given
        let mut sut = Loader::new();
        let gen = TranslationGen::new(1.0, 1.0, 2.0, 2.0);
        sut.translation_gens.insert(String::from("trans"), gen);
        let object = yaml("t: trans");

        // when
        let mut result = sut.parse_translation(&object).unwrap();

        // then
        assert_eq!(Point::new(1.0, 2.0), result.generate());
    }

    #[test]
    fn loader_parse_translation_unknown_reference() {
        // given
        let sut = Loader::new();
        let object = yaml("t: trans");

        // when
        let result = sut.parse_translation(&object).err().unwrap();

        // then
        assert_eq!(UnknownReference(String::from("trans")), result);
    }

    #[test]
    fn loader_parse_velocity_concrete_value() {
        // given
        let sut = Loader::new();
        let object = yaml("v: {dx: 1.2, dy: 3.4}");

        // when
        let mut result = sut.parse_velocity(&object).unwrap();

        // then
        assert_eq!(Vector::new(1.2, 3.4), result.generate());
    }

    #[test]
    fn loader_parse_velocity_default_value() {
        // given
        let sut = Loader::new();
        let object = yaml("m: 0.2");

        // when
        let mut result = sut.parse_velocity(&object).unwrap();

        // then
        assert_eq!(Vector::zero(), result.generate());
    }

    #[test]
    fn loader_parse_velocity_gen_reference() {
        // given
        let mut sut = Loader::new();
        let gen = VelocityGen::new(1.0, 1.0, 2.0, 2.0);
        sut.velocity_gens.insert(String::from("vel"), gen);
        let object = yaml("v: vel");

        // when
        let mut result = sut.parse_velocity(&object).unwrap();

        // then
        assert_eq!(Vector::new(1.0, 2.0), result.generate());
    }

    #[test]
    fn loader_parse_velocity_unknown_reference() {
        // given
        let sut = Loader::new();
        let object = yaml("v: vel");

        // when
        let result = sut.parse_velocity(&object).err().unwrap();

        // then
        assert_eq!(UnknownReference(String::from("vel")), result);
    }

    #[test]
    fn loader_parse_rotation_concrete_value() {
        // given
        let sut = Loader::new();
        let object = yaml("r: 180.0");

        // when
        let mut result = sut.parse_rotation(&object).unwrap();

        // then
        assert_eq!(PI, result.generate());
    }

    #[test]
    fn loader_parse_rotation_default_value() {
        // given
        let sut = Loader::new();
        let object = yaml("m: 0.2");

        // when
        let mut result = sut.parse_rotation(&object).unwrap();

        // then
        assert_eq!(0.0, result.generate());
    }

    #[test]
    fn loader_parse_rotation_gen_reference() {
        // given
        let mut sut = Loader::new();
        let gen = RotationGen::new_radians(1.0, 2.0);
        sut.rotation_gens.insert(String::from("rot"), gen);
        let object = yaml("r: rot");

        // when
        let mut result = sut.parse_rotation(&object).unwrap();

        // then
        let rotation = result.generate();
        assert!(rotation >= 1.0 && rotation <= 2.0);
    }

    #[test]
    fn loader_parse_rotation_unknown_reference() {
        // given
        let sut = Loader::new();
        let object = yaml("r: rot");

        // when
        let result = sut.parse_rotation(&object).err().unwrap();

        // then
        assert_eq!(UnknownReference(String::from("rot")), result);
    }

    // Body Parsing //////////////////////////////////////////////////////////

    // parse body
    #[test]
    fn loader_parse_body() {
        // given
        let sut = Loader::new();
        let input = "
        name: earth
        m: 10.0";

        let object = yaml(input);

        // when
        let result = sut.parse_body(&object).unwrap();

        // then
        assert_eq!(String::from("earth"), result.0);
        assert_eq!(1, result.1.len());

        let expected = Body(TVR(Point::zero(), Vector::zero(), 0.0), Mass::new(10.0));
        let actual = result.1[0].clone();
        assert_eq!(expected, actual);
    }

    #[test]
    fn loader_parse_body_num() {
        // given
        let sut = Loader::new();
        let input = "{name: earth, num: 3, m: 10.0}";

        let object = yaml(input);

        // when
        let result = sut.parse_body(&object).unwrap();

        // then
        assert_eq!(String::from("earth"), result.0);
        assert_eq!(3, result.1.len());

        let expected = Body(TVR(Point::zero(), Vector::zero(), 0.0), Mass::new(10.0));
        assert_eq!(expected, result.1[0].clone());
        assert_eq!(expected, result.1[1].clone());
        assert_eq!(expected, result.1[2].clone());
    }

    #[test]
    fn loader_parse_body_invalid_num() {
        // given
        let sut = Loader::new();
        let input = "{name: earth, num: -4, m: 10.0}";

        let object = yaml(input);

        // when
        let result = sut.parse_body(&object).err().unwrap();

        // then
        assert_eq!(InvalidValue(String::from("num must be greater than 1")), result);
    }

    #[test]
    fn loader_parse_bodies() {
        // given
        let mut sut = Loader::new();
        let input = "
        bodies: [{name: earth, m: 10.0}, {name: moon, m: 1.0}]";

        let object = yaml(input);
        let bodies = sut.get_vec(&object, "bodies").unwrap();

        // when
        let result = sut.parse_bodies(bodies);

        // then
        assert_eq!(Ok(()), result);
        assert_eq!(2, sut.bodies.len());

        let earths = sut.bodies.remove("earth").unwrap();
        let expected = Body(TVR::default(), Mass::new(10.0));
        assert_eq!(1, earths.len());
        assert_eq!(expected, earths[0]);

        let moons = sut.bodies.remove("moon").unwrap();
        let expected = Body(TVR::default(), Mass::new(1.0));
        assert_eq!(1, moons.len());
        assert_eq!(expected, moons[0]);
    }

    // System Parsing ////////////////////////////////////////////////////////

    #[test]
    fn loader_parse_system_body_reference() {
        // given
        let mut sut = Loader::new();
        let sun = Body(TVR::default(), Mass::new(100.0));
        sut.bodies.insert(String::from("sun"), vec![sun.clone()]);

        let object = yaml("name: sun");

        // when
        let root = sut.parse_system(&object).unwrap();

        // then
        // root node
        assert_eq!(1, root.len());
        // is only node, so first index
        assert_eq!(0, root[0]);
        // only one node in the tree
        assert_eq!(1, sut.tree.nodes.len());
        // this node is the sun
        assert_eq!(sun, sut.tree.nodes[0]);
    }

    #[test]
    fn loader_parse_system_body_unknown_reference() {
        // given
        let mut sut = Loader::new();
        let object = yaml("name: sun");

        // when
        let result = sut.parse_system(&object).err().unwrap();

        // then
        assert_eq!(UnknownReference(String::from("sun")), result);
    }

    #[test]
    fn loader_parse_system_with_subsystems() {
        // given
        let mut sut = Loader::new();
        let sun = Body(TVR::default(), Mass::new(100.0));
        let earth = Body(TVR::default(), Mass::new(10.0));
        let moon = Body(TVR::default(), Mass::new(1.0));
        sut.bodies.insert(String::from("sun"), vec![sun.clone()]);
        sut.bodies.insert(String::from("earth"), vec![earth.clone()]);
        sut.bodies.insert(String::from("moon"), vec![moon.clone()]);

        let input = "
        systems:
          - name: sun
          - systems:
            - name: earth
            - name: moon
        ";

        let object = yaml(input);

        // when
        let root = sut.parse_system(&object).unwrap();

        // then
        // 5 nodes in total: 3 for sun, earth, moon. 1 for earth system. 1 for solar system
        assert_eq!(5, sut.tree.nodes.len());
        // root node returned
        assert_eq!(1, root.len());
        // root is last node added, so index is 4
        assert_eq!(4, root[0]);
        // sun added first
        assert_eq!(sun, sut.tree.nodes[0]);
        // then earth
        assert_eq!(earth, sut.tree.nodes[1]);
        // then moon
        assert_eq!(moon, sut.tree.nodes[2]);
        // then earth system
        assert_eq!(System(TVR::default(), vec![1, 2]), sut.tree.nodes[3]);
        // then solar system (root)
        assert_eq!(System(TVR::default(), vec![0, 3]), sut.tree.nodes[4]);
    }

    // Load //////////////////////////////////////////////////////////////////

    #[test]
    fn loader_load() {
        // given
        let mut sut = Loader::new();
        let input = "
        gens:
          -
            name: p_mass
            type: mass
            min: 0.1
            max: 0.1
          -
            name: p_trans
            type: translation
            x: {min: 2.0, max: 2.0}
            y: {min: 3.0, max: 3.0}
          -
            name: p_vel
            type: velocity
            dx: {min: 4.0, max: 4.0}
            dy: {min: 5.0, max: 5.0}
          -
            name: p_rot
            type: rotation
            min: 0.0
            max: 0.0

        bodies:
          -
            name: sun
            m: 100.0
          -
            name: planets
            num: 7

            m: p_mass
            t: p_trans
            v: p_vel
            r: p_rot
          -
            name: earth
            m: 20.0
          -
            name: moon
            m: 3.0
            t: {x: 10.0, y: 0.0}
            v: {dx: 0.0, dy: 2.0}

        systems:
          - name: sun
          - name: planets
          - # earth system
            t: p_trans
            v: p_vel
            r: p_rot
            systems:
              - name: earth
              - name: moon
        ";

        // when
        let result = sut.load(String::from(input));

        // then
        assert!(result.is_ok());

        // these are the generated values
        let tvr = TVR(Point::new(2.0, 3.0), Vector::new(4.0, 5.0), 0.0);

        // there should be 10 body nodes & 2 system nodes
        assert_eq!(12, sut.tree.nodes.len());

        // first is the sun
        let sun = Body(TVR::default(), Mass::from(100.0));
        assert_eq!(sun, sut.tree.nodes[0]);

        // then 7 planets
        let planet = Body(tvr.clone(), Mass::from(0.1));
        assert_eq!(planet, sut.tree.nodes[1]);
        assert_eq!(planet, sut.tree.nodes[2]);
        assert_eq!(planet, sut.tree.nodes[3]);
        assert_eq!(planet, sut.tree.nodes[4]);
        assert_eq!(planet, sut.tree.nodes[5]);
        assert_eq!(planet, sut.tree.nodes[6]);
        assert_eq!(planet, sut.tree.nodes[7]);

        // next is earth
        let earth = Body(TVR::default(), Mass::from(20.0));
        assert_eq!(earth, sut.tree.nodes[8]);

        // then moon
        let moon_tvr = TVR(Point::new(10.0, 0.0), Vector::new(0.0, 2.0), 0.0);
        let earth = Body(moon_tvr, Mass::from(3.0));
        assert_eq!(earth, sut.tree.nodes[9]);

        // then earth system
        let earth_system = System(tvr.clone(), vec![8, 9]);
        assert_eq!(earth_system, sut.tree.nodes[10]);

        // lastly solar system (root)
        let solar_system = System(TVR::default(), vec![0, 1, 2, 3, 4, 5, 6, 7, 10]);
        assert_eq!(solar_system, sut.tree.nodes[11]);
    }

    #[test]
    fn loader_load_bodies() {
        // given
        let mut sut = Loader::new();
        let input = "
        bodies:
          - {name: sun, m: 100.0}
          - {name: earth, m: 10.0}
          - {name: moon, m: 1.0, t: {x: 0.0, y: 10.0}, v: {dx: 2.0, dy: 0.0}}

        systems:
          - {name: sun}
          -
            t: {x: 100.0, y: 0.0}
            v: {dx: 0.0, dy: 5.0}
            systems:
              - name: earth
              - name: moon
        ";

        // when
        let result = sut.load(String::from(input)).unwrap();

        // then
        assert_eq!(3, result.len());

        // first is the sun
        let sun = &result[0];
        assert_eq!(100.0, sun.mass.value());
        assert_eq!(Point::zero(), sun.position);
        assert_eq!(Vector::zero(), sun.velocity);

        // then the earth
        let earth = &result[1];
        assert_eq!(10.0, earth.mass.value());
        assert_eq!(Point::new(100.0, 0.0), earth.position);
        assert_eq!(Vector::new(0.0, 5.0), earth.velocity);

        // finally the moon
        let moon = &result[2];
        assert_eq!(1.0, moon.mass.value());
        assert_eq!(Point::new(100.0, 10.0), moon.position);
        assert_eq!(Vector::new(2.0, 5.0), moon.velocity);
    }

    #[test]
    fn loader_load_no_gens() {
        // given
        let mut sut = Loader::new();
        let input = "
        bodies: [{name: sun, m: 100.0}]
        systems: [{name: sun}]";

        // when
        let result = sut.load(String::from(input));

        // then
        assert!(result.is_ok());
    }

    #[test]
    fn loader_load_no_bodies() {
        // given
        let mut sut = Loader::new();
        let input = "systems: [{name: sun}]";

        // when
        let result = sut.load(String::from(input)).err().unwrap();

        // then
        assert_eq!(MissingKey(String::from("bodies")), result);
    }

    #[test]
    fn loader_load_no_systems() {
        // given
        let mut sut = Loader::new();
        let input = "bodies: [{name: sun, m: 100.0}]";

        // when
        let result = sut.load(String::from(input)).err().unwrap();

        // then
        assert_eq!(MissingKey(String::from("systems")), result);
    }

    #[test]
    fn loader_load_invalid_yaml() {
        // given
        let mut sut = Loader::new();
        let input = "num: [1, 2";

        // when
        let result = sut.load(String::from(input)).err().unwrap();

        // then
        assert_eq!(InvalidYaml, result);
    }

    #[test]
    fn loader_load_incorrect_path() {
        // given
        let mut sut = Loader::new();

        // when
        let result = sut.load_from_path("nonexistent.yaml").err().unwrap();

        // then
        assert_eq!(IOError, result);
    }
}