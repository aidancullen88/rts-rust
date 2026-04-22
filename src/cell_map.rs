use std::collections::HashMap;
use std::collections::HashSet;

use crate::{npc::Npc, point::Point};

type Id = usize;

#[derive(Eq, PartialEq, Hash, Clone)]
struct Cell {
    x: usize,
    y: usize,
}

pub struct Cells {
    cells: HashMap<Cell, HashSet<Id>>,
    items: HashMap<Id, Cell>,
}

impl Cells {
    pub fn new() -> Self {
        Cells {
            cells: HashMap::new(),
            items: HashMap::new(),
        }
    }
    fn insert_to_cell(&mut self, cell: Cell, item_id: Id) {
        self.cells.entry(cell.clone()).or_default().insert(item_id);
    }
    fn set_item_pos(&mut self, item_id: Id, cell: Cell) {
        self.items.insert(item_id, cell);
    }
    fn remove_from_cell(&mut self, cell: &Cell, item_id: &usize) {
        if let Some(item_set) = self.cells.get_mut(cell) {
            item_set.remove(item_id);
            if item_set.is_empty() {
                self.cells.remove(cell);
            }
        }
    }
    // Updates the position of an item in the cell map (inserts if it didn't already exist)
    pub fn update_position(&mut self, pos: &Point, item_id: usize) {
        let new_cell = calculate_cell_from_pos(pos);
        if let Some(old_cell) = self.items.get(&item_id).cloned() {
            // check if the cell hasn't changed
            if old_cell != new_cell {
                self.remove_from_cell(&old_cell, &item_id);
            } else {
                return;
            }
        };
        self.insert_to_cell(new_cell.clone(), item_id);
        self.set_item_pos(item_id, new_cell);
    }
}

fn calculate_cell_from_pos(pos: &Point) -> Cell {
    const CELL_SIZE: f64 = 100.0;
    if (pos.x < 0.0) || (pos.y < 0.0) {
        return Cell { x: 0, y: 0 };
    }
    Cell {
        x: (pos.x / CELL_SIZE).floor() as usize,
        y: (pos.y / CELL_SIZE).floor() as usize,
    }
}
