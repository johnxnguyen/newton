use std::fs;
use std::io::Write;
use std::path::Path;

use geometry::types::{Point, Vector};
use physics::types::Body;

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
    pub fn write_point_file(&mut self, points: Vec<Point>) {
        let path = format!("{}/frame-{}.txt", self.directory, self.counter);
        match self.write_points(points, path) {
            Err(e) => panic!("Error writing data. {}", e),
            Ok(_) => (),
        }
        self.counter += 1;
    }

    fn write_points(&self, points: Vec<Point>, path: String) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        for point in points { write!(file, "{},{}\n", point.x, point.y)?; }
        Ok(())
    }

    /// Creates a new file in the writers directory with each body written on a separate line.
    pub fn write_body_file(&mut self, bodies: Vec<Body>) {
        let path = format!("{}/frame-{}.txt", self.directory, self.counter);
        match self.write_bodies(bodies, path) {
            Err(e) => panic!("Error writing bodies. {},", e),
            Ok(_) => (),
        }
        self.counter += 1;
    }

    pub fn write_bodies(&self, bodies: Vec<Body>, path: String) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        for body in bodies {
            write!(file,"{}-{}-{}-{}-{}\n", body.mass, body.position.x, body.position.y,
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
        let mut writer_points = DataWriter::new("temp");

        // when
        writer_points.write_point_file(vec![Point::new(3.4, 6.7)]);
        writer_points.write_point_file(vec![Point::new(6.4, 6.785)]);

        // then
        let mut file = fs::File::open("temp/frame-0.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "3.4,6.7\n".to_owned());

        let mut file = fs::File::open("temp/frame-1.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "6.4,6.785\n".to_owned());

        // after
        fs::remove_dir_all("temp").expect("Error cleaning up test.");
    }

    fn data_writer_writes_bodies(){
        // given
        let mut writer_bodies = DataWriter::new("temp1");

        // when
        writer_bodies.write_body_file(vec![Body::new(2.0,
                                                     Point::new(1.0,2.0),
                                                     Vector::zero())]);
        writer_bodies.write_body_file(vec![Body::new(3.0,
                                                            Point::new(4.0, 5.0),
                                                            Vector::zero())]);
        // then
        let mut file = fs::File::open("temp1/frame-0.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "2.0-1.0-2.0-0.0-0.0".to_owned());

        let mut file = fs::File::open("temp1/frame-1.txt").expect("Error opening file.");
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        assert_eq!(contents, "3.0-4.0-5.0-0.0-0.0".to_owned());
    }
}
