use crate::bug::{Bug, Pos};
use crate::constants::*;
use crate::gene::{Chromosome, Ethnicity, Gene};
use crate::rng::DeterministicRng;
use crate::world::{World, WorldStats};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub max_ticks: Option<i32>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            max_ticks: None,
        }
    }
}

/// Main simulation engine
#[derive(Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub world: World,
    pub rng: DeterministicRng,
    pub config: SimConfig,

    // Dynamic parameters (from original bugs.c)
    pub food_hump: f64,
    pub safety: i32,
    pub leak: i32,
    pub force_mate: u8,
    pub cost_mate: i32,
    pub drink_or_die: i32,
    pub age_div: i32,
    pub target_pop: usize,

    // Statistics
    pub stats_history: VecDeque<WorldStats>,
}

impl Simulation {
    pub fn new(config: SimConfig) -> Self {
        let mut sim = Self {
            world: World::new(),
            rng: DeterministicRng::new(config.seed),
            config,
            food_hump: 1.0,
            safety: 1,
            leak: 1,
            force_mate: 0,
            cost_mate: COST_MATE_INITIAL,
            drink_or_die: 0,
            age_div: 0,
            target_pop: POP_TARGET,
            stats_history: VecDeque::with_capacity(L_HIST),
        };

        sim.init_world();
        sim
    }

    /// Initialize world with terrain and initial bug
    fn init_world(&mut self) {
        // Initialize terrain
        for x in 0..WORLD_X {
            for y in 0..WORLD_Y {
                let pos = Pos::new(x as i32, y as i32);
                if let Some(cell) = self.world.get_cell_mut(pos) {
                    // Simple terrain - can be made more complex
                    cell.food = FOOD_START;
                    cell.water = 0;
                    cell.terrain_height = 0;
                }
            }
        }

        // Create "bug one" - the initial bug
        self.create_bug_one();
    }

    /// Create the initial bug with simple genes
    fn create_bug_one(&mut self) {
        let start_pos = Pos::new((WORLD_X / 2) as i32, (WORLD_Y / 2) as i32);
        let mut bug = Bug::new(0, start_pos, self.world.current_tick);

        // Generate unique ethnicity
        let uid = self.rng.gen_u64();
        bug.brain.ethnicity = Ethnicity::new(
            uid,
            self.rng.gen_range(256) as u8,
            self.rng.gen_range(256) as u8,
            self.rng.gen_range(256) as u8,
        );

        // Create simple initial genes for each decision
        for i in 0..N_DECISIONS {
            let mut genes_a = Vec::new();
            let mut genes_b = Vec::new();

            match i {
                ACT_EAT => {
                    // Eat when food is available
                    genes_a.push(Gene::new_sense(ITEM_FOOD));
                    genes_b.push(Gene::new_constant(100));
                }
                ACT_MOVE => {
                    // Move when low on food
                    genes_a.push(Gene::new_sense(SENSE_SELF + ACT_EAT));
                    genes_b.push(Gene::new_constant(50));
                }
                ACT_DIVIDE => {
                    // Divide when have enough mass
                    genes_a.push(Gene::new_sense(SENSE_SELF + ACT_EAT));
                    genes_a.push(Gene::new_compare(SENSE_SELF + ACT_EAT, 500));
                    genes_b.push(Gene::new_constant(10));
                }
                _ => {
                    // Default small weight
                    genes_a.push(Gene::new_constant(10));
                    genes_b.push(Gene::new_constant(10));
                }
            }

            bug.brain.decisions[i] = (
                Chromosome::with_genes(genes_a, bug.brain.ethnicity),
                Chromosome::with_genes(genes_b, bug.brain.ethnicity),
            );
        }

        bug.brain.update_gene_count();
        self.world.add_bug(bug);
    }

    /// Run one simulation tick
    pub fn step(&mut self) -> bool {
        self.world.current_tick += 1;

        // Update dynamic parameters based on population and time
        self.update_dynamics();

        // Process all bugs
        self.process_bugs();

        // Grow food
        self.grow_food();

        // Record stats
        let stats = self.world.stats();
        if self.stats_history.len() >= L_HIST {
            self.stats_history.pop_front();
        }
        self.stats_history.push_back(stats);

        // Check if simulation should continue
        if let Some(max_ticks) = self.config.max_ticks {
            if self.world.current_tick >= max_ticks {
                return false;
            }
        }

        self.world.bug_count() > 0
    }

