use crate::point::Point;

pub struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Vector {
        Vector { x, y }
    }

    pub fn normalise(mut self) -> Self {
        let mag = (self.x * self.x + self.y * self.y).sqrt();
        self.x = self.x / mag;
        self.y = self.y / mag;
        self
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

/// Moves a point in a direction by a distance. The vector passed in here should usually be
/// normalised. If not, the distance moved will be affected by the vector's magnitude
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

pub fn get_direction_between_points(a: &Point, b: &Point) -> Vector {
    Vector {
        x: b.x - a.x,
        y: b.y - a.y,
    }
    .normalise()
}
