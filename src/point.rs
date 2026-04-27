#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}

// impl From<&Point> for Vec2d {
//     fn from(p: &Point) -> Vec2d {
//         [p.x, p.y]
//     }
// }

impl From<Point> for [f64; 2] {
    fn from(p: Point) -> [f64; 2] {
        [p.x, p.y]
    }
}

impl From<&Point> for [f64; 2] {
    fn from(p: &Point) -> [f64; 2] {
        [p.x, p.y]
    }
}

impl From<&[f64; 2]> for Point {
    fn from(a: &[f64; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

impl From<[f64; 2]> for Point {
    fn from(a: [f64; 2]) -> Point {
        Point { x: a[0], y: a[1] }
    }
}

pub fn calculate_midpoint(a: &Point, b: &Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

pub fn check_distance_between_points(a: &Point, b: &Point, distance: &f64) -> bool {
    distance * distance >= get_squared_distance_between_points(a, b)
}

fn get_squared_distance_between_points(a: &Point, b: &Point) -> f64 {
    (b.x - a.x).powi(2) + (b.y - a.y).powi(2)
}
