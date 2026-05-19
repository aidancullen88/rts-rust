use std::f64::{
    self,
    consts::{FRAC_PI_2, PI},
};

use crate::point::{Point, is_point_distance_leq};

#[derive(Clone, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
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

    pub fn angle(&self) -> f64 {
        let mut quad_rad = self.y.atan2(self.x);
        if quad_rad <= 0.0 {
            quad_rad += 2.0 * f64::consts::PI;
        }
        return quad_rad;
    }
    
    pub fn dot(&self, v: &Vector) -> f64 {
        (self.x * v.x) + (self.y * v.y)
    }
    pub fn sub(&self, v: &Vector) -> Vector {
        Vector::new(self.x - v.x, self.y - v.y)
    }
    pub fn rotate(&self, theta: f64) -> Vector {
        Vector::new(
            (self.x * theta.cos()) - (self.y * theta.sin()),
            (self.x * theta.sin()) + (self.y * theta.cos()),
        )
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
    distance: f64,
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

pub fn reverse_vector(v: &Vector) -> Vector {
    Vector { x: -v.x, y: -v.y }
}

pub enum Quad {
    LeftUp,
    RightUp,
    RightDown,
    LeftDown,
}

pub fn get_vector_quad(v: &Vector) -> Option<Quad> {
    if v.x == 0.0 && v.y == 0.0 {
        return None;
    }
    let angle = v.angle();
    if angle < FRAC_PI_2 {
        Some(Quad::LeftUp)
    } else if angle < PI {
        Some(Quad::RightUp)
    } else if angle < FRAC_PI_2 * 3.0 {
        Some(Quad::RightDown)
    } else {
        Some(Quad::LeftDown)
    }
}

pub fn check_ray_collides_circle(origin: &Point, direction: &Vector, circle_pos: &Point, circle_radius: f64) -> Option<Point> {
    let vec_npc_origin = circle_pos.sub(origin).into_vec();
    let circle_projection = vec_npc_origin.dot(direction).max(0.0);
    let closest = translate_point_direction_distance(origin, direction, circle_projection);
    if is_point_distance_leq(circle_pos, &closest, circle_radius) {
        Some(closest)
    } else {
        None
    }
}

///// Given 3 vectors (assumed to be with the same origin), returns true if v is at an angle between
///// the other two
// pub fn vector_is_between(v: &Vector, a: &Vector, b: &Vector) -> bool {
//     const FLOAT_TOLERANCE: f64 = 1e-6;
//     let (v_rad, a_rad, b_rad) = (v.angle(), a.angle(), b.angle());
//     // If the
//     if (a_rad - b_rad).abs() < FLOAT_TOLERANCE {
//         return false;
//     }
//     if ((a_rad - b_rad).abs() - PI).abs() < FLOAT_TOLERANCE {
//         return false;
//     }
//     v_rad >= a_rad.min(b_rad) && v_rad <= a_rad.max(b_rad)
// }
