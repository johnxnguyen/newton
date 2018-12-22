use std::fs;
use std::io::Write;
use std::path::Path;

use geometry::types::Point;
use geometry::types::Vector;

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
    pub fn write(&mut self, points: Vec<Point>, velocities: Vec<Vector> ) {
        let path = format!("{}/frame-{}.txt", self.directory, self.counter);
        match self.write_points(points, velocities, path) {
            Err(e) => panic!("Error writing data. {}", e),
            Ok(_) => (),
        }
        self.counter += 1;
    }

    fn write_points(&self, points: Vec<Point>, velocities: Vec<Vector> ,path: String) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        for point in points {
            for velocity in velocities{
                write!(file, "{},{},{},{},\n", point.x, point.y, velocity.dx, velocity.dy)?;
            }
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

        // when
        writer.write(vec![Point::new(3.4, 6.7)]);
        writer.write(vec![Point::new(6.4, 6.785)]);

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
}
