use bugs_core::bug::Pos;
use serde::{Deserialize, Serialize};

/// Cause of bug death
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    Starvation,
    Collision,
    Drowning,
}

/// Compact representation of genetic changes for recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactGenome {
    pub gene_count: u16,
    pub generation: u32,
    pub parent_id: Option<u64>,
}

/// Events that occur during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationEvent {
    /// A bug moved from one position to another
    BugMoved {
        id: u64,
        from: Pos,
        to: Pos,
        facing: i8,
        weight: i32,
    },

    /// A bug was born
    BugBorn {
        id: u64,
        parent_id: u64,
        pos: Pos,
        genome: CompactGenome,
        ethnicity_r: u8,
        ethnicity_g: u8,
        ethnicity_b: u8,
    },

    /// A bug died
    BugDied {
        id: u64,
        cause: DeathCause,
        age: i32,
    },

    /// Food changed at a position
    FoodChanged {
        pos: Pos,
        amount: i32,
    },

    /// A bug performed an action
    BugAction {
        id: u64,
        action: u8,
        weight_change: i32,
    },

    /// Two bugs mated
    BugsMated {
        id1: u64,
        id2: u64,
    },

    /// Tick marker (for synchronization)
    Tick {
        tick: i32,
        bug_count: usize,
    },
}

impl SimulationEvent {
    /// Get the size in bytes (approximate)
    pub fn size_estimate(&self) -> usize {
        match self {
            SimulationEvent::BugMoved { .. } => 32,
            SimulationEvent::BugBorn { .. } => 64,
            SimulationEvent::BugDied { .. } => 24,
            SimulationEvent::FoodChanged { .. } => 16,
            SimulationEvent::BugAction { .. } => 20,
            SimulationEvent::BugsMated { .. } => 16,
            SimulationEvent::Tick { .. } => 12,
        }
    }
}
