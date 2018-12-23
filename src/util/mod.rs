use std::fs;
use std::io::Write;
use std::path::Path;

use physics::types::Body;
//use geometry::types::Point;
//use geometry::types::Vector;

// DataWriter ////////////////////////////////////////////////////////////////
//
// A utility object to simplify persistence of point data. Each call to
// write will generate a new file in the specified directory. Files are
// sequentially numbered.

pub struct DataWriter {
    directory: String,
    counter: u32,
}

impl DataWriter {
    /// Creates a new directory in the current working path.
    pub fn new(directory: &str) -> DataWriter {
        if !Path::new(directory).exists() {
            fs::create_dir(directory)
                .expect("Couldn't create dir.");
        }
        DataWriter {
            directory: directory.to_owned(),
            counter: 0
        }
    }

    /// Creates a new file in the writers directory with each point written
    /// on a separate line.
    pub fn write(&mut self, bodies: Vec<Body>) {
        let path = format!("{}/frame-{}.txt", self.directory, self.counter);
        match self.write_bodies(bodies, path) {
            Err(e) => panic!("Error writing data. {}", e),
            Ok(_) => (),
        }
        self.counter += 1;
    }

    fn write_bodies(&self, bodies: Vec<Body>,path: String) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;

        for body in bodies {
            write!(file, "{},{},{},{},{},\n", body.mass, body.position.x, body.position.y,
                                            body.velocity.dx, body.velocity.dy)?;
        }

        Ok(())
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn data_writer_writes() {
        // given
        let mut writer = DataWriter::new("temp");
        let b1 = Body {
            mass: 1.0,
            position: Point { x: 1.0, y: 2.0 },
            velocity: Vector::zero(),
        };

        let b2 = Body {
            mass: 2.0,
            position: Point { x: 4.0, y: 5.0 },
            velocity: Vector::zero(),
        };

        // when
        writer.write(vec![b1]);
        writer.write(vec![b2]);

        // then
        let mut file = fs::File::open("temp/frame-0.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "1.0, (1.0, 2.0) ,(0.0, 0.0)\n".to_owned());

        let mut file = fs::File::open("temp/frame-1.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "2.0 , (4.0, 5.0) ,(0.0, 0.0) \n".to_owned());

        // after
        fs::remove_dir_all("temp").expect("Error cleaning up test.");
    }
}
