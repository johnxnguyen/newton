use geometry::types::Point;
use std::fs::File;

pub fn write_points(points: Vec<Point>, file_name: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    for point in points { write!(file, "{},{}\n", point.x, point.y)?; }
    Ok(())
}
