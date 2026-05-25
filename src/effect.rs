use std::collections::VecDeque;

use crate::{point::Point, vector::{Vector, translate_point_direction_distance}};
use graphics::{Context, Graphics};

/// The Effect trait allows the effect renderer to remain simple and generic by defining some
/// common properties of effects
pub trait Effect<G: Graphics> {
    fn render(&self, c: &Context, g: &mut G);
    fn tick(&mut self, dt: f64) -> bool;
    fn print(&self) -> String;
}

pub fn new_effect_queue<G: Graphics>() -> Vec<Box<dyn Effect<G>>> {
    Vec::new()
}

pub struct Bullet {
    start: Point,
    end: Point,
    duration: f64,
}

impl Bullet {
    pub fn new(origin: &Point, direction: &Vector, end: Option<&Point>) -> Bullet {
        const DURATION: f64 = 0.005;
        let end_point = match end {
            Some(point) => point.clone(),
            None => translate_point_direction_distance(origin, direction, 2000.0),
        };
        Bullet { start: origin.clone(), end: end_point, duration: DURATION }
    }
}

impl <G: Graphics>Effect<G> for Bullet {
    fn render(&self, c: &graphics::Context, g: &mut G) {
        graphics::Line::new(graphics::color::WHITE, 1.0).draw_from_to(
            &self.start,
            &self.end,
            &c.draw_state,
            c.transform,
            g,
        );
    }
    fn tick(&mut self, dt: f64) -> bool {
        self.duration -= 1.0 * dt;
        if self.duration <= 0.0 {
            return true;
        }
        false
    }
    fn print(&self) -> String {
        "Bullet".to_string()
    }
}

pub fn render_effects<G: Graphics>(effects: &mut Vec<Box<dyn Effect<G>>>, c: &Context, g: &mut G, dt: f64) {
    effects.retain_mut(|effect| {
        effect.render(c, g);
        !effect.tick(dt)
    });
}
