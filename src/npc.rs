use graphics::{Context, Graphics};
use std::collections::{HashMap, HashSet};

use crate::{
    GameMap, GameState,
    cell_map::{self, CellPos, Cells},
    cover::get_random_cover_target,
    point::{self, Point},
    vector::{self, Vector},
};

/// Id represents the entity id for each entity in the game
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct Id(pub usize);

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct NpcMap {
    pub cell_map: Cells,
    map: HashMap<Id, Npc>,
    selected: Option<Id>,
}

// This can probably go in it's own file at some point
impl NpcMap {
    pub fn new(cell_size: f64) -> NpcMap {
        NpcMap {
            map: HashMap::new(),
            selected: None,
            cell_map: Cells::new(cell_size),
        }
    }

    pub fn spawn_npc(
        &mut self,
        pos: Point,
        look_dir: Vector,
        enemy_dir: Option<Vector>,
        game_state: &mut GameState,
    ) -> Id {
        let npc_id = game_state.get_next_entity_id();
        let mut new_npc = Npc::new(npc_id, &mut self.cell_map, pos.clone());
        new_npc.set_look_dir(look_dir);
        if let Some(dir) = enemy_dir {
            new_npc.set_enemy_dir(dir);
        }
        self.map.insert(npc_id, new_npc);
        npc_id
    }

    pub fn get_npc_by_id_mut(&mut self, id: &Id) -> Option<&mut Npc> {
        self.map.get_mut(id)
    }

    pub fn get_npc_by_id(&self, id: &Id) -> Option<&Npc> {
        self.map.get(id)
    }

    pub fn get_npc_iter(&self) -> impl Iterator<Item = &Npc> {
        self.map.values()
    }

    pub fn get_npc_info_map(&self) -> HashMap<Id, NpcAttributes> {
        self.map
            .iter()
            .map(|(id, npc)| (*id, npc.attributes.clone()))
            .collect()
    }

    pub fn select_npc(&mut self, id: Id) {
        self.selected = Some(id);
    }

    pub fn deselect_npc(&mut self) {
        self.selected = None;
    }

    pub fn get_selected_npc(&mut self) -> Option<&mut Npc> {
        self.selected.and_then(|s| self.map.get_mut(&s))
    }

    pub fn get_selected_npc_id(&self) -> Option<&Id> {
        self.selected.as_ref()
    }

    pub fn clear_npcs(&mut self) {
        self.map.clear();
        self.cell_map.clear();
    }

    pub fn update_npcs(&mut self, game_map: &GameMap, dt: &f64) {
        let npc_info = self.get_npc_info_map();
        for npc in self.map.values_mut() {
            npc.act(&mut self.cell_map, game_map, &npc_info, dt);
        }
    }
}

pub struct Npc {
    id: Id,
    knowledge: NpcKnowledge,
    look_dir: Vector,
    tasks: NpcTasks,
    attributes: NpcAttributes,
}

#[derive(Clone)]
pub struct NpcAttributes {
    pub speed: f64,
    pub vision: f64,
    pub radius: f64,
    pub position: Point,
}

struct NpcKnowledge {
    movement_target: Option<Point>,
    current_cell: CellPos,
    enemy_direction: Option<Vector>,
    full_covers: HashSet<Id>,
}

struct NpcTasks {
    current_action: Option<Action>,
    queue: std::collections::VecDeque<Task>,
}

impl Npc {
    pub fn new(npc_id: Id, cells: &mut Cells, pos: Point) -> Npc {
        let current_cell = cells.register_initial_position(&pos, &npc_id);
        Npc {
            id: npc_id,
            knowledge: NpcKnowledge {
                movement_target: None,
                current_cell,
                enemy_direction: None,
                full_covers: HashSet::new(),
            },
            look_dir: [1.0, 0.0].into(),
            tasks: NpcTasks {
                current_action: None,
                queue: std::collections::VecDeque::new(),
            },
            attributes: NpcAttributes {
                speed: 100.0,
                vision: 500.0,
                radius: 10.0,
                position: pos,
            },
        }
    }

