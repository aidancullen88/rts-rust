use std::collections::VecDeque;

use crate::npc::Id;

pub enum Event {
    Instant(EventType)
}

pub enum EventType {
    Shot(f64),
}

pub struct EventQueue(VecDeque<(Id, Event)>);

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue(VecDeque::new())
    }
    
    pub fn add_event(&mut self, id: Id, event: Event) {
        self.0.push_back((id, event));
    }

    pub fn get_next_event(&mut self) -> Option<(Id, Event)> {
        self.0.pop_front()
    }
}
