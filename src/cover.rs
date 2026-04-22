use graphics::{Context, Graphics};

use crate::cell_map::Cells;
use crate::point;
use crate::point::Point;

pub struct Cover {
    start: Point,
    end: Point,
    midpoint: Point,
}

pub fn init_covers(simple_cover_list: [[u32; 4]; 6]) -> Vec<Cover> {
    simple_cover_list
        .iter()
        .map(|c| {
            let start_point = Point::new(c[0].into(), c[1].into());
            let end_point = Point::new(c[2].into(), c[3].into());
            let mid_point = point::calculate_midpoint(&start_point, &end_point);
            Cover {
                start: start_point,
                end: end_point,
                midpoint: mid_point,
            }
        })
        .collect()
}

pub fn render_covers<G: Graphics>(covers: &Vec<Cover>, c: &Context, g: &mut G) {
    use graphics::Line;
    for cover in covers {
        // render
        Line::new(graphics::color::WHITE, 1.0).draw_from_to(
            &cover.start,
            &cover.end,
            &c.draw_state,
            c.transform,
            g,
        );
    }
}

pub fn render_grid<G: Graphics>(cell_map: &Cells, c: &Context, g: &mut G) {
    use graphics::Line;
    // render out grid properly
    // TODO: render the vertical lines and position the npc ids correctly
    const CELL_SIZE: f64 = 100.0;
    const GREY: [f32; 4] = [1.0, 1.0, 1.0, 0.1];
    let range = 1..10;
    for value in range {
        let float_val = f64::from(value);
        Line::new(GREY, 1.0).draw_from_to(
            [0.0, float_val * CELL_SIZE],
            [1000.0, float_val * CELL_SIZE],
            &c.draw_state,
            c.transform,
            g,
        );
    }
}