    /// Update dynamic challenge parameters
    fn update_dynamics(&mut self) {
        let tick = self.world.current_tick;
        let pop = self.world.bug_count();

        // Progressive challenges (from original bugs.c)
        if tick == 3000 {
            self.force_mate = 0x10;
        }
        if tick == 4000 {
            self.force_mate = 0x30;
        }
        if tick == 5000 {
            self.force_mate = 0x70;
        }
        if tick == 6000 {
            self.force_mate = 0x71;
        }
        if tick == 7000 {
            self.force_mate = 0x73;
        }
        if tick == 8000 {
            self.force_mate = 0x77;
        }
        if tick == 9000 {
            self.force_mate = 0x7F;
        }

        if tick == 10000 {
            self.cost_mate = 24;
        }
        if tick == 11000 {
            self.cost_mate = 48;
        }
        if tick == 12000 {
            self.cost_mate = 96;
        }
        if tick == 13000 {
            self.cost_mate = 144;
        }

        if tick == 14000 {
            self.drink_or_die = 3;
        }
        if tick == 15000 {
            self.drink_or_die = 2;
        }
        if tick == 16000 {
            self.drink_or_die = 1;
        }

        // Dynamic food regulation
        if tick > 3000 && tick > SEASON_LENGTH {
            if tick % 32 == 0 {
                if self.age_div < 30 {
                    self.food_hump *= 1.001;
                }
                if self.age_div > 300 {
                    self.food_hump /= 1.001;
                }
            }

            if tick % 8 == 0 {
                self.age_div += 1;
            }
        }

        // Population control
        if pop < self.target_pop && self.age_div > 0 {
            self.age_div -= 1;
        }

        if pop > POP_HARD_LIMIT {
            // Find oldest bug's birthday
            if let Some(oldest) = self.world.bugs.values().min_by_key(|b| b.data.birthday) {
                self.age_div = tick - oldest.data.birthday;
            }
        }
    }

    /// Process all bugs in deterministic order
    fn process_bugs(&mut self) {
        // Get sorted bug IDs for determinism
        let mut bug_ids: Vec<u64> = self.world.bugs.keys().copied().collect();
        bug_ids.sort_unstable();

        for bug_id in bug_ids {
            self.process_single_bug(bug_id);
        }
    }

    /// Process one bug's decision and action
    fn process_single_bug(&mut self, bug_id: u64) {
        // Gather senses
        let senses = self.gather_senses(bug_id);

        // Evaluate all decisions
        let mut weights = vec![0.0; N_DECISIONS];
        if let Some(bug) = self.world.get_bug(bug_id) {
            for i in 0..N_DECISIONS {
                weights[i] = bug.brain.evaluate_decision(i, &senses);
            }
        } else {
            return;
        }

        // Find action with highest weight
        let action = weights
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(ACT_SLEEP);

        // Execute action
        self.execute_action(bug_id, action);
    }

    /// Gather sense data for a bug
    fn gather_senses(&self, bug_id: u64) -> Vec<i32> {
        let mut senses = vec![0; N_SENSES];

        let Some(bug) = self.world.get_bug(bug_id) else {
            return senses;
        };

        let pos = bug.current_state.pos;
        let facing = bug.current_state.facing;

        // Sense surrounding cells
        let directions = [DIR_E, DIR_SE, DIR_SW, DIR_W, DIR_NW, DIR_NE];
        for (i, &dir) in directions.iter().enumerate() {
            let sense_pos = pos.step(dir);
            let base_idx = i * 4;

            if let Some(cell) = self.world.get_cell(sense_pos) {
                senses[base_idx + ITEM_FOOD] = cell.food;
            }

            if let Some(other_bug) = self.world.get_bug_at(sense_pos) {
                senses[base_idx + ITEM_BUG] = other_bug.current_state.weight / 1024;
                senses[base_idx + ITEM_BUG_FACE] =
                    (((other_bug.current_state.facing - facing) as i32 + 6) % 6) as i32;
                // TODO: genetic match calculation
                senses[base_idx + ITEM_BUG_MATCH] = 0;
            }
        }

        // Self senses
        senses[SELF_AGE] = self.world.current_tick - bug.data.birthday;
        senses[THIRST_SENSE] = bug.current_state.hydrate;

        // Action history (simplified)
        for i in 0..N_ACTIONS {
            senses[SENSE_SELF + i] = 0; // TODO: implement action history
        }

        senses
    }

