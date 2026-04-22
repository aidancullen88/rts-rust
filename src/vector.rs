use crate::point::Point;

pub struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Vector {
        Vector { x, y }
    }
}

impl From<Vector> for [f64; 2] {
    fn from(v: Vector) -> [f64; 2] {
        [v.x, v.y]
    }
}

impl From<&Vector> for [f64; 2] {
    fn from(v: &Vector) -> [f64; 2] {
        [v.x, v.y]
    }
}


impl From<[f64; 2]> for Vector {
    fn from(f: [f64; 2]) -> Vector {
        Vector { x: f[0], y: f[1] }
    }
}

impl From<&[f64; 2]> for Vector {
    fn from(f: &[f64; 2]) -> Vector {
        Vector { x: f[0], y: f[1] }
    }
}

pub fn translate_point_direction_distance(
    point: &Point,
    direction: &Vector,
    distance: &f64,
) -> Point {
    Point {
        x: point.x + (direction.x * distance),
        y: point.y + (direction.y * distance),
    }
}
