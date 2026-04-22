use graphics::{Context, Graphics};

use crate::{GameState, cell_map, point::Point, vector::{self, Vector}};

pub struct Npc {
    id: usize,
    pos: Point,
    look_dir: Vector,
    radius: f64,
}

impl Npc {
    pub fn new(game_state: &mut GameState, cell_map: &mut cell_map::Cells, pos: Point) -> Npc {
        let npc_id = game_state.get_next_entity_id();
        cell_map.update_position(&pos, npc_id);
        Npc {
            id: npc_id,
            pos,
            look_dir: [1.0, 0.0].into(),
            radius: 15.0,
        }
    }
    
    pub fn set_look_dir(mut self, look_dir: Vector) -> Self {
        self.look_dir = look_dir;
        self
    }
}

pub fn render_npcs<G: Graphics>(npc_list: &Vec<Npc>, c: &Context, g: &mut G) {
    for npc in npc_list {
        let circum = npc.radius * 2.0;
        // Render npc circle
        graphics::Ellipse::new_border(graphics::color::WHITE, 0.5)
            .resolution(128)
            .draw(
                [
                    npc.pos.x - npc.radius,
                    npc.pos.y - npc.radius,
                    circum,
                    circum,
                ],
                &c.draw_state,
                c.transform,
                g,
            );
        // Calculate the positions for the view direction
        let circum_point =
            vector::translate_point_direction_distance(&npc.pos, &npc.look_dir, &npc.radius);
        let extended_point = vector::translate_point_direction_distance(
            &npc.pos,
            &npc.look_dir,
            &(npc.radius + 10.0),
        );
        // Render the little "looking this way" line
        graphics::Line::new(graphics::color::RED, 1.0).draw_from_to(
            &circum_point,
            &extended_point,
            &c.draw_state,
            c.transform,
            g,
        );
    }
}