    pub fn set_look_dir(&mut self, look_dir: Vector) {
        self.look_dir = look_dir;
    }

    pub fn set_enemy_dir(&mut self, enemy_dir: Vector) {
        self.knowledge.enemy_direction = Some(enemy_dir);
    }

    pub fn update_position(&mut self, cells: &mut Cells, new_pos: Point) {
        if let Some(new_cell) =
            cells.update_position(&new_pos, &self.knowledge.current_cell, &self.id)
        {
            self.knowledge.current_cell = new_cell;
        }
        self.attributes.position = new_pos;
    }

    pub fn get_position(&self) -> &Point {
        &self.attributes.position
    }

    pub fn get_id(&self) -> Id {
        self.id
    }

    // pub fn check_target_position(&self, cells: &Cells, new_pos: &Point) -> bool {
    //     // Check if the target collides with any other npcs
    //     // If so, return false, if not return true
    // }

    pub fn get_current_task(&self) -> Option<&Task> {
        self.tasks.queue.front()
    }

    fn act(
        &mut self,
        cells: &mut cell_map::Cells,
        game_map: &GameMap,
        npc_info: &HashMap<Id, NpcAttributes>,
        dt: &f64,
    ) {
        if let Some(action) = &self.tasks.current_action {
            match action {
                Action::Moving => self.move_npc(cells, game_map, npc_info, dt),
            }
        } else {
            self.setup_next_task(cells, game_map, npc_info);
        }
    }

    pub fn setup_next_task(
        &mut self,
        cells: &cell_map::Cells,
        game_map: &GameMap,
        npc_info: &HashMap<Id, NpcAttributes>,
    ) {
        let Some(current_task) = self.tasks.queue.pop_front() else {
            return;
        };
        match &current_task.task_type {
            TaskType::Move(target_point) => self.target_move(&target_point),
            TaskType::FindCloseCover => self.find_close_cover(cells, game_map, npc_info),
        }
    }

    pub fn queue_task(&mut self, task: Task) {
        self.tasks.queue.push_back(task);
    }

    pub fn end_current_action(&mut self) {
        self.tasks.current_action = None;
    }

    fn move_npc(
        &mut self,
        cells: &mut cell_map::Cells,
        game_map: &GameMap,
        npc_info: &HashMap<Id, NpcAttributes>,
        dt: &f64,
    ) {
        // If there's no movement target, then don't move
        let Some(movement_target) = self.knowledge.movement_target.clone() else {
            return;
        };
        // If the npc is close enough to the movement target, stop moving and end the task
        if point::is_point_distance_leq(&self.attributes.position, &movement_target, 1.0) {
            // Finish current movement task
            self.end_current_action();
            if let Some(enemy_dir) = &self.knowledge.enemy_direction {
                self.look_dir = enemy_dir.clone();
            };
            return;
        };
        // If we're close to the destination, begin checking if another npc has moved into the
        // target spot
        if point::is_point_distance_leq(&self.attributes.position, &movement_target, 100.0)
            && matches!(cells.check_if_npc_target_collides_with_npc(&movement_target, npc_info, self.attributes.radius), Some(id) if id != self.id)
        {
            println!("Someone took my spot!");
            // Find another cover spot, we can proceed with moving after that
            self.find_close_cover(cells, game_map, npc_info);
        };
        let movement_direction =
            vector::get_direction_between_points(&self.attributes.position, &movement_target);
        let new_pos = vector::translate_point_direction_distance(
            &self.attributes.position,
            &movement_direction,
            self.attributes.speed * dt,
        );
        self.look_dir = movement_direction;
        self.update_position(cells, new_pos);
    }

    fn target_move(&mut self, target_point: &Point) {
        println!("targeting move to point: {:#?}", target_point);
        self.knowledge.movement_target = Some(target_point.to_owned());
        self.tasks.current_action = Some(Action::Moving);
    }

