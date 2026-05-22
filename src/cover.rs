use graphics::{Context, Graphics};
use opengl_graphics::{GlyphCache, Texture};
use std::collections::{HashSet, HashMap};

use crate::GameState;
use crate::cell_map::{CellPos, Cells};
use crate::point::{self, get_distance_between_points, is_point_distance_leq};
use crate::point::Point;
use crate::vector::{self, Vector};
use crate::Id;

#[derive(Debug)]
pub struct Cover {
    id: Id,
    start: Point,
    end: Point,
    midpoint: Point,
    direction: Vector,
    length: f64,
}

impl Cover {
    pub fn get_midpoint(&self) -> &Point {
        &self.midpoint
    }
    
    pub fn get_direction(&self) -> &Vector {
        &self.direction
    }
    
    pub fn get_length(&self) -> &f64 {
        &self.length
    }
    
    pub fn get_id(&self) -> &Id {
        &self.id
    }
}

pub fn init_covers(simple_cover_list: &[[u32; 4]], game_state: &mut GameState) -> HashMap<Id, Cover> {
    simple_cover_list
        .iter()
        .map(|c| {
            let start_point = Point::new(c[0].into(), c[1].into());
            let end_point = Point::new(c[2].into(), c[3].into());
            let mid_point = point::calculate_midpoint(&start_point, &end_point);
            let direction = vector::get_direction_between_points(&start_point, &end_point);
            let length = point::get_distance_between_points(&start_point, &end_point);
            let id = game_state.get_next_entity_id();
            (id, Cover {
                id,
                start: start_point,
                end: end_point,
                midpoint: mid_point,
                direction,
                length,
            })
        })
        .collect()
}

pub fn get_random_cover_target<'a>(covers: &'a HashMap<Id, Cover>, covers_to_exclude: &HashSet<Id>, npc_pos: &Point, threshold: f64) -> Option<&'a Cover> {
    // The power here determines how likely the npc is to pick a cover further away
    // Eventually, this could be passed in from npc info!
    const CHOICE_FACTOR: i32 = 5;
    // Get the covers that are within the threshold, and the weight (threshold - distance)
    let mut filtered_cover_iter: Vec<(f64, &Cover)> = covers.iter().filter_map(|(id, c)| {
        let distance = get_distance_between_points(npc_pos, &c.midpoint);
        if covers_to_exclude.contains(&c.id) {
            return None;
        }
        if distance < threshold {
            Some(((threshold - distance).powi(CHOICE_FACTOR), c))
        } else {
            None
        }
    }).collect();
    // If there's no covers, return None
    filtered_cover_iter.first()?;
    // Get the total of all the weights
    let total_weight: f64 = filtered_cover_iter.iter().map(|t| t.0).sum();
    let random_threshold = fastrand::f64() * total_weight.clone();
    let mut acc = 0.0;
    // Get a random cover from the list, weighted to closer covers
    for (weight, cover) in filtered_cover_iter {
        acc += weight;
        if random_threshold <= acc {
            return Some(cover);
        }
    }
    return None;
}

pub fn get_closest_advancing_cover<'a>(covers: &'a HashMap<Id, Cover>, covers_to_exclude: &HashSet<Id>, npc_pos: &Point, advance_direction: &Vector, threshold: f64) -> Option<&'a Cover> {
    // Get the covers that are within the threshold, and the weight (threshold - distance)
    covers.iter().filter_map(|(id, c)| {
        let distance = get_distance_between_points(npc_pos, &c.midpoint);
        if covers_to_exclude.contains(&c.id) {
            return None;
        }
        if distance > threshold {
            return None;
        }
        // println!("x dir: {}, y dir: {}", (c.midpoint.x - npc_pos.x) * advance_direction.x, (c.midpoint.y - npc_pos.y) * advance_direction.y);
        if advance_direction.x == 0.0 || (c.midpoint.x - npc_pos.x) * advance_direction.x >= 100.0 && advance_direction.y == 0.0 || (c.midpoint.y - npc_pos.y) * advance_direction.y >= 100.0 {
            return Some((distance, c))
        }
        return None;
    }).min_by(|x, y| x.0.total_cmp(&y.0)).map(|x| x.1)
}

pub fn render_covers<G: Graphics>(covers: &HashMap<Id, Cover>, c: &Context, g: &mut G) {
    use graphics::Line;
    for (id, cover) in covers {
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
    for x in 1..100 {
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
    for x in 0..100{
        for y in 0..100 {
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
