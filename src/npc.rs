use graphics::{Context, Graphics};

use crate::{
    cell_map::{self, Cells},
    point::Point,
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

pub struct Npc {
    id: Id,
    pos: Point,
    knowledge: NpcKnowledge,
    look_dir: Vector,
    radius: f64,
    tasks: NpcTasks,
}

struct NpcKnowledge {
    movement_target: Option<Point>,
}

struct NpcTasks {
    current_action: Option<Action>,
    queue: std::collections::VecDeque<Task>,
}

impl Npc {
    pub fn new(npc_id: Id, pos: Point) -> Npc {
        Npc {
            id: npc_id,
            pos,
            knowledge: NpcKnowledge {
                movement_target: None,
            },
            look_dir: [1.0, 0.0].into(),
            radius: 15.0,
            tasks: NpcTasks {
                current_action: None,
                queue: std::collections::VecDeque::new(),
            },
        }
    }

    pub fn set_look_dir(mut self, look_dir: Vector) -> Self {
        self.look_dir = look_dir;
        self
    }

    pub fn update_position(&mut self, cells: &mut Cells, new_pos: Point) -> bool {
        // TODO: check the new position first before moving? Might not be needed if we're doing
        // pathfinding system rather than collision system
        cells.update_position(&new_pos, &self.id);
        self.pos = new_pos;
        true
    }

    pub fn get_position(&self) -> &Point {
        &self.pos
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

    pub fn act(&mut self, cells: &mut cell_map::Cells, dt: &f64) {
        if let Some(action) = &self.tasks.current_action {
            match action {
                Action::Moving => self.move_npc(cells, dt),
            }
            return;
        }

        self.setup_next_task();
    }

    pub fn setup_next_task(&mut self) {
        let Some(current_task) = self.tasks.queue.pop_front() else {
            return;
        };
        match &current_task.task_type {
            TaskType::Move(target_point) => self.target_move(&target_point),
        }
    }

    pub fn queue_task(&mut self, task: Task) {
        self.tasks.queue.push_back(task);
    }

    fn move_npc(&mut self, cells: &mut cell_map::Cells, dt: &f64) {
        if let Some(movement_target) = &self.knowledge.movement_target {
            // update the position
        }
    }

    fn target_move(&mut self, target_point: &Point) {
        self.knowledge.movement_target = Some(target_point.to_owned());
    }

}

#[derive(Clone)]
pub struct Task {
    task_type: TaskType,
}

#[derive(Clone)]
pub enum TaskType {
    Move(Point),
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
        let circum = npc.radius * 2.0;
        // Render npc circle
        graphics::Ellipse::new_border(npc_colour, 0.5)
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