    fn find_close_cover(
        &mut self,
        cells: &cell_map::Cells,
        game_map: &GameMap,
        npc_info: &HashMap<Id, NpcAttributes>,
    ) {
        // The distance npc's should try and keep from each other and from objects when positioning
        const EXCLUSION_RADIUS: f64 = 5.0;
        let Some(cover_target) = get_random_cover_target(
            &game_map.cover,
            &self.knowledge.full_covers,
            &self.attributes.position,
            self.attributes.vision,
        ) else {
            return;
        };
        // Used to make sure npc's don't position themselves outside the cover
        let cover_radius = cover_target.get_length() / 2.0;
        // Position the npc correctly on the cover by collision checking
        let enemy_dir = self
            .knowledge
            .enemy_direction
            .as_ref()
            .expect("if there's no enemies, why are we taking cover?");

        let cover_midpoint = cover_target.get_midpoint();
        let rev_enemy_dir = vector::reverse_vector(&enemy_dir);

        // Accumulators for adjusting the position if there's an npc in the way
        let mut vert_adjust_accum = 0.0;
        let mut horz_adjust_accum = self.attributes.radius + EXCLUSION_RADIUS;
        // Attempt a bunch of positions behind the cover, checking at each one if there's already
        // an npc there
        let target_pos = loop {
            let new_cover_point = vector::translate_point_direction_distance(
                cover_midpoint,
                cover_target.get_direction(),
                vert_adjust_accum,
            );
            let candidate_pos = vector::translate_point_direction_distance(
                &new_cover_point,
                &rev_enemy_dir,
                horz_adjust_accum,
            );
            if cells.check_if_npc_target_collides_with_npc(&candidate_pos, npc_info, self.attributes.radius).is_none() {
                break candidate_pos;
            }

            // If this point wasn't a fit, then we check the increments:
            if vert_adjust_accum > 0.0 {
                vert_adjust_accum = -vert_adjust_accum;
            } else {
                let new_vert_adjust = vert_adjust_accum.abs() + self.attributes.radius * 2.0 + EXCLUSION_RADIUS;
                if new_vert_adjust >= cover_radius {
                    horz_adjust_accum += self.attributes.radius * 2.0 + EXCLUSION_RADIUS;
                    vert_adjust_accum = 0.0;
                } else {
                    vert_adjust_accum += new_vert_adjust;
                }
            }
            if horz_adjust_accum >= cover_radius {
                println!("couldn't find cover here {}", horz_adjust_accum);
                self.knowledge.full_covers.insert(*cover_target.get_id());
                // Find another cover. Eventually, this should exclude any covers we know are full?
                self.tasks.queue.push_back(Task::new(TaskType::FindCloseCover));
                return;
            }
        };
        println!("found a target at this cover, moving now!");
        self.knowledge.movement_target = Some(target_pos);
        self.tasks.current_action = Some(Action::Moving);
    }
}

#[derive(Clone)]
pub struct Task {
    task_type: TaskType,
}

#[derive(Clone)]
pub enum TaskType {
    Move(Point),
    FindCloseCover,
}

impl Task {
    pub fn new(task_type: TaskType) -> Task {
        Task { task_type }
    }
}

enum Action {
    Moving,
}

pub fn render_npcs<'a, G: Graphics>(
    npc_list: impl Iterator<Item = &'a Npc>,
    selected_npc: Option<&Id>,
    c: &Context,
    g: &mut G,
) {
    for npc in npc_list {
        let npc_colour = match selected_npc {
            Some(id) if *id == npc.get_id() => graphics::color::RED,
            _ => graphics::color::WHITE,
        };
        let circum = npc.attributes.radius * 2.0;
        // Render npc circle
        graphics::Ellipse::new_border(npc_colour, 0.5)
            .resolution(128)
            .draw(
                [
                    npc.attributes.position.x - npc.attributes.radius,
                    npc.attributes.position.y - npc.attributes.radius,
                    circum,
                    circum,
                ],
                &c.draw_state,
                c.transform,
                g,
            );
        // Calculate the positions for the view direction
        let circum_point = vector::translate_point_direction_distance(
            &npc.attributes.position,
            &npc.look_dir,
            npc.attributes.radius,
        );
        let extended_point = vector::translate_point_direction_distance(
            &npc.attributes.position,
            &npc.look_dir,
            npc.attributes.radius + 10.0,
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
