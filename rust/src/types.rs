use crate::constants::*;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct Ethnicity {
    pub uid: i64,
    pub r: i8,
    pub g: i8,
    pub b: i8,
}

impl Default for Ethnicity {
    fn default() -> Self {
        Self { uid: -1, r: 0, g: 0, b: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BugState {
    pub p: Pos,
    pub face: i64,
    pub act: i64,
    pub weight: i64,
    pub hydrate: i64,
}

/// A gene in the genetic program tree.
/// In C this is a doubly-linked list with tree branches (prod/sum).
/// Here we use arena indices instead of pointers.
#[derive(Clone, Debug)]
pub struct Gene {
    pub tp: i64,
    pub si: i64,
    pub c1: i64,
    pub c2: i64,
    pub next: Option<usize>,  // index in GeneArena
    pub prev: Option<usize>,
    pub prod: Option<usize>,
    pub sum: Option<usize>,
}

/// Arena allocator for genes - replaces malloc/free of C gene nodes
#[derive(Clone, Debug)]
pub struct GeneArena {
    pub genes: Vec<Gene>,
    pub free_list: Vec<usize>,
}

impl GeneArena {
    pub fn new() -> Self {
        Self {
            genes: Vec::with_capacity(4096),
            free_list: Vec::new(),
        }
    }

    pub fn alloc(&mut self, gene: Gene) -> usize {
        if let Some(idx) = self.free_list.pop() {
            self.genes[idx] = gene;
            idx
        } else {
            let idx = self.genes.len();
            self.genes.push(gene);
            idx
        }
    }

    pub fn free(&mut self, idx: usize) {
        self.free_list.push(idx);
    }

    pub fn get(&self, idx: usize) -> &Gene {
        &self.genes[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Gene {
        &mut self.genes[idx]
    }
}

#[derive(Clone, Debug)]
pub struct BugAct {
    pub a: Option<usize>,  // gene arena index for chromosome a
    pub b: Option<usize>,  // gene arena index for chromosome b
    pub ea: Ethnicity,
    pub eb: Ethnicity,
}

impl Default for BugAct {
    fn default() -> Self {
        Self {
            a: None,
            b: None,
            ea: Ethnicity::default(),
            eb: Ethnicity::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BugBrain {
    pub act: [BugAct; NDECISIONS],
    pub family: [Ethnicity; FAMHIST],
    pub eth: Ethnicity,
    pub generation: i64,
    pub divide: i64,
    pub ngenes: i16,
    pub expression: i16,
}

impl BugBrain {
    pub fn new() -> Self {
        Self {
            act: std::array::from_fn(|_| BugAct::default()),
            family: [Ethnicity::default(); FAMHIST],
            eth: Ethnicity::default(),
            generation: 0,
            divide: 3,
            ngenes: 0,
            expression: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BugData {
    pub birthday: i64,
    pub kills: i64,
    pub defends: i64,
    pub moves: i64,
    pub mate_success: i64,
    pub mate_fails: i64,
    pub mate_repeat: i64,
    pub offspring: i64,
    pub underwater: i64,
    pub pos: [BugState; POSHISTORY],
    pub brain: BugBrain,
    pub matebrain: BugBrain,
}

impl BugData {
    pub fn new(p: Pos, face: i64, weight: i64, hydrate: i64) -> Self {
        let state = BugState { p, face, act: ACTSLEEP, weight, hydrate };
        Self {
            birthday: 0,
            kills: 0,
            defends: 0,
            moves: 0,
            mate_success: 0,
            mate_fails: 0,
            mate_repeat: 0,
            offspring: 0,
            underwater: 0,
            pos: [state; POSHISTORY],
            brain: BugBrain::new(),
            matebrain: BugBrain::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldCell {
    pub food: i64,
    pub water: i64,
    pub elevation: i64,
    pub nearest: i64,
    pub bug: Option<usize>,  // index into bug list
}

impl Default for WorldCell {
    fn default() -> Self {
        Self {
            food: FOODSTART,
            water: INIT_DEPTH,
            elevation: 0,
            nearest: -1,
            bug: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HistoryData {
    pub n_bugs: i64,
    pub movement: i64,
    pub collisions: i64,
    pub starvations: i64,
    pub drownings: i64,
    pub births: i64,
    pub avgweight: i64,
    pub avgfood: i64,
    pub avggenes: i64,
}
