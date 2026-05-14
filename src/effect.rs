use std::collections::VecDeque;

use crate::point::Point;
use graphics::{Context, Graphics};

pub trait Effect<G: Graphics> {
    fn render(&self, c: &Context, g: &mut G);
    fn tick(&mut self) -> bool;
    fn print(&self) -> String;
}

pub struct Bullet {
    start: Point,
    end: Point,
    frame_duration: usize,
}

impl Bullet {
    pub fn new(start: Point, end: Point) -> Bullet {
        Bullet { start, end, frame_duration: 5 }
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
    fn tick(&mut self) -> bool {
        if self.frame_duration == 1 { 
            return true
        };
        self.frame_duration -= 1;
        false
    }
    fn print(&self) -> String {
        format!("Bullet")
    }
}

pub fn render_effects<G: Graphics>(effects: &mut Vec<Box<dyn Effect<G>>>, c: &Context, g: &mut G) {
    effects.retain_mut(|effect| {
        effect.render(c, g);
        !effect.tick()
    });
}
