use std::collections::HashMap;
use std::collections::HashSet;

use crate::Npcs;
use crate::npc::Id;
use crate::point;
use crate::point::Point;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CellPos(pub u32, pub u32);

/// Holds the cell map, item map, and cell size
/// CELL MAP:
/// This represents a "sparse" 2D grid of cells, where each cell is a position (CellPos) +
/// HashSet<Id> pair. If there is nothing in a cell, then it shouldn't exist in the map (and a get
/// will return None).
///
/// ITEM MAP:
/// This is a map of every item in the cell map, and which cell they are currently in. This allows
/// quick lookup of an item's position without a. iterating the entire cell map b. storing the cell
/// pos externally (e.g. on the item itself), which can easily lead to desync
///
/// CELL SIZE: The size of the cells in pixels. This should be kept relatively small, maybe 2 x the
/// biggest collider in the map
pub struct Cells {
    cells: HashMap<CellPos, HashSet<Id>>,
    items: HashMap<Id, CellPos>,
    cell_size: f64,
}

impl Cells {
    pub fn new(cell_size: f64) -> Self {
        Cells {
            cells: HashMap::new(),
            items: HashMap::new(),
            cell_size: cell_size,
        }
    }
    // PRIVATE METHODS

    /// Given a CellPos (i.e. a pair of x-y co-ords on the cell grid), update the hashset at that cell
    /// to contain the id of the item
    fn insert_to_cell(&mut self, cell: CellPos, item_id: &Id) {
        // Get the current hashset for this cell, or create the default if it doesn't exist
        self.cells.entry(cell).or_default().insert(*item_id);
    }
    /// Update the item list to contain the item id and the position. This is used to quickly check
    /// where an item is in the cell map, given its id
    fn set_item_pos(&mut self, item_id: &Id, cell: CellPos) {
        self.items.insert(*item_id, cell);
    }
    /// Remove an item from the given cell. If the cell is now empty, remove the whole hashmap
    /// entry
    fn remove_from_cell(&mut self, cell: &CellPos, item_id: &Id) {
        if let Some(item_set) = self.cells.get_mut(cell) {
            item_set.remove(item_id);
            // If there's nothing left in the hashset, remove the whole hashmap entry. Keeps the size of
            // the hashmap down as items join and leave
            if item_set.is_empty() {
                self.cells.remove(cell);
            }
        }
    }

    fn get_adjacent_entities(&self, target_cell: &CellPos) -> Option<Vec<Id>> {
        // Given a cell (e.g. the x and y int co-ordinates of a cell on a grid, returns either a
        // vec of the potentially colliding entity ids, or None
        let mut out = Vec::with_capacity(20);
        let (x, y) = (target_cell.0, target_cell.1);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let x_coord = x as i32 + dx;
                let y_coord = y as i32 + dy;

                if x_coord < 0 || y_coord < 0 {
                    continue;
                }

                if let Some(id_set) = self.cells.get(&CellPos(x_coord as u32, y_coord as u32)) {
                    out.extend(id_set.iter().copied());
                }
            }
        }
        (!out.is_empty()).then_some(out)
    }

    // PUBLIC METHODS

    /// Updates the position of an item in the cell map, inserting if it didn't already exist. Also
    /// handles updating the item map. This is the main function that is called when an item moves.
    pub fn update_position(&mut self, pos: &Point, item_id: &Id) {
        let new_cell = self.calculate_cell_from_pos(pos);
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

    pub fn check_if_target_collides_with_npc(&self, target_pos: &Point, npcs: &Npcs) -> Option<Id> {
        let target_cell = self.calculate_cell_from_pos(target_pos);
        self.get_adjacent_entities(&target_cell)
            .and_then(|entity_list| {
                println!("npc list: {:?}", entity_list);
                entity_list.iter().filter(|e| {
                    point::check_distance_between_points(
                        npcs.get_npc_by_id(&e).unwrap().get_position(),
                        target_pos,
                        &30.0,
                    )
                // Get the first from the list if there is one, or else None
                }).next().copied()
            })
    }

    /// Given a CellPos, get the corresponding hashset of IDs for the cell (or None if the cell
    /// does not exist/isn't init'd
    pub fn get_cell_values(&self, cell: &CellPos) -> Option<&HashSet<Id>> {
        self.cells.get(cell)
    }

    /// Returns the currently set cell size
    pub fn get_cell_size(&self) -> &f64 {
        &self.cell_size
    }

    /// Given a position and the set CELL_SIZE, calculate the CellPos that the position would be in
    pub fn calculate_cell_from_pos(&self, pos: &Point) -> CellPos {
        if (pos.x < 0.0) || (pos.y < 0.0) {
            return CellPos(0, 0);
        }
        CellPos(
            (pos.x / self.cell_size).floor() as u32,
            (pos.y / self.cell_size).floor() as u32,
        )
    }
}
