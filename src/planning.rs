use crate::{
    npc::{Id, NpcKnowledge, Task, TaskType}, vector::Vector,
};
use std::collections::HashMap;

struct Goal {
    action: Action
}

struct LocationCondition {
    target: Vector,
}

enum Conditions {
    Location(LocationCondition),
}

struct Action {
    action_type: ActionType,
    preconditions: Option<Vec<Goal>>,
    postcondition: Option<Conditions>,
}

#[derive(Eq, Hash, PartialEq)]
enum ActionType {
    MoveTo,
    Idle,
}

fn plan(goal: &Goal, npc_actions: &HashMap<ActionType, Action>, world_state: &NpcKnowledge) -> Action {
    let Some(npc_action) = npc_actions.get(&goal.action.action_type) else {
        return Action {
            action_type: ActionType::Idle,
            preconditions: None,
            postcondition: None,
        }
    };
    // check preconditions by recursing plan? they will return actions or none, which will then
    // form the action chain for satisfying the plan
    let tasks_queue = match npc_action.preconditions {
        None if npc_action.postcondition.is_none() => {
            vec![]
        },
        None => {
            let new_task = Task::new(TaskType::Move(()))
            vec![]
        }
    }
}
