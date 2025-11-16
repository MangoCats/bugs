use crate::bug::{Bug, BugBrain, Pos};
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

        // Add some variety to terrain
        self.init_terrain();

        // Create "bug one" - the initial bug
        self.create_bug_one();
    }

    /// Initialize terrain with basic features
    fn init_terrain(&mut self) {
        // Simple cosine-based terrain
        for x in 0..WORLD_X {
            for y in 0..WORLD_Y {
                let pos = Pos::new(x as i32, y as i32);
                if let Some(cell) = self.world.get_cell_mut(pos) {
                    // Create gentle height variations
                    let height = ((x as f64 * 0.01).cos() + (y as f64 * 0.01).sin()) * 512.0;
                    cell.terrain_height = height as i32;

                    // Add water to low areas
                    if cell.terrain_height < -INIT_DEPTH {
                        cell.water = (-cell.terrain_height).min(DROWN_DEPTH * 2);
                    }
                }
            }
        }
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
                ACT_SLEEP => {
                    // Sleep rarely
                    genes_a.push(Gene::new_constant(5));
                    genes_b.push(Gene::new_constant(5));
                }
                ACT_EAT => {
                    // Eat when food is available
                    genes_a.push(Gene::new_sense(ITEM_FOOD));
                    genes_b.push(Gene::new_constant(100));
                }
                ACT_TURN_CW | ACT_TURN_CCW => {
                    // Turn occasionally
                    genes_a.push(Gene::new_constant(20));
                    genes_b.push(Gene::new_constant(20));
                }
                ACT_MOVE => {
                    // Move when hungry or randomly
                    genes_a.push(Gene::new_constant(40));
                    genes_b.push(Gene::new_constant(40));
                }
                ACT_MATE => {
                    // Mate occasionally
                    genes_a.push(Gene::new_constant(15));
                    genes_b.push(Gene::new_constant(15));
                }
                ACT_DIVIDE => {
                    // Divide when well-fed - very high priority
                    let mut gene = Gene::new_constant(1000);
                    gene.prod_index = None;
                    gene.sum_index = None;
                    genes_a.push(gene);
                    genes_b.push(Gene::new_constant(1000));
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

        // Track bugs to remove (dead bugs)
        let mut dead_bugs = Vec::new();

        for bug_id in bug_ids {
            self.process_single_bug(bug_id);

            // Check if bug died
            if let Some(bug) = self.world.get_bug(bug_id) {
                // Check starvation
                if bug.current_state.weight <= 0 {
                    dead_bugs.push(bug_id);
                }
            }
        }

        // Remove dead bugs
        for bug_id in dead_bugs {
            self.world.remove_bug(bug_id);
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

    fn action_mate(&mut self, bug_id: u64) {
        // Get bug position and facing
        let (pos, facing) = {
            let bug = match self.world.get_bug(bug_id) {
                Some(b) => b,
                None => return,
            };
            (bug.current_state.pos, bug.current_state.facing)
        };

        let target_pos = pos.step(facing);

        // Check if there's another bug to mate with
        if let Some(mate_id) = self.world.bug_positions.get(&(target_pos.x, target_pos.y)).copied() {
            if mate_id != bug_id {
                // Exchange genetic material
                self.mate_bugs(bug_id, mate_id);
            }
        }

        // Pay the cost
        if let Some(bug) = self.world.get_bug_mut(bug_id) {
            bug.current_state.weight -= self.cost_mate;
            bug.current_state.action = ACT_MATE;
        }
    }

    fn action_divide(&mut self, bug_id: u64) {
        let bug_data = {
            let bug = match self.world.get_bug(bug_id) {
                Some(b) => b,
                None => return,
            };

            // Check if bug has enough mass to divide
            let min_weight = 1024 * bug.brain.divide_count as i32;
            if bug.current_state.weight < min_weight {
                return;
            }

            // Check age requirement if forcemate is active
            let age = self.world.current_tick - bug.data.birthday;
            if (self.force_mate & 0x01) != 0 && age < self.age_div {
                // Penalty for dividing too young
                if let Some(b) = self.world.get_bug_mut(bug_id) {
                    let penalty = (self.force_mate >> 4) as i32 * (self.age_div - age);
                    b.current_state.weight -= penalty;
                }
                return;
            }

            (
                bug.current_state.pos,
                bug.current_state.facing,
                bug.brain.divide_count,
                bug.current_state.weight,
                bug.brain.clone(),
            )
        };

        let (pos, facing, divide_count, weight, brain) = bug_data;

        // Create offspring
        let weight_per_child = weight / (divide_count as i32 + 1);
        let mut offspring_ids = Vec::new();

        for i in 0..divide_count {
            // Calculate position for offspring (behind parent)
            let offset_facing = (facing + 3 + (i as i8 - 1)) % 6;
            let offset_facing = if offset_facing < -2 { offset_facing + 6 } else { offset_facing };
            let child_pos = pos.step(offset_facing).wrap();

            // Check if position is free
            if self.world.get_bug_at(child_pos).is_some() {
                continue;
            }

            // Create child bug
            let mut child = Bug::new(0, child_pos, self.world.current_tick);
            child.current_state.weight = weight_per_child;
            child.current_state.facing = facing;

            // Copy and mutate genes
            child.brain = brain.clone();
            child.brain.generation += 1;
            self.mutate_brain(&mut child.brain);

            let child_id = self.world.add_bug(child);
            offspring_ids.push(child_id);
        }

        // Update parent
        if let Some(parent) = self.world.get_bug_mut(bug_id) {
            parent.current_state.weight = weight_per_child;
            parent.data.children += offspring_ids.len() as u32;
            parent.current_state.action = ACT_DIVIDE;
        }
    }

    /// Mate two bugs by exchanging genetic material
    fn mate_bugs(&mut self, id1: u64, id2: u64) {
        // Get genes from both bugs
        let (brain1, brain2) = {
            let bug1 = match self.world.get_bug(id1) {
                Some(b) => b,
                None => return,
            };
            let bug2 = match self.world.get_bug(id2) {
                Some(b) => b,
                None => return,
            };
            (bug1.brain.clone(), bug2.brain.clone())
        };

        // Randomly exchange chromosomes
        for i in 0..N_DECISIONS {
            if self.rng.gen_bool(0.5) {
                // Swap chromosome A
                if let Some(bug1) = self.world.get_bug_mut(id1) {
                    bug1.brain.decisions[i].0 = brain2.decisions[i].0.clone();
                }
                if let Some(bug2) = self.world.get_bug_mut(id2) {
                    bug2.brain.decisions[i].0 = brain1.decisions[i].0.clone();
                }
            }
            if self.rng.gen_bool(0.5) {
                // Swap chromosome B
                if let Some(bug1) = self.world.get_bug_mut(id1) {
                    bug1.brain.decisions[i].1 = brain2.decisions[i].1.clone();
                }
                if let Some(bug2) = self.world.get_bug_mut(id2) {
                    bug2.brain.decisions[i].1 = brain1.decisions[i].1.clone();
                }
            }
        }

        // Update statistics
        if let Some(bug1) = self.world.get_bug_mut(id1) {
            bug1.data.mate_success += 1;
        }
        if let Some(bug2) = self.world.get_bug_mut(id2) {
            bug2.data.mate_success += 1;
        }
    }

    /// Apply mutations to a brain
    fn mutate_brain(&mut self, brain: &mut BugBrain) {
        // Mutation rate increases with generation
        let mutation_chance = 0.1 + (brain.generation as f64 * 0.01).min(0.5);

        for decision_idx in 0..N_DECISIONS {
            // Mutate chromosome A
            if self.rng.gen_bool(mutation_chance) {
                self.mutate_chromosome(&mut brain.decisions[decision_idx].0);
            }

            // Mutate chromosome B
            if self.rng.gen_bool(mutation_chance) {
                self.mutate_chromosome(&mut brain.decisions[decision_idx].1);
            }
        }

        // Mutate expression bitmap
        if self.rng.gen_bool(0.1) {
            let bit = self.rng.gen_range(16);
            brain.expression ^= 1 << bit;
        }

        // Mutate divide count
        if self.rng.gen_bool(0.05) {
            brain.divide_count = (brain.divide_count as i8 + self.rng.gen_range_i32(-1, 2) as i8)
                .clamp(2, 7) as u8;
        }

        brain.update_gene_count();
    }

    /// Mutate a chromosome
    fn mutate_chromosome(&mut self, chromosome: &mut Chromosome) {
        if chromosome.genes.is_empty() {
            // Add initial gene
            chromosome.genes.push(self.create_random_gene());
            return;
        }

        let mutation_type = self.rng.gen_range(100);

        if mutation_type < 30 && chromosome.genes.len() > 1 {
            // Remove a gene (30%)
            let idx = self.rng.gen_range(chromosome.genes.len() as u32) as usize;
            chromosome.genes.remove(idx);
        } else if mutation_type < 60 {
            // Add a gene (30%)
            let new_gene = self.create_random_gene();
            let idx = self.rng.gen_range((chromosome.genes.len() + 1) as u32) as usize;
            chromosome.genes.insert(idx, new_gene);
        } else if mutation_type < 90 && !chromosome.genes.is_empty() {
            // Modify a gene (30%)
            let idx = self.rng.gen_range(chromosome.genes.len() as u32) as usize;
            self.mutate_gene(&mut chromosome.genes[idx]);
        } else {
            // Modify gene links (10%)
            if !chromosome.genes.is_empty() {
                let gene_count = chromosome.genes.len();
                let idx = self.rng.gen_range(gene_count as u32) as usize;

                let prod_idx = if self.rng.gen_bool(0.5) {
                    if self.rng.gen_bool(0.3) {
                        None
                    } else {
                        Some(self.rng.gen_range(gene_count as u32) as usize)
                    }
                } else {
                    chromosome.genes[idx].prod_index
                };

                let sum_idx = if self.rng.gen_bool(0.5) {
                    if self.rng.gen_bool(0.3) {
                        None
                    } else {
                        Some(self.rng.gen_range(gene_count as u32) as usize)
                    }
                } else {
                    chromosome.genes[idx].sum_index
                };

                let gene = &mut chromosome.genes[idx];
                gene.prod_index = prod_idx;
                gene.sum_index = sum_idx;
            }
        }
    }

    /// Create a random gene
    fn create_random_gene(&mut self) -> Gene {
        match self.rng.gen_range(5) {
            0 => Gene::new_constant(self.rng.gen_range_i32(-100, 100)),
            1 => Gene::new_sense(self.rng.gen_range(N_SENSES as u32) as usize),
            2 => {
                let min = self.rng.gen_range_i32(-100, 100);
                let max = self.rng.gen_range_i32(min, 100);
                Gene::new_limit(self.rng.gen_range(N_SENSES as u32) as usize, min, max)
            }
            3 => Gene::new_compare(
                self.rng.gen_range(N_SENSES as u32) as usize,
                self.rng.gen_range_i32(-100, 100),
            ),
            _ => Gene::new_match(self.rng.gen_range(N_SENSES as u32) as usize),
        }
    }

    /// Mutate a gene's parameters
    fn mutate_gene(&mut self, gene: &mut Gene) {
        // Mutate constants
        if self.rng.gen_bool(0.5) {
            gene.c1 += self.rng.gen_range_i32(-10, 11);
        }
        if self.rng.gen_bool(0.5) {
            gene.c2 += self.rng.gen_range_i32(-10, 11);
        }

        // Mutate sense index
        if self.rng.gen_bool(0.3) {
            gene.sense_index = self.rng.gen_range(N_SENSES as u32) as usize;
        }
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
