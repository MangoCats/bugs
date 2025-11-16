use serde::{Deserialize, Serialize};
use crate::gene::{Chromosome, Ethnicity};
use crate::constants::*;

/// 2D position in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Move in a direction (hexagonal grid)
    pub fn step(&self, facing: i8) -> Self {
        let mut new_pos = *self;

        match facing {
            DIR_E => new_pos.x += 1,
            DIR_W => new_pos.x -= 1,
            DIR_SE => {
                new_pos.x += if new_pos.y % 2 == 0 { 0 } else { 1 };
                new_pos.y += 1;
            }
            DIR_SW => {
                new_pos.x += if new_pos.y % 2 == 0 { -1 } else { 0 };
                new_pos.y += 1;
            }
            DIR_NE => {
                new_pos.x += if new_pos.y % 2 == 0 { 0 } else { 1 };
                new_pos.y -= 1;
            }
            DIR_NW => {
                new_pos.x += if new_pos.y % 2 == 0 { -1 } else { 0 };
                new_pos.y -= 1;
            }
            _ => {}
        }

        new_pos
    }

    /// Wrap coordinates to world bounds
    pub fn wrap(&self) -> Self {
        let x = if self.x < 0 {
            WORLD_X as i32 - 1
        } else if self.x >= WORLD_X as i32 {
            0
        } else {
            self.x
        };

        let y = if self.y < 0 {
            WORLD_Y as i32 - 1
        } else if self.y >= WORLD_Y as i32 {
            0
        } else {
            self.y
        };

        Pos::new(x, y)
    }
}

/// Bug state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugState {
    pub pos: Pos,
    pub facing: i8,
    pub action: usize,
    pub weight: i32,    // Fixed point: actual weight * 1024
    pub hydrate: i32,   // Water units
}

/// Bug brain - genetic programming decision system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugBrain {
    /// Two chromosomes (a/b - like diploid genetics) for each decision type
    pub decisions: Vec<(Chromosome, Chromosome)>,

    /// Family history for ethnicity tracking
    pub family: Vec<Ethnicity>,

    /// Current ethnicity
    pub ethnicity: Ethnicity,

    /// Generation number
    pub generation: u32,

    /// Number of offspring in division (2-7)
    pub divide_count: u8,

    /// Total gene count (cached for performance)
    pub n_genes: u16,

    /// Expression bitmap - which chromosomes are active
    pub expression: u16,
}

impl BugBrain {
    pub fn new() -> Self {
        let mut decisions = Vec::new();
        for _ in 0..N_DECISIONS {
            decisions.push((Chromosome::new(), Chromosome::new()));
        }

        Self {
            decisions,
            family: vec![Ethnicity::default(); FAM_HIST],
            ethnicity: Ethnicity::default(),
            generation: 0,
            divide_count: 2,
            n_genes: 0,
            expression: 0xFFFF, // All chromosomes active by default
        }
    }

    /// Count total genes across all chromosomes
    pub fn count_genes(&self) -> u16 {
        let mut count = 0;
        for (a, b) in &self.decisions {
            count += a.genes.len() + b.genes.len();
        }
        count as u16
    }

    /// Update cached gene count
    pub fn update_gene_count(&mut self) {
        self.n_genes = self.count_genes();
    }

    /// Evaluate a decision using both chromosomes
    /// Returns the weight for this action
    pub fn evaluate_decision(&self, decision_idx: usize, senses: &[i32]) -> f64 {
        if decision_idx >= self.decisions.len() {
            return 0.0;
        }

        let (chr_a, chr_b) = &self.decisions[decision_idx];

        // Check expression bitmap
        let use_a = (self.expression & (1 << (decision_idx * 2))) != 0;
        let use_b = (self.expression & (1 << (decision_idx * 2 + 1))) != 0;

        let val_a = if use_a { chr_a.evaluate(senses) } else { 0.0 };
        let val_b = if use_b { chr_b.evaluate(senses) } else { 0.0 };

        // Average the two chromosomes
        (val_a + val_b) / 2.0
    }
}

impl Default for BugBrain {
    fn default() -> Self {
        Self::new()
    }
}

/// Bug life history data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugData {
    pub birthday: i32,
    pub kills: u32,
    pub defends: u32,
    pub moves: u32,
    pub mate_success: u32,
    pub mate_reject: u32,
    pub children: u32,
    pub food_consumed: i32,
    pub underwater: i32,  // Number of turns underwater (for drowning)
}

impl Default for BugData {
    fn default() -> Self {
        Self {
            birthday: 0,
            kills: 0,
            defends: 0,
            moves: 0,
            mate_success: 0,
            mate_reject: 0,
            children: 0,
            food_consumed: 0,
            underwater: 0,
        }
    }
}

/// Complete bug entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bug {
    pub id: u64,
    pub brain: BugBrain,
    pub data: BugData,
    pub current_state: BugState,
    pub position_history: Vec<BugState>,
}

impl Bug {
    pub fn new(id: u64, pos: Pos, birthday: i32) -> Self {
        let state = BugState {
            pos,
            facing: DIR_E,
            action: ACT_SLEEP,
            weight: 1024 * 10, // Start with weight of 10
            hydrate: 10,
        };

        let mut bug = Self {
            id,
            brain: BugBrain::new(),
            data: BugData {
                birthday,
                ..Default::default()
            },
            current_state: state.clone(),
            position_history: vec![state; POS_HISTORY],
        };

        bug.brain.ethnicity.uid = id;
        bug
    }

    /// Update position history
    pub fn record_position(&mut self) {
        self.position_history.rotate_right(1);
        self.position_history[0] = self.current_state.clone();
    }

    /// Get age in ticks
    pub fn age(&self, current_tick: i32) -> i32 {
        current_tick - self.data.birthday
    }

    /// Get dry weight (weight without water)
    pub fn dry_weight(&self) -> i32 {
        self.current_state.weight / 1024
    }

    /// Check if bug would drown at a given water depth
    pub fn would_drown(&self, water_depth: i32) -> bool {
        water_depth > (self.current_state.weight * 10) / 1024
    }

    /// Cost of living based on gene count
    pub fn gene_cost(&self) -> i32 {
        let n = self.brain.n_genes as i32;
        let knee = GENE_KNEE;
        (n * n * n) / (knee * knee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_step() {
        let pos = Pos::new(10, 10);
        assert_eq!(pos.step(DIR_E), Pos::new(11, 10));
        assert_eq!(pos.step(DIR_W), Pos::new(9, 10));
    }

    #[test]
    fn test_pos_wrap() {
        let pos = Pos::new(-1, 10);
        assert_eq!(pos.wrap(), Pos::new(WORLD_X as i32 - 1, 10));

        let pos = Pos::new(WORLD_X as i32, 10);
        assert_eq!(pos.wrap(), Pos::new(0, 10));
    }

    #[test]
    fn test_bug_creation() {
        let bug = Bug::new(1, Pos::new(100, 100), 0);
        assert_eq!(bug.id, 1);
        assert_eq!(bug.age(100), 100);
        assert_eq!(bug.dry_weight(), 10);
    }
}
