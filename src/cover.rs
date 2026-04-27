use graphics::{Context, Graphics};
use opengl_graphics::{GlyphCache, Texture};

use crate::cell_map::{CellPos, Cells};
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

pub fn render_grid<G: Graphics<Texture = Texture>>(
    cell_map: &Cells,
    c: &Context,
    g: &mut G,
    glyphs: &mut GlyphCache,
) {
    use graphics::Line;
    // render out grid properly
    let cell_size = cell_map.get_cell_size();
    const GREY: [f32; 4] = [1.0, 1.0, 1.0, 0.1];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    for x in 1..12 {
        let float_val = f64::from(x);
        // Draw horizontal lines
        Line::new(GREY, 1.0).draw_from_to(
            [0.0, float_val * cell_size],
            [1500.0, float_val * cell_size],
            &c.draw_state,
            c.transform,
            g,
        );
        // Draw vertical lines
        Line::new(GREY, 1.0).draw_from_to(
            [float_val * cell_size, 0.0],
            [float_val * cell_size, 1500.0],
            &c.draw_state,
            c.transform,
            g,
        );
    }
    for x in 0..20 {
        for y in 0..20 {
            let current_cell = CellPos(x, y);
            if let Some(cell_contents) = cell_map.get_cell_values(&current_cell) {
                // Convert the HashSet contents to a nice vec
                let mut id_list = cell_contents
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>();
                id_list.sort();
                let id_string = id_list.join(" ");
                // Render the text at the botton left of each grid square
                graphics::text::Text::new_color(WHITE, 8)
                    .draw_pos(
                        &id_string,
                        [
                            f64::from(x) * cell_size,
                            f64::from(y) * cell_size + cell_size,
                        ],
                        glyphs,
                        &c.draw_state,
                        c.transform,
                        g,
                    )
                    .unwrap();
            }
        }
    }
}
