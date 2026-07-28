use crate::{
    npc::{Id, Npc, NpcKnowledge, Task, TaskType}, point::{Point, is_point_distance_leq}, vector::Vector
};
use std::collections::HashMap;

struct Goal {
    preconditions: Vec<(Condition, Option<Goal>)>,
    action: Action,
    postcondition: Condition
}

enum Condition {
    Location(Point),
}

struct Action {
    action_type: ActionType,
    preconditions: Vec<Goal>,
}

#[derive(Eq, Hash, PartialEq)]
enum ActionType {
    MoveTo,
    Idle,
}

enum PreConditionResult {
    Passed,
    Failed,
    Continue(Vec<Goal>)
}

fn plan(goal: &Goal, npc_actions: &HashMap<ActionType, Action>, npc_knowledge: &NpcKnowledge) {
    // for each goal:
    // check preconditions, recursing through to bottom level
    // if successful, should have a list of actions in order
    // if not (no matching actions or pre-condition fails), move on to next goal
}

// three conditions:
// all preconditions met/no preconditions
// 1 or more preconditions with action fails: need list of child goals to check
// precondition without goal fails: entire plan fails, move to next top level goal

fn check_goal_preconditions(preconditions: &[(Condition, Option<Action>)], npc: &Npc) -> PreConditionResult {
    // for each precondition:
    // match against type
    // do the comparison against the npc knowledge
    // if passes: return none
    // if not: add the action to the list
    if preconditions.is_empty() { return PreConditionResult::Passed; };
    let result: Vec<Goal> = preconditions.iter().filter_map(|(condition, subgoal)| {
        match condition {
            Condition::Location(loc) => {
                if is_point_distance_leq(loc, npc.get_position(), 20.0) {
                    return None;
                } else {
                    return Some(subgoal);
                }
            }
        }
    }).collect();
    if result.is_empty()
}
