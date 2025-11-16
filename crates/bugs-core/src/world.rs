use serde::{Deserialize, Serialize};
use crate::bug::{Bug, Pos};
use crate::constants::*;
use std::collections::HashMap;

/// World cell data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub food: i32,
    pub water: i32,
    pub terrain_height: i32,
    pub nearest: i32,  // Distance to nearest bug (-1 if none)
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            food: 0,
            water: 0,
            terrain_height: 0,
            nearest: -1,
        }
    }
}

/// World state containing terrain and all bugs
#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    /// Grid of cells
    pub cells: Vec<Vec<Cell>>,

    /// All bugs indexed by ID
    pub bugs: HashMap<u64, Bug>,

    /// Spatial index: position -> bug ID
    pub bug_positions: HashMap<(i32, i32), u64>,

    /// Next bug ID to assign
    pub next_bug_id: u64,

    /// Current simulation tick
    pub current_tick: i32,
}

impl World {
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(WORLD_X);
        for _ in 0..WORLD_X {
            let mut col = Vec::with_capacity(WORLD_Y);
            for _ in 0..WORLD_Y {
                col.push(Cell::default());
            }
            cells.push(col);
        }

        Self {
            cells,
            bugs: HashMap::new(),
            bug_positions: HashMap::new(),
            next_bug_id: 1,
            current_tick: 0,
        }
    }

    /// Get cell at position
    pub fn get_cell(&self, pos: Pos) -> Option<&Cell> {
        let pos = pos.wrap();
        self.cells
            .get(pos.x as usize)
            .and_then(|col| col.get(pos.y as usize))
    }

    /// Get mutable cell at position
    pub fn get_cell_mut(&mut self, pos: Pos) -> Option<&mut Cell> {
        let pos = pos.wrap();
        self.cells
            .get_mut(pos.x as usize)
            .and_then(|col| col.get_mut(pos.y as usize))
    }

    /// Get bug at position
    pub fn get_bug_at(&self, pos: Pos) -> Option<&Bug> {
        let pos = pos.wrap();
        self.bug_positions
            .get(&(pos.x, pos.y))
            .and_then(|id| self.bugs.get(id))
    }

    /// Get mutable bug at position
    pub fn get_bug_at_mut(&mut self, pos: Pos) -> Option<&mut Bug> {
        let pos = pos.wrap();
        let id = *self.bug_positions.get(&(pos.x, pos.y))?;
        self.bugs.get_mut(&id)
    }

    /// Get bug by ID
    pub fn get_bug(&self, id: u64) -> Option<&Bug> {
        self.bugs.get(&id)
    }

    /// Get mutable bug by ID
    pub fn get_bug_mut(&mut self, id: u64) -> Option<&mut Bug> {
        self.bugs.get_mut(&id)
    }

    /// Add a bug to the world
    pub fn add_bug(&mut self, mut bug: Bug) -> u64 {
        let id = self.next_bug_id;
        self.next_bug_id += 1;

        bug.id = id;
        let pos = bug.current_state.pos.wrap();

        self.bug_positions.insert((pos.x, pos.y), id);
        self.bugs.insert(id, bug);

        id
    }

    /// Remove a bug from the world
    pub fn remove_bug(&mut self, id: u64) -> Option<Bug> {
        if let Some(bug) = self.bugs.remove(&id) {
            let pos = bug.current_state.pos.wrap();
            self.bug_positions.remove(&(pos.x, pos.y));
            Some(bug)
        } else {
            None
        }
    }

    /// Move a bug to a new position
    pub fn move_bug(&mut self, id: u64, new_pos: Pos) -> bool {
        let new_pos = new_pos.wrap();

        // Check if destination is occupied
        if self.bug_positions.contains_key(&(new_pos.x, new_pos.y)) {
            return false;
        }

        if let Some(bug) = self.bugs.get_mut(&id) {
            let old_pos = bug.current_state.pos.wrap();
            self.bug_positions.remove(&(old_pos.x, old_pos.y));

            bug.current_state.pos = new_pos;
            self.bug_positions.insert((new_pos.x, new_pos.y), id);

            true
        } else {
            false
        }
    }

    /// Get total bug count
    pub fn bug_count(&self) -> usize {
        self.bugs.len()
    }

    /// Calculate total food in world
    pub fn total_food(&self) -> i64 {
        let mut total = 0i64;
        for col in &self.cells {
            for cell in col {
                total += cell.food as i64;
            }
        }
        total
    }

    /// Calculate total bug mass
    pub fn total_bug_mass(&self) -> i64 {
        self.bugs.values().map(|b| b.current_state.weight as i64).sum()
    }

    /// Get statistics (event counters should be set by simulation)
    pub fn stats(&self) -> WorldStats {
        let bug_count = self.bugs.len();

        if bug_count == 0 {
            return WorldStats::default();
        }

        let total_food = self.total_food();
        let total_mass = self.total_bug_mass();
        let total_genes: u32 = self.bugs.values().map(|b| b.brain.n_genes as u32).sum();

        WorldStats {
            tick: self.current_tick,
            bug_count,
            total_food,
            total_bug_mass: total_mass,
            avg_bug_mass: total_mass / bug_count as i64,
            avg_genes: total_genes as f64 / bug_count as f64,
            avg_food_per_cell: (total_food / (WORLD_X * WORLD_Y) as i64) as i32,

            // Event counters initialized to 0, should be set by simulation
            births: 0,
            starvations: 0,
            collisions: 0,
            drownings: 0,
            movements: 0,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// World statistics snapshot
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WorldStats {
    pub tick: i32,
    pub bug_count: usize,
    pub total_food: i64,
    pub total_bug_mass: i64,
    pub avg_bug_mass: i64,
    pub avg_genes: f64,
    pub avg_food_per_cell: i32,

    // Event counters for this tick
    pub births: u32,
    pub starvations: u32,
    pub collisions: u32,
    pub drownings: u32,
    pub movements: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.bug_count(), 0);
        assert_eq!(world.current_tick, 0);
    }

    #[test]
    fn test_add_remove_bug() {
        let mut world = World::new();
        let bug = Bug::new(0, Pos::new(10, 10), 0);
        let id = world.add_bug(bug);

        assert_eq!(world.bug_count(), 1);
        assert!(world.get_bug(id).is_some());

        world.remove_bug(id);
        assert_eq!(world.bug_count(), 0);
    }

    #[test]
    fn test_move_bug() {
        let mut world = World::new();
        let bug = Bug::new(0, Pos::new(10, 10), 0);
        let id = world.add_bug(bug);

        assert!(world.move_bug(id, Pos::new(11, 10)));
        assert_eq!(world.get_bug(id).unwrap().current_state.pos, Pos::new(11, 10));
    }
}
