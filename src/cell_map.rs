use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;

use crate::npc::NpcAttributes;
use crate::npc::NpcStatus;
use crate::npc::{Id, Npc, NpcMap};
use crate::point;
use crate::point::Point;
use crate::point::get_distance_between_points;
use crate::vector::Quad;
use crate::vector::Vector;
use crate::vector::check_ray_collides_circle;
use crate::vector::get_direction_between_points;
use crate::vector::get_vector_quad;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
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
#[derive(Debug)]
pub struct Cells {
    cells: HashMap<CellPos, HashSet<Id>>,
    cell_size: f64,
}

impl Cells {
    pub fn new(cell_size: f64) -> Self {
        Cells {
            cells: HashMap::new(),
            cell_size,
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
    // PRIVATE METHODS

    /// Given a CellPos (i.e. a pair of x-y co-ords on the cell grid), update the hashset at that cell
    /// to contain the id of the item
    fn insert_to_cell(&mut self, cell: CellPos, item_id: &Id) {
        // Get the current hashset for this cell, or create the default if it doesn't exist
        self.cells.entry(cell).or_default().insert(*item_id);
    }
    /// Remove an item from the given cell. If the cell is now empty, remove the whole hashmap
    /// entry
    pub fn remove_from_cell(&mut self, cell: &CellPos, item_id: &Id) {
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
    pub fn update_position(
        &mut self,
        new_pos: &Point,
        old_cell: &CellPos,
        item_id: &Id,
    ) -> Option<CellPos> {
        let new_cell = self.calculate_cell_from_pos(new_pos);
        // check if the cell hasn't changed
        if old_cell != &new_cell {
            self.remove_from_cell(old_cell, item_id);
        } else {
            return None;
        };
        self.insert_to_cell(new_cell.clone(), item_id);
        Some(new_cell)
    }

    pub fn register_initial_position(&mut self, pos: &Point, item_id: &Id) -> CellPos {
        let init_cell = self.calculate_cell_from_pos(pos);
        self.insert_to_cell(init_cell.clone(), item_id);
        init_cell
    }

    pub fn check_if_npc_target_collides_with_npc(
        &self,
        target_pos: &Point,
        npc_info: &HashMap<Id, NpcAttributes>,
        npc_radius: f64,
    ) -> Option<Id> {
        let target_cell = self.calculate_cell_from_pos(target_pos);
        self.get_adjacent_entities(&target_cell)
            // Immediately return if there's nothing adjacent
            .and_then(|entity_list| {
                entity_list
                    .iter()
                    // Find the first npc that is collided with
                    .find(|e| {
                        let current_npc_info = &npc_info
                            .get(e)
                            .expect("Npc shouldn't be missing from info map");
                        point::is_point_distance_leq(
                            &current_npc_info.position,
                            target_pos,
                            npc_radius + current_npc_info.radius,
                        )
                    })
                    .copied()
            })
    }

    pub fn check_if_point_target_collides_with_npc(
        &self,
        target_pos: &Point,
        npc_info: &HashMap<Id, NpcAttributes>,
    ) -> Option<Id> {
        let target_cell = self.calculate_cell_from_pos(target_pos);
        self.get_adjacent_entities(&target_cell)
            .and_then(|entity_list| {
                entity_list
                    .iter()
                    // Find the first npc that the target collides with
                    .find(|e| {
                        let current_npc_info = &npc_info
                            .get(e)
                            .expect("Npc shouldn't be missing from info map");
                        point::is_point_distance_leq(
                            &current_npc_info.position,
                            target_pos,
                            current_npc_info.radius,
                        )
                    })
                    .copied()
            })
    }

    pub fn check_if_ray_collides_with_npc<'a>(
        &self,
        origin: &Point,
        direction: &Vector,
        npc_info: &'a HashMap<Id, NpcAttributes>,
        current_npc: &Npc,
    ) -> Option<(&Id, Point, &'a Point)> {
        // Get the start cell, figure out the quadrant direction of the ray, and then get all the
        // cells for that quad rather than all of them

        let ray_quad = get_vector_quad(direction)?;
        let start_cell = self.calculate_cell_from_pos(origin);
        // Check what quad we're in and filter the cells based on this e.g. for LeftTop, cellpos
        // should be >= both x and y of the current cell
        self.cells
            .iter()
            .filter(|(cell_pos, _)| match ray_quad {
                Quad::RightDown => cell_pos.0 >= start_cell.0 && cell_pos.1 >= start_cell.1,
                Quad::LeftDown => cell_pos.0 <= start_cell.0 && cell_pos.1 >= start_cell.1,
                Quad::LeftUp => cell_pos.0 <= start_cell.0 && cell_pos.1 <= start_cell.1,
                Quad::RightUp => cell_pos.0 >= start_cell.0 && cell_pos.1 <= start_cell.1,
            })
            // Flatten the list of npcs from the cells into a list of ids
            .flat_map(|(_, ids)| {
                ids.iter()
            })
            // Get the npc_info for each id and add it to the list
            .map(|id| {
                (
                    id,
                    npc_info
                        .get(id)
                        .expect("npc in cell map should exist in npc_info"),
                )
            })
            // Filter the list by some pre-conditions, and finally the collision check with the ray
            .filter_map(|(id, npc_info)| {
                if *id == current_npc.get_id()
                    || &npc_info.team == current_npc.get_team()
                    // Don't collide with dead npcs
                    || matches!(npc_info.status, NpcStatus::Dead) {
                        return None;
                };
                check_ray_collides_circle(
                        origin,
                        direction,
                        &npc_info.position,
                        npc_info.radius,
                    ).map(|point| (id, point, &npc_info.position))
            })
            // Get the first npc the ray collides with
            .next()
    }

    // fn line_collides_with_cell(&self, cell: &CellPos, origin: &Point, direction: &Vector) -> bool {
    //     let (x, y) = (cell.0 as f64 * self.cell_size, cell.1 as f64 * self.cell_size);
    //     let sides = [
    //         // Left
    //         (Point::new(x, y), Point::new(x, y + self.cell_size)),
    //         // Top
    //         (Point::new(x, y), Point::new(x + self.cell_size, y)),
    //         // Bottom
    //         (Point::new(x, y + self.cell_size), Point::new(x + self.cell_size, y + self.cell_size)),
    //         // Right
    //         (Point::new(x + self.cell_size, y), Point::new(x + self.cell_size, y + self.cell_size)),
    //     ];
    //     sides.iter().filter(|(start, end)| {
    //         let start_angle = get_direction_between_points(origin, start);
    //         let end_angle = get_direction_between_points(origin, end);
    //         vector_is_between(direction, &start_angle, &end_angle)
    //     }).count() == 2
    //     // go through sides, check if 2 collide with the line
    //     // maybe just iterate without making the array above?
    // }

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