    /// Execute a bug action
    fn execute_action(&mut self, bug_id: u64, action: usize) {
        match action {
            ACT_SLEEP => self.action_sleep(bug_id),
            ACT_EAT => self.action_eat(bug_id),
            ACT_TURN_CW => self.action_turn(bug_id, 1),
            ACT_TURN_CCW => self.action_turn(bug_id, -1),
            ACT_MOVE => self.action_move(bug_id),
            ACT_MATE => self.action_mate(bug_id),
            ACT_DIVIDE => self.action_divide(bug_id),
            _ => {}
        }
    }

    fn action_sleep(&mut self, bug_id: u64) {
        if let Some(bug) = self.world.get_bug_mut(bug_id) {
            bug.current_state.weight -= COST_SLEEP;
            bug.current_state.action = ACT_SLEEP;
        }
    }

    fn action_eat(&mut self, bug_id: u64) {
        let bug_pos = self.world.get_bug(bug_id).map(|b| b.current_state.pos);
        if let Some(pos) = bug_pos {
            let food_available = self
                .world
                .get_cell(pos)
                .map(|c| c.food)
                .unwrap_or(0)
                .min(100);

            if let Some(cell) = self.world.get_cell_mut(pos) {
                cell.food -= food_available;
            }

            if let Some(bug) = self.world.get_bug_mut(bug_id) {
                bug.current_state.weight += food_available;
                bug.data.food_consumed += food_available;
                bug.current_state.action = ACT_EAT;
            }
        }
    }

    fn action_turn(&mut self, bug_id: u64, direction: i8) {
        if let Some(bug) = self.world.get_bug_mut(bug_id) {
            bug.current_state.facing = (bug.current_state.facing + direction + 6) % 6 - 2;
            bug.current_state.weight -= COST_TURN;
            bug.current_state.action = if direction > 0 {
                ACT_TURN_CW
            } else {
                ACT_TURN_CCW
            };
        }
    }

    fn action_move(&mut self, bug_id: u64) {
        let (pos, facing) = {
            let bug = match self.world.get_bug(bug_id) {
                Some(b) => b,
                None => return,
            };
            (bug.current_state.pos, bug.current_state.facing)
        };

        let new_pos = pos.step(facing);

        if self.world.move_bug(bug_id, new_pos) {
            if let Some(bug) = self.world.get_bug_mut(bug_id) {
                bug.current_state.weight -= COST_MOVE;
                bug.data.moves += 1;
                bug.current_state.action = ACT_MOVE;
                bug.record_position();
            }
        }
    }

    fn action_mate(&mut self, _bug_id: u64) {
        // TODO: Implement mating logic
    }

    fn action_divide(&mut self, _bug_id: u64) {
        // TODO: Implement division logic with mutations
    }

    /// Grow food in all cells
    fn grow_food(&mut self) {
        for x in 0..WORLD_X {
            for y in 0..WORLD_Y {
                let pos = Pos::new(x as i32, y as i32);

                // Don't grow food where bugs are (if leak is off)
                if self.leak == 0 && self.world.get_bug_at(pos).is_some() {
                    continue;
                }

                if let Some(cell) = self.world.get_cell_mut(pos) {
                    let growth = (FOOD_SPREAD as f64 * self.food_hump) as i32;
                    cell.food = (cell.food + growth).min(FOOD_CAP);
                }
            }
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> WorldStats {
        self.world.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(SimConfig::default());
        assert_eq!(sim.world.bug_count(), 1);
    }

    #[test]
    fn test_simulation_step() {
        let mut sim = Simulation::new(SimConfig::default());
        let initial_tick = sim.world.current_tick;
        sim.step();
        assert_eq!(sim.world.current_tick, initial_tick + 1);
    }

    #[test]
    fn test_determinism() {
        let config = SimConfig {
            seed: 12345,
            max_ticks: Some(100),
        };

        let mut sim1 = Simulation::new(config.clone());
        let mut sim2 = Simulation::new(config);

        for _ in 0..100 {
            sim1.step();
            sim2.step();
        }

        assert_eq!(sim1.world.bug_count(), sim2.world.bug_count());
        assert_eq!(sim1.world.total_food(), sim2.world.total_food());
    }
}
