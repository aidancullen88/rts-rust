use std::collections::VecDeque;

use crate::{point::Point, vector::{Vector, translate_point_direction_distance}};
use graphics::{Context, Graphics};

pub trait Effect<G: Graphics> {
    fn render(&self, c: &Context, g: &mut G);
    fn tick(&mut self, dt: f64) -> bool;
    fn print(&self) -> String;
}

pub struct Bullet {
    start: Point,
    end: Point,
    duration: f64,
}

impl Bullet {
    pub fn new(origin: &Point, direction: &Vector, end: Option<&Point>) -> Bullet {
        const DURATION: f64 = 0.005;
        if let Some(end_point) = end {
            Bullet { start: origin.clone(), end: end_point.clone(), duration: DURATION }
        } else {
            let end_point = translate_point_direction_distance(origin, direction, 500.0);
            Bullet { start: origin.clone(), end: end_point, duration: DURATION }
        }
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
        format!("Bullet")
    }
}

pub fn render_effects<G: Graphics>(effects: &mut Vec<Box<dyn Effect<G>>>, c: &Context, g: &mut G, dt: f64) {
    effects.retain_mut(|effect| {
        effect.render(c, g);
        !effect.tick(dt)
    });
}
