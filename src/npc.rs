use graphics::{Context, Graphics};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::mem;

use crate::EffectQueue;
use crate::effect::Bullet;
use crate::event::{Event, EventQueue, EventType};
use crate::point::get_distance_between_points;
use crate::vector::get_direction_between_points;
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
        team: NpcTeam,
        game_state: &mut GameState,
    ) -> Id {
        let npc_id = game_state.get_next_entity_id();
        let mut new_npc = Npc::new(npc_id, &mut self.cell_map, pos.clone(), team);
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

    pub fn delete_npc(&mut self, id: &Id) {
        let npc_cell = self
            .get_npc_by_id(&id)
            .expect("Should def be an npc with this id")
            .knowledge
            .current_cell
            .clone();
        self.map.remove(id);
        self.cell_map.remove_from_cell(&npc_cell, id);
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

    pub fn get_mut_npc_by_id(&mut self, id: &Id) -> Option<&mut Npc> {
        self.map.get_mut(id)
    }

    pub fn queue_task_all_npcs(&mut self, task: Task) {
        for npc in self.map.values_mut() {
            if !matches!(npc.attributes.status, NpcStatus::Dead) {
                npc.queue_task(task.clone());
            }
        }
    }

    pub fn update_npcs(
        &mut self,
        game_map: &GameMap,
        effects: &mut crate::EffectQueue,
        event_queue: &mut EventQueue,
        dt: &f64,
    ) {
        while let Some((id, event)) = event_queue.get_next_event() {
            let npc = self
                .get_mut_npc_by_id(&id)
                .expect("Event for npc that doesn't exist, was it deleted?");
            match event {
                Event::Instant(etype) => match etype {
                    EventType::Shot(_) => npc.kill(),
                },
            }
        }
        let npc_info = self.get_npc_info_map();
        for npc in self.map.values_mut() {
            npc.act(
                &mut self.cell_map,
                game_map,
                &npc_info,
                effects,
                event_queue,
                dt,
            );
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

#[derive(Clone, Debug)]
pub struct NpcAttributes {
    pub speed: f64,
    pub vision: f64,
    pub radius: f64,
    pub position: Point,
    pub team: NpcTeam,
    pub status: NpcStatus,
}

struct NpcKnowledge {
    movement_target: Option<Point>,
    current_cell: CellPos,
    enemy_direction: Option<Vector>,
    cover_target: Option<Id>,
    full_covers: HashSet<Id>,
    shoot_dir: Option<Vector>,
}

struct NpcTasks {
    current_action: Option<Action>,
    queue: std::collections::VecDeque<Task>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NpcStatus {
    Alive,
    Dead,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NpcTeam {
    Blue,
    Red,
}

impl NpcTeam {
    fn get_enemies(&self) -> Option<&NpcTeam> {
        match &self {
            NpcTeam::Blue => Some(&NpcTeam::Red),
            NpcTeam::Red => Some(&NpcTeam::Blue),
            _ => None,
        }
    }
}

impl Npc {
    pub fn new(npc_id: Id, cells: &mut Cells, pos: Point, team: NpcTeam) -> Npc {
        let current_cell = cells.register_initial_position(&pos, &npc_id);
        Npc {
            id: npc_id,
            knowledge: NpcKnowledge {
                movement_target: None,
                current_cell,
                enemy_direction: None,
                cover_target: None,
                full_covers: HashSet::new(),
                shoot_dir: None,
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
                team,
                status: NpcStatus::Alive,
            },
        }
    }

    pub fn kill(&mut self) {
        self.attributes.status = NpcStatus::Dead;
        self.tasks.current_action = None;
        self.tasks.queue.clear();
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

    pub fn get_team(&self) -> &NpcTeam {
        &self.attributes.team
    }

    // pub fn check_target_position(&self, cells: &Cells, new_pos: &Point) -> bool {
    //     // Check if the target collides with any other npcs
    //     // If so, return false, if not return true
    // }

    pub fn get_current_task(&self) -> Option<&Task> {
        self.tasks.queue.front()
    }

    // fn process_event(&mut self, event_type: &EventType) {
    //     match event_type {
    //         EventType::Shot(_) =>
    //     }
    // }

    fn act(
        &mut self,
        cells: &mut cell_map::Cells,
        game_map: &GameMap,
        npc_info: &HashMap<Id, NpcAttributes>,
        effects: &mut EffectQueue,
        event_queue: &mut EventQueue,
        dt: &f64,
    ) {
        match &mut self.tasks.current_action {
            None => {
                self.setup_next_task(cells, game_map, npc_info);
                return;
            },
            Some(Action::Moving) => self.move_npc(cells, game_map, npc_info, dt),
            Some(Action::Shooting(timer)) => {
                // Only shoot at the start of the action
                if timer.current == 0.0 {
                    self.shoot(cells, npc_info, effects, event_queue)
                }
            }
        };
        match &mut self.tasks.current_action {
            None => {},
            Some(Action::Moving) => {},
            Some(Action::Shooting(timer)) => {
                if timer.current >= timer.limit {
                    self.end_current_action();
                } else {
                    timer.current += dt;
                }
            },
        };
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
            TaskType::FindTarget => self.find_target(cells, npc_info),
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
            self.knowledge.cover_target = None;
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
            // Find another cover spot if we were moving to cover
            // If not, just stop moving
            if self.knowledge.cover_target.is_some() {
                self.find_close_cover(cells, game_map, npc_info);
            } else {
                self.end_current_action();
            };
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

    fn shoot(
        &mut self,
        cells: &Cells,
        npc_info: &HashMap<Id, NpcAttributes>,
        effects: &mut EffectQueue,
        event_queue: &mut EventQueue,
    ) {
        // println!("{} is shooting", self.id);
        let shoot_dir = self
            .knowledge
            .shoot_dir
            .as_ref()
            .expect("Shouldn't be shooting without a target!");
        // get unified list of all entity positions and bounds
        // cast ray and get first collision
        let hit_option =
            cells.check_if_ray_collides_with_npc(self.get_position(), &shoot_dir, npc_info, &self);
        // Convert the option tuple to just contain the end point as Bullet::new expects an
        // Option<&Point> for the end point
        let end_point = hit_option.as_ref().map(|t| &t.1);
        // println!("target_hit (should always be true!): {:#?}", hit_option);
        if let Some((id, _, _)) = hit_option {
            event_queue.add_event(*id, Event::Instant(EventType::Shot(100.0)));
        }
        // Calculate start and end point based off collisions
        effects.push(Box::new(Bullet::new(
            self.get_position(),
            &shoot_dir,
            end_point,
        )));
        println!("Bang");
    }

    fn target_move(&mut self, target_point: &Point) {
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
        let Some(cover_target) = self
            .knowledge
            .cover_target
            .and_then(|id| game_map.cover.get(&id))
            // Don't use the current cover if it's marked as full
            .and_then(|cover| {
                (!self.knowledge.full_covers.contains(cover.get_id())).then_some(cover)
            })
            // If there's no current cover, try and get another one from ones nearby
            .or_else(|| {
                get_random_cover_target(
                    &game_map.cover,
                    &self.knowledge.full_covers,
                    &self.attributes.position,
                    self.attributes.vision,
                )
            })
        else {
            // If there were no covers in the list, just stop moving (for now).
            // TODO: This should probably return to another decision later on
            self.end_current_action();
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
            // Check the current cover position
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
            if cells
                .check_if_npc_target_collides_with_npc(
                    &candidate_pos,
                    npc_info,
                    self.attributes.radius,
                )
                .is_none()
            {
                // If this spot is good, then finish the loop
                break candidate_pos;
            }

            // If this point wasn't a fit, then we check the increments:
            // If we checked the positive direction last, check the negative now
            if vert_adjust_accum > 0.0 {
                vert_adjust_accum = -vert_adjust_accum;
                continue;
            // Otherwise, we must have checked the negative or 0 direction so check the positive
            // direction
            } else {
                // Get the new vertical adjustment
                let new_vert_adjust =
                    vert_adjust_accum.abs() + self.attributes.radius * 2.0 + EXCLUSION_RADIUS;
                // If the vert adjustment is going to put the npc out of cover, then reset to 0 and
                // increment the horizontal adjustment
                if new_vert_adjust >= cover_radius {
                    horz_adjust_accum += self.attributes.radius * 2.0 + EXCLUSION_RADIUS;
                    vert_adjust_accum = 0.0;
                } else {
                    // If it's not, then we will go check the positive direction
                    vert_adjust_accum = new_vert_adjust;
                    continue;
                }
            }
            // Check if the horizontal adjustment puts them out of cover. This exact method could
            // change (cover radius is an odd metric). If so, then stop moving and insert this
            // cover into the npc's list of full covers. If not, then keep looping and check the
            // next midpoint
            if horz_adjust_accum >= cover_radius {
                // println!("couldn't find cover here {}", horz_adjust_accum);
                self.end_current_action();
                // Why does this not exclude the full cover??
                self.knowledge.full_covers.insert(*cover_target.get_id());
                // Find another cover. Eventually, this should exclude any covers we know are full?
                self.tasks
                    .queue
                    .push_back(Task::new(TaskType::FindCloseCover));
                return;
            }
        };
        // If we made it out of the loop, we have a cover target pos to move to. Set the movement
        // target, the cover target, and the action
        self.knowledge.movement_target = Some(target_pos);
        self.knowledge.cover_target = Some(*cover_target.get_id());
        self.tasks.current_action = Some(Action::Moving);
    }

    fn find_target(&mut self, cells: &cell_map::Cells, npc_info: &HashMap<Id, NpcAttributes>) {
        // Get the closest N
        let closest_enemy = npc_info
            .iter()
            .filter(|(id, attr)| {
                // Only check npcs on the other team
                &attr.team != &self.attributes.team
                    // Don't target itself
                    && **id != self.id
                    // Don't target dead npcs
                    && !matches!(attr.status, NpcStatus::Dead)
            })
            .map(|(id, attr)| {
                let distance = get_distance_between_points(self.get_position(), &attr.position);
                (id, distance, attr)
            })
            .min_by(|x, y| x.1.total_cmp(&y.1));
        let Some(enemy) = closest_enemy else {
            self.tasks.queue.push_front(Task::new(TaskType::FindTarget));
            return;
        };
        // Get the shoot direction and also move it by between +- PI/4
        let shoot_dir = get_direction_between_points(&self.get_position(), &enemy.2.position)
            .rotate((fastrand::f64() - 0.5) * (PI / 16.0));
        // println!("{} decided to shoot in direction {:#?}!", self.id, shoot_dir);
        self.knowledge.shoot_dir = Some(shoot_dir);
        // This is a temp const, this would be determined by weapon/xp/etc
        const SHOOT_TIMER: f64 = 1.0;
        self.tasks.current_action = Some(Action::Shooting(Timer::new(SHOOT_TIMER)));
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
    FindTarget,
}

impl Task {
    pub fn new(task_type: TaskType) -> Task {
        Task { task_type }
    }
}

enum Action {
    Moving,
    Shooting(Timer),
}

struct Timer {
    current: f64,
    limit: f64,
}

impl Timer {
    fn new(duration: f64) -> Timer {
        Timer {
            current: 0.0,
            limit: duration,
        }
    }
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
            _ => match npc.attributes.status {
                NpcStatus::Alive => graphics::color::WHITE,
                NpcStatus::Dead => graphics::color::GRAY,
            },
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
