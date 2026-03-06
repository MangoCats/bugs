use crate::constants::*;
use crate::hex::hexmove;
use crate::rng::Rng;
use crate::types::*;

pub struct Simulation {
    pub world: Vec<Vec<WorldCell>>,  // [x][y]
    pub bugs: Vec<Option<BugData>>,  // arena-style storage
    pub bug_order: Vec<usize>,       // iteration order (linked list simulation)
    pub genes: GeneArena,
    pub hist: Vec<HistoryData>,
    pub sense: [i64; NSENSES],
    pub today: i64,
    pub idcounter: i64,
    pub totalfood: i128,
    pub totalbug: i128,
    pub genecount: i128,
    pub leak: i64,
    pub geneknee2: i64,
    pub forcemate: i64,
    pub costmate: i64,
    pub agediv: i64,
    pub rot: [i64; 4],
    pub safety: i64,
    pub target_pop: i64,
    pub drink_or_die: i64,
    pub foodhump: f32,
    pub rng: Rng,
    pub stage: i64,
    pub wait: i64,
    pub n_bugs: i64,
    free_bug_slots: Vec<usize>,
}

impl Simulation {
    pub fn new() -> Self {
        let mut world = Vec::with_capacity(WORLD_X as usize);
        for _ in 0..WORLD_X {
            let mut col = Vec::with_capacity(WORLD_Y as usize);
            for _ in 0..WORLD_Y {
                col.push(WorldCell::default());
            }
            world.push(col);
        }

        let hist = vec![HistoryData::default(); LHIST];

        let mut sim = Simulation {
            world,
            bugs: Vec::new(),
            bug_order: Vec::new(),
            genes: GeneArena::new(),
            hist,
            sense: [0; NSENSES],
            today: 0,
            idcounter: 0,
            totalfood: 0,
            totalbug: 0,
            genecount: 0,
            leak: -1,
            geneknee2: GENEKNEE * GENEKNEE,
            forcemate: 0,
            costmate: COSTMATE,
            agediv: 0,
            rot: [988, 973, 1012, 1023],
            safety: 1,
            target_pop: POP_TARGET,
            drink_or_die: 4,
            foodhump: 1.4,
            rng: Rng::new(),
            stage: 0,
            wait: 0,
            n_bugs: 0,
            free_bug_slots: Vec::new(),
        };
        sim.bug_one();
        sim
    }

    // ---- Bug list management ----

    fn alloc_bug(&mut self, bug: BugData) -> usize {
        if let Some(idx) = self.free_bug_slots.pop() {
            self.bugs[idx] = Some(bug);
            idx
        } else {
            let idx = self.bugs.len();
            self.bugs.push(Some(bug));
            idx
        }
    }

    fn add_bug_to_world(&mut self, bug: BugData) -> usize {
        let x = bug.pos[0].p.x as usize;
        let y = bug.pos[0].p.y as usize;
        let idx = self.alloc_bug(bug);
        self.world[x][y].bug = Some(idx);
        self.bug_order.push(idx);
        self.n_bugs += 1;
        idx
    }

    fn kill_bug(&mut self, bug_idx: usize) {
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        let x = bug.pos[0].p.x as usize;
        let y = bug.pos[0].p.y as usize;
        let weight = bug.pos[0].weight;
        let hydrate = bug.pos[0].hydrate;

        // Free all genes in brain and matebrain
        self.free_brain_genes(bug_idx, false);
        self.free_brain_genes(bug_idx, true);

        self.world[x][y].food += weight;
        self.world[x][y].water += hydrate;
        self.world[x][y].bug = None;

        self.bugs[bug_idx] = None;
        self.free_bug_slots.push(bug_idx);
        self.n_bugs -= 1;
    }

    fn free_genes_chain(&mut self, start: Option<usize>) {
        let mut current = start;
        while let Some(idx) = current {
            let gene = self.genes.get(idx).clone();
            current = gene.next;
            self.genes.free(idx);
        }
    }

    fn free_brain_genes(&mut self, bug_idx: usize, mate: bool) {
        let brain = if mate {
            &self.bugs[bug_idx].as_ref().unwrap().matebrain
        } else {
            &self.bugs[bug_idx].as_ref().unwrap().brain
        };
        let mut chains = Vec::new();
        for i in 0..NDECISIONS {
            if let Some(a) = brain.act[i].a {
                chains.push(a);
            }
            if let Some(b) = brain.act[i].b {
                chains.push(b);
            }
        }
        for start in chains {
            self.free_genes_chain(Some(start));
        }
        let brain = if mate {
            &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
        } else {
            &mut self.bugs[bug_idx].as_mut().unwrap().brain
        };
        for i in 0..NDECISIONS {
            brain.act[i].a = None;
            brain.act[i].b = None;
        }
        brain.ngenes = 0;
    }

    // ---- Gene operations ----

    fn countgenes(&self, start: Option<usize>) -> i64 {
        let mut count = 0;
        let mut current = start;
        while let Some(idx) = current {
            count += 1;
            current = self.genes.get(idx).next;
        }
        count
    }

    /// Copy a chromosome tree. Uses a mutable cclp state to maintain prev links.
    fn copy_chromosome(&mut self, g: Option<usize>, cclp: &mut Option<usize>) -> Option<usize> {
        if let Some(src_idx) = g {
            let src = self.genes.get(src_idx).clone();
            let new_gene = Gene {
                tp: src.tp,
                si: src.si,
                c1: src.c1,
                c2: src.c2,
                next: None,
                prev: *cclp,
                prod: None,
                sum: None,
            };
            let new_idx = self.genes.alloc(new_gene);

            if let Some(prev_idx) = *cclp {
                self.genes.get_mut(prev_idx).next = Some(new_idx);
            }
            *cclp = Some(new_idx);

            let prod_src = src.prod;
            let sum_src = src.sum;
            let new_prod = self.copy_chromosome(prod_src, cclp);
            let new_sum = self.copy_chromosome(sum_src, cclp);
            self.genes.get_mut(new_idx).prod = new_prod;
            self.genes.get_mut(new_idx).sum = new_sum;

            Some(new_idx)
        } else {
            None
        }
    }

    /// Copy brain from one bug to another (by index), handling the "to" side
    fn copy_brain_data(&mut self, from: &BugBrain) -> BugBrain {
        let mut new_brain = BugBrain::new();
        new_brain.generation = from.generation;
        new_brain.divide = from.divide;
        new_brain.ngenes = from.ngenes;
        new_brain.expression = from.expression;
        new_brain.eth = from.eth;
        new_brain.family = from.family;

        for i in 0..NDECISIONS {
            let mut cclp = None;
            new_brain.act[i].a = self.copy_chromosome(from.act[i].a, &mut cclp);
            let mut cclp = None;
            new_brain.act[i].b = self.copy_chromosome(from.act[i].b, &mut cclp);
            new_brain.act[i].ea = from.act[i].ea;
            new_brain.act[i].eb = from.act[i].eb;
        }
        new_brain
    }

    /// Simple add_gene for bug_one initialization
    fn add_gene(&mut self, tp: i64, si: i64, c1: i64, c2: i64, og: Option<usize>, p: i64) -> Option<usize> {
        let gene = Gene {
            tp,
            si,
            c1,
            c2,
            next: og,
            prev: None,
            prod: None,
            sum: None,
        };
        let ng = self.genes.alloc(gene);

        if let Some(og_idx) = og {
            self.genes.get_mut(og_idx).prev = Some(ng);
            if p == 0 {
                self.genes.get_mut(ng).sum = Some(og_idx);
            } else {
                self.genes.get_mut(ng).prod = Some(og_idx);
            }
        }
        Some(ng)
    }

    // ---- World initialization ----

    fn bug_one(&mut self) {
        let px = WORLD_X / 2;
        let py = WORLD_Y / 2;
        let p = Pos { x: px, y: py };

        let mut bug = BugData::new(p, DIR_E, DIETHIN * 256, DIETHIN / 4);
        bug.birthday = self.today;

        for i in 0..FAMHIST {
            bug.brain.family[i] = Ethnicity {
                uid: -1,
                r: (ETHNIC_DUR / 8) as i8,
                g: (ETHNIC_DUR / 8) as i8,
                b: (ETHNIC_DUR / 8) as i8,
            };
        }
        bug.brain.eth.uid = self.idcounter;
        self.idcounter += 1;
        bug.brain.generation = 0;
        bug.brain.divide = 3;
        bug.brain.eth.r = ETHNIC_DUR as i8;
        bug.brain.eth.g = 0;
        bug.brain.eth.b = 0;

        let bug_idx = self.add_bug_to_world(bug);

        // Build the initial gene sets matching C bug_one() exactly
        for i in 0..NDECISIONS {
            let (a, b) = match i {
                0 => {
                    let a = self.add_gene(GENESENSE, THIRSTSENSE as i64, -5000, 2500, None, 0);
                    let b = self.add_gene(GENESENSE, THIRSTSENSE as i64, -20000, 3500, None, 0);
                    (a, b)
                }
                1 => {
                    let a0 = self.add_gene(5, 81, 1216, 1084, None, 0);
                    let a1 = self.add_gene(3, 81, 1216, 1084, a0, 0);
                    let a = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 1500, 1048, a1, 1);
                    let b0 = self.add_gene(3, 81, 1203, 1056, None, 0);
                    let b = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 2000, 1048, b0, 1);
                    (a, b)
                }
                2 => {
                    let a = self.add_gene(GENELIMIT, SENSESELF as i64 + i as i64, 50, 1200, None, 0);
                    let b = self.add_gene(GENELIMIT, SENSESELF as i64 + i as i64, 760, 776, None, 0);
                    (a, b)
                }
                3 => {
                    let a = self.add_gene(GENELIMIT, SENSESELF as i64 + i as i64, 100, 1000, None, 0);
                    let b = self.add_gene(GENELIMIT, SENSESELF as i64 + i as i64, 510, 514, None, 0);
                    (a, b)
                }
                4 => {
                    let a0 = self.add_gene(3, 82, 4274, 2187, None, 0);
                    let a1 = self.add_gene(3, 0, 173, -53, a0, 0);
                    let a = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 1500, 1048, a1, 1);
                    let b0 = self.add_gene(3, 82, 3944, 2187, None, 0);
                    let b1 = self.add_gene(3, 0, 226, -76, b0, 0);
                    let b = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 2000, 1048, b1, 1);
                    (a, b)
                }
                5 => {
                    let a0 = self.add_gene(2, 13, 734, 101, None, 0);
                    let a = self.add_gene(2, 79, 1421, 456, a0, 1);
                    let b0 = self.add_gene(2, 13, 785, 101, None, 0);
                    let b = self.add_gene(2, 79, 1339, 567, b0, 1);
                    (a, b)
                }
                6 => {
                    let a0 = self.add_gene(GENELIMIT, SPAWNWEIGHTNORM as i64, 1200, 3000, None, 1);
                    let a = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 3500, 1048, a0, 1);
                    let b0 = self.add_gene(GENELIMIT, SPAWNWEIGHTNORM as i64, 1800, 1850, None, 1);
                    let b = self.add_gene(GENECONST, NSENSECELLS as i64 + 1, 4000, 1048, b0, 1);
                    (a, b)
                }
                7 => {
                    let a = self.add_gene(3, 11, -50, 591, None, 0);
                    let b = self.add_gene(3, 75, -79, 546, None, 0);
                    (a, b)
                }
                _ => unreachable!(),
            };

            let count_a = self.countgenes(a) as i16;
            let count_b = self.countgenes(b) as i16;
            let bug = self.bugs[bug_idx].as_mut().unwrap();
            bug.brain.act[i].a = a;
            bug.brain.act[i].b = b;
            bug.brain.ngenes += count_a;
            bug.brain.ngenes += count_b;
            bug.brain.act[i].ea = bug.brain.eth;
            bug.brain.act[i].eb = bug.brain.eth;
            bug.matebrain.act[i].a = None;
            bug.matebrain.act[i].b = None;
        }

        // copy_brain(&brain, &matebrain) then mutatebrain(&matebrain)
        let brain_copy = self.bugs[bug_idx].as_ref().unwrap().brain.clone();
        let matebrain = self.copy_brain_data(&brain_copy);
        self.bugs[bug_idx].as_mut().unwrap().matebrain = matebrain;
        self.mutatebrain_on(bug_idx, true);
    }

    // ---- Growing season ----

    fn growing_season(&self, x: i64, y: i64) -> i64 {
        let sax = (x + (self.today * WORLD_X) / SEASONLENGTH) % WORLD_X;
        let fgf: f32 = 0.1
            + self.foodhump
                * (3.14159 * (sax as f32) / (WORLD_X as f32)).sin()
                * (0.51
                    - (3.14159 * 6.0 * (y as f32) / (WORLD_Y as f32)).cos() * 0.5);
        ((((FOODGROW - 1024) as f32) * fgf) as i64) + 1024
    }

    // ---- Update nearest ----

    fn update_nearest(&mut self) {
        for x in 0..WORLD_X as usize {
            for y in 0..WORLD_Y as usize {
                if self.world[x][y].bug.is_none() {
                    self.world[x][y].nearest = -1;
                } else {
                    self.world[x][y].nearest = 0;
                }
            }
        }
        // The rest of update_nearest is commented out in 0.28
    }

    // ---- Flow water ----

    fn flow_water(&mut self) {
        for _v in 0..4 {
            for x in 0..WORLD_X {
                for y in 0..WORLD_Y {
                    let j = self.rng.limitedrandom(6);
                    for i in -2..=3_i64 {
                        let mut p = Pos { x, y };
                        hexmove(&mut p, i + j);
                        let px = p.x as usize;
                        let py = p.y as usize;
                        let xu = x as usize;
                        let yu = y as usize;

                        // MAX_SLOPE erosion
                        if self.world[px][py].elevation > self.world[xu][yu].elevation + MAX_SLOPE {
                            self.world[px][py].elevation -= 1;
                            self.world[xu][yu].elevation += 1;
                        }
                        if self.world[xu][yu].elevation > self.world[px][py].elevation + MAX_SLOPE {
                            self.world[px][py].elevation += 1;
                            self.world[xu][yu].elevation -= 1;
                        }

                        let w = self.world[px][py].water + self.world[xu][yu].water;
                        if w > 0 {
                            if (self.world[px][py].water + self.world[px][py].elevation)
                                != (self.world[xu][yu].water + self.world[xu][yu].elevation)
                            {
                                let level = (self.world[px][py].water
                                    + self.world[px][py].elevation
                                    + self.world[xu][yu].water
                                    + self.world[xu][yu].elevation)
                                    / 2;

                                if self.world[px][py].elevation < self.world[xu][yu].elevation {
                                    let flow = level - self.world[px][py].elevation - self.world[px][py].water;
                                    self.world[px][py].water = level - self.world[px][py].elevation;
                                    if self.world[px][py].water > w {
                                        self.world[px][py].water = w;
                                        self.world[xu][yu].water = 0;
                                        self.world[px][py].elevation += 1;
                                        self.world[xu][yu].elevation -= 1;
                                    } else if self.world[px][py].water < 0 {
                                        self.world[px][py].water = 0;
                                        self.world[xu][yu].water = w;
                                    } else {
                                        self.world[xu][yu].water = w - self.world[px][py].water;
                                        let flow = flow / MAX_SLOPE;
                                        if flow > 0 {
                                            self.world[px][py].elevation += flow;
                                            self.world[xu][yu].elevation -= flow;
                                        }
                                    }
                                } else {
                                    let flow = level - self.world[xu][yu].elevation - self.world[xu][yu].water;
                                    self.world[xu][yu].water = level - self.world[xu][yu].elevation;
                                    if self.world[xu][yu].water > w {
                                        self.world[xu][yu].water = w;
                                        self.world[px][py].water = 0;
                                        self.world[px][py].elevation -= 1;
                                        self.world[xu][yu].elevation += 1;
                                    } else if self.world[xu][yu].water < 0 {
                                        self.world[xu][yu].water = 0;
                                        self.world[px][py].water = w;
                                    } else {
                                        self.world[px][py].water = w - self.world[xu][yu].water;
                                        let flow = flow / MAX_SLOPE;
                                        if flow > 0 {
                                            self.world[xu][yu].elevation += flow;
                                            self.world[px][py].elevation -= flow;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- Grow food ----

    pub fn grow_food(&mut self) {
        self.update_nearest();
        self.flow_water();

        self.totalfood = 0;
        self.totalbug = 0;
        self.genecount = 0;

        for y in 0..WORLD_Y {
            for x in 0..WORLD_X {
                let xu = x as usize;
                let yu = y as usize;
                let fgl = self.growing_season(x, y);

                if self.world[xu][yu].nearest == -1 || self.leak < self.world[xu][yu].nearest {
                    self.world[xu][yu].food = (self.world[xu][yu].food * fgl) / 1024;
                } else {
                    let nearest = self.world[xu][yu].nearest as usize;
                    self.world[xu][yu].food =
                        (self.world[xu][yu].food * self.rot[nearest]) / 1024;
                }

                if self.world[xu][yu].food > FOODCAP {
                    self.world[xu][yu].food -=
                        ((self.world[xu][yu].food - FOODCAP) * FOODDECAY) / 1024;
                }
                if self.world[xu][yu].food > FOODCAP * 10 {
                    self.world[xu][yu].food = FOODCAP * 10;
                }

                self.totalfood += (self.world[xu][yu].food / 1024) as i128;

                if let Some(bug_idx) = self.world[xu][yu].bug {
                    if let Some(ref bug) = self.bugs[bug_idx] {
                        self.totalbug += bug.pos[0].weight as i128;
                        self.genecount += bug.brain.ngenes as i128;
                    }
                }

                // Spread to nearby cells
                for i in -2..=3_i64 {
                    let mut p = Pos { x, y };
                    hexmove(&mut p, i);
                    let px = p.x as usize;
                    let py = p.y as usize;
                    if self.world[px][py].food < self.world[xu][yu].food / 16 {
                        if self.world[px][py].nearest == -1
                            || self.leak < self.world[px][py].nearest
                        {
                            let t = (self.world[xu][yu].food * FOODSPREAD) / 1024;
                            self.world[xu][yu].food -= t;
                            self.world[px][py].food += t;
                        }
                    }
                }
            }
        }
    }

    // ---- Family match ----

    fn range_match(b1: &BugBrain, b2: &BugBrain, s1: usize, e1: usize, s2: usize, e2: usize) -> i64 {
        let mut m = 0;
        for i in s1..=e1 {
            for j in s2..=e2 {
                if b1.family[i].uid == b2.family[j].uid {
                    m += 1;
                }
            }
        }
        m
    }

    fn family_match(b1: &BugBrain, b2: &BugBrain, level: i64) -> i64 {
        if level == 0 {
            return 1024;
        }
        let mut r = 0;
        r += Self::range_match(b1, b2, 0, 1, 0, 1) * 256;
        if r == 512 {
            return 1024;
        }
        if level == 3 {
            return r;
        }
        r += Self::range_match(b1, b2, 2, 5, 2, 5) * 64;
        if level == 2 {
            return r;
        }
        r += Self::range_match(b1, b2, 6, 13, 6, 13) * 16;
        r += Self::range_match(b1, b2, 14, 29, 14, 29) * 4;
        r += Self::range_match(b1, b2, 30, 62, 30, 62);
        r
    }

    // ---- Gather senses ----

    fn gather_senses(&mut self, bug_idx: usize) {
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        let bug_pos = bug.pos[0];
        let bug_face = bug.pos[0].face;
        let bug_weight = if bug.pos[0].weight <= 0 { 1 } else { bug.pos[0].weight };
        let bug_brain_clone = bug.brain.clone(); // needed for family_match

        for i in 0..NSENSECELLS {
            let mut cp = bug_pos.p;
            let level: i64;
            match i {
                0 => { level = 0; }
                1 => { hexmove(&mut cp, bug_face); level = 1; }
                2 => { hexmove(&mut cp, bug_face); hexmove(&mut cp, bug_face); level = 2; }
                3 => { hexmove(&mut cp, bug_face + DIR_CCW); level = 2; }
                4 => { hexmove(&mut cp, bug_face + DIR_CW); level = 2; }
                5 => { hexmove(&mut cp, bug_face); hexmove(&mut cp, bug_face); hexmove(&mut cp, bug_face); level = 3; }
                6 => { hexmove(&mut cp, bug_face + DIR_CCW); hexmove(&mut cp, bug_face + DIR_CCW); level = 3; }
                7 => { hexmove(&mut cp, bug_face + DIR_CCW); hexmove(&mut cp, bug_face); level = 3; }
                8 => { hexmove(&mut cp, bug_face + DIR_CW); hexmove(&mut cp, bug_face); level = 3; }
                9 => { hexmove(&mut cp, bug_face + DIR_CW); hexmove(&mut cp, bug_face + DIR_CW); level = 3; }
                10 => { hexmove(&mut cp, bug_face + DIR_CCW * 2); level = 3; }
                11 => { hexmove(&mut cp, bug_face + DIR_CW * 2); level = 3; }
                _ => unreachable!(),
            }

            let cx = cp.x as usize;
            let cy = cp.y as usize;
            let bpx = bug_pos.p.x as usize;
            let bpy = bug_pos.p.y as usize;

            self.sense[i] = (self.world[cx][cy].food * 1024) / bug_weight;
            self.sense[i + NSENSECELLS * 4] = self.world[cx][cy].water;
            self.sense[i + NSENSECELLS * 5] = self.world[cx][cy].elevation - self.world[bpx][bpy].elevation;

            if let Some(other_idx) = self.world[cx][cy].bug {
                if let Some(ref other) = self.bugs[other_idx] {
                    self.sense[i + NSENSECELLS] = (other.pos[0].weight * 1024) / bug_weight;
                    let mut f = other.pos[0].face - bug_face;
                    while f < -2 { f += 6; }
                    while f > 3 { f -= 6; }
                    self.sense[i + NSENSECELLS * 2] = f * 1024;
                    self.sense[i + NSENSECELLS * 3] = Self::family_match(&other.brain, &bug_brain_clone, level);
                } else {
                    self.sense[i + NSENSECELLS] = 0;
                    self.sense[i + NSENSECELLS * 2] = 0;
                    self.sense[i + NSENSECELLS * 3] = 0;
                }
            } else {
                self.sense[i + NSENSECELLS] = 0;
                self.sense[i + NSENSECELLS * 2] = 0;
                self.sense[i + NSENSECELLS * 3] = 0;
            }
        }

        // Self-awareness senses
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        for i in 0..NACT {
            let mut j = 0;
            while j < POSHISTORY {
                if bug.pos[j].act == i as i64 {
                    self.sense[i + NSENSECELLS * 6] = (j as i64 * 1024) / POSHISTORY as i64;
                    break;
                } else if j == POSHISTORY - 1 {
                    self.sense[i + NSENSECELLS * 6] = 1024;
                }
                j += 1;
            }
        }

        self.sense[SPAWNWEIGHTNORM] = (((bug.pos[0].weight / bug.brain.divide) - COSTDIVIDE) * 1024) / DIETHIN;
        self.sense[STARVEWEIGHTNORM] = (bug.pos[0].weight * 1024) / DIETHIN;
        self.sense[SELFAGE] = self.today - bug.birthday;
        self.sense[THIRSTSENSE] = bug.pos[0].hydrate;
    }

    // ---- Gene evaluation ----

    fn limit_fn(x: i64, l1: i64, l2: i64) -> i64 {
        if l1 <= l2 {
            if x < l1 { return 0; }
            if x > l2 { return 1024; }
            // NOTE: C code has a bug here: `if ( l1 = l2 )` which is assignment, always true when l1==l2
            // We must replicate this bug for pixel-perfect matching
            // In C, `if (l1 = l2)` assigns l2 to l1, then tests if l2 != 0
            // Since we can't mutate the params the same way in a pure function,
            // we replicate the behavior: after assignment l1=l2, if l2 != 0 return 512
            // If l2 == 0, fall through to the interpolation with l1=l2 (division by zero!)
            // Actually: the C code modifies local l1, so after `l1 = l2`:
            //   if l2 != 0: return 512
            //   if l2 == 0: compute (1024 * (x - l2)) / (l2 - l2) => divide by zero
            // But in practice this means: if l2 != 0, return 512. If l2==0, undefined behavior.
            // Let's replicate: treat `if (l1 = l2)` as `if l2 != 0`
            if l2 != 0 { return 512; }
            // l2 == 0 case: (1024 * (x - 0)) / (0 - 0) would be division by zero in C
            // This likely never happens in practice, but return 512 as safety
            return 512;
        }
        // l1 > l2 case
        if x < l2 { return 1024; }
        if x > l1 { return 0; }
        1024 - (1024 * (x - l2)) / (l1 - l2)
    }

    fn evaluate_gene(&self, g_idx: Option<usize>) -> i64 {
        let idx = match g_idx {
            Some(i) => i,
            None => return 0,
        };
        let gene = self.genes.get(idx);
        let si = gene.si.clamp(0, NSENSES as i64 - 1) as usize;
        let tp = gene.tp;
        let c1 = gene.c1;
        let c2 = gene.c2;
        let prod = gene.prod;
        let sum = gene.sum;

        let mut v = match tp {
            GENECONST => c1,
            GENESENSE => (self.sense[si] * c1) / 1024 + c2,
            GENECOMPARE => {
                // NOTE: C has fall-through from GENECOMPARE to GENEMATCH (missing break)
                // So GENECOMPARE computes v but then GENEMATCH overwrites it
                let _v_compare = (self.sense[si] - self.sense[(c1 as usize) % NSENSES]) + c2;
                // Fall through to GENEMATCH
                let v2 = 1024 - (self.sense[si] - self.sense[(c2 as usize) % NSENSES]).abs() * c1 / 1024;
                if v2 < 0 { 0 } else { v2 }
            }
            GENEMATCH => {
                let v2 = 1024 - (self.sense[si] - self.sense[(c2 as usize) % NSENSES]).abs() * c1 / 1024;
                if v2 < 0 { 0 } else { v2 }
            }
            _ => {
                // GENELIMIT and default
                Self::limit_fn(self.sense[si], c1, c2)
            }
        };

        if let Some(prod_idx) = prod {
            v = (v * self.evaluate_gene(Some(prod_idx))) / 1024;
        }
        if let Some(sum_idx) = sum {
            v += self.evaluate_gene(Some(sum_idx));
        }
        v
    }

    fn bugdecide(&self, brain: &BugBrain) -> i64 {
        let mut maxv: i64 = -1048576;
        let mut j = 0;
        let mut x: i16 = 1;
        for i in 0..=ACTDIVIDE as usize {
            let v = if (brain.expression & x) != 0 {
                self.evaluate_gene(brain.act[i].a)
            } else {
                self.evaluate_gene(brain.act[i].b)
            };
            x *= 2;
            if v > maxv {
                maxv = v;
                j = i as i64;
            }
        }
        j
    }

    // ---- Cost calculation ----

    fn costcalc(&mut self, cost: i64, bug_idx: usize) {
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        let weight = bug.pos[0].weight.abs();
        let ngenes = bug.brain.ngenes as i64;
        let mut mass = weight + (GENECOST * ngenes * ngenes * ngenes) / self.geneknee2;

        let mut cost = cost;
        if mass > MASSCAP {
            cost = cost * (1 + (mass - MASSCAP) / 102400);
        }
        mass = (cost * mass) / NOMMASS;

        let bug = self.bugs[bug_idx].as_mut().unwrap();
        bug.pos[0].weight -= mass;
        if bug.pos[0].weight <= 0 {
            bug.pos[0].weight = 1;
        }

        // Water transpiration
        mass = mass / 1024;
        mass = mass / self.drink_or_die;
        if mass > bug.pos[0].hydrate {
            mass = bug.pos[0].hydrate;
        }
        bug.pos[0].hydrate -= mass;

        // Rain down somewhere
        let rx = self.rng.limitedrandom(WORLD_X) as usize;
        let ry = self.rng.limitedrandom(WORLD_Y) as usize;
        self.world[rx][ry].water += mass;
    }

    // ---- Ethnicity ----

    fn det_ethnicity(offs: &mut Ethnicity, mom: &Ethnicity, dad: &Ethnicity, p: &Pos) {
        offs.r = ((mom.r as i16 + dad.r as i16) / 2) as i8;
        offs.g = ((mom.g as i16 + dad.g as i16) / 2) as i8;
        offs.b = ((mom.b as i16 + dad.b as i16) / 2) as i8;

        match (p.y * 3) / WORLD_Y {
            0 => {
                if offs.r > 0 { offs.r -= 1; offs.b += 1; }
                if offs.g > 0 { offs.g -= 1; offs.b += 1; }
                while (offs.r as i16 + offs.g as i16 + offs.b as i16) < ETHNIC_DUR as i16 {
                    offs.b += 1;
                }
            }
            1 => {
                if offs.g > 0 { offs.g -= 1; offs.r += 1; }
                if offs.b > 0 { offs.b -= 1; offs.r += 1; }
                while (offs.r as i16 + offs.g as i16 + offs.b as i16) < ETHNIC_DUR as i16 {
                    offs.r += 1;
                }
            }
            _ => {
                if offs.r > 0 { offs.r -= 1; offs.g += 1; }
                if offs.b > 0 { offs.b -= 1; offs.g += 1; }
                while (offs.r as i16 + offs.g as i16 + offs.b as i16) < ETHNIC_DUR as i16 {
                    offs.g += 1;
                }
            }
        }
    }

    // ---- Mutation ----

    fn tweakgene(&mut self, g_idx: usize) {
        let mut r = 1 + self.rng.limitedrandom(255);
        while r < 256 {
            match self.rng.limitedrandom(4) {
                0 => {
                    let gene = self.genes.get_mut(g_idx);
                    gene.tp += self.rng.limitedrandom(4) + 1;
                    if gene.tp > 5 { gene.tp -= 5; }
                }
                1 => {
                    let mut d = self.rng.limitedrandom(NSENSES as i64 + 6) - 3;
                    if d == 0 { d = 6; }
                    let gene = self.genes.get_mut(g_idx);
                    gene.si += d;
                    if gene.si < 0 { gene.si += NSENSES as i64; }
                    if gene.si > NSENSES as i64 - 1 { gene.si = gene.si % NSENSES as i64; }
                }
                2 => {
                    let d = 1024 + self.rng.limitedrandom(256) - 128;
                    let adj = self.rng.limitedrandom(128) - 64;
                    let gene = self.genes.get_mut(g_idx);
                    gene.c1 = (gene.c1 * d) / 1024 + adj;
                }
                3 => {
                    let d = 1024 + self.rng.limitedrandom(256) - 128;
                    let adj = self.rng.limitedrandom(128) - 64;
                    let gene = self.genes.get_mut(g_idx);
                    gene.c2 = (gene.c2 * d) / 1024 + adj;
                }
                _ => unreachable!(),
            }
            r *= 2;
        }
    }

    fn disposebranch(&mut self, g_idx: usize) -> i64 {
        if self.genes.get(g_idx).prev.is_none() {
            return 0;
        }
        let mut dropped = 0;
        let prod = self.genes.get(g_idx).prod;
        let sum = self.genes.get(g_idx).sum;
        if let Some(p) = prod {
            dropped += self.disposebranch(p);
        }
        if let Some(s) = sum {
            dropped += self.disposebranch(s);
        }
        // Fix the linked list
        let next = self.genes.get(g_idx).next;
        let prev = self.genes.get(g_idx).prev;
        if let Some(next_idx) = next {
            self.genes.get_mut(next_idx).prev = prev;
        }
        if let Some(prev_idx) = prev {
            self.genes.get_mut(prev_idx).next = next;
        }
        self.genes.free(g_idx);
        dropped + 1
    }

    fn mutatebrain_on(&mut self, bug_idx: usize, mate: bool) {
        let mut r = 1 + self.rng.limitedrandom(16383);
        while r < 16384 {
            let n = self.rng.limitedrandom(NDECISIONS as i64 + 1);

            if n == NDECISIONS as i64 {
                let adj = self.rng.limitedrandom(3) - 1;
                let brain = if mate {
                    &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                } else {
                    &mut self.bugs[bug_idx].as_mut().unwrap().brain
                };
                brain.divide += adj;
                if brain.divide > 7 { brain.divide = 6; }
                if brain.divide < 2 { brain.divide = 3; }
            } else {
                let n = n as usize;
                let ab = self.rng.limitedrandom(2);
                let brain = if mate {
                    &self.bugs[bug_idx].as_ref().unwrap().matebrain
                } else {
                    &self.bugs[bug_idx].as_ref().unwrap().brain
                };
                let eth = brain.eth;

                let g_head = if ab != 0 { brain.act[n].a } else { brain.act[n].b };

                // Set ethnicity on the modified chromosome
                let brain_mut = if mate {
                    &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                } else {
                    &mut self.bugs[bug_idx].as_mut().unwrap().brain
                };
                if ab != 0 {
                    brain_mut.act[n].ea = eth;
                } else {
                    brain_mut.act[n].eb = eth;
                }

                let g_head = g_head.unwrap(); // should always exist
                let c = self.countgenes(Some(g_head));
                let mut c_pick = self.rng.limitedrandom(c);
                let mut g = g_head;
                while c_pick > 0 {
                    g = self.genes.get(g).next.unwrap();
                    c_pick -= 1;
                }

                if self.rng.limitedrandom(2) != 0 {
                    self.tweakgene(g);
                } else {
                    if self.rng.limitedrandom(4) != 0 {
                        // Add a gene
                        let mut g2 = g_head;
                        loop {
                            let s = self.rng.limitedrandom(2);
                            if s != 0 {
                                if self.genes.get(g2).prod.is_none() {
                                    // Copy gene g's values into new gene
                                    let src = self.genes.get(g).clone();
                                    let new_gene = Gene {
                                        tp: src.tp, si: src.si, c1: src.c1, c2: src.c2,
                                        next: None, prev: None, prod: None, sum: None,
                                    };
                                    let gn = self.genes.alloc(new_gene);

                                    let brain = if mate {
                                        &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                                    } else {
                                        &mut self.bugs[bug_idx].as_mut().unwrap().brain
                                    };
                                    brain.ngenes += 1;

                                    self.genes.get_mut(g2).prod = Some(gn);
                                    // Traverse to end of list from g
                                    let mut end = g;
                                    while self.genes.get(end).next.is_some() {
                                        end = self.genes.get(end).next.unwrap();
                                    }
                                    self.genes.get_mut(end).next = Some(gn);
                                    self.genes.get_mut(gn).prev = Some(end);

                                    if self.rng.limitedrandom(2) != 0 {
                                        self.tweakgene(gn);
                                    }
                                    break;
                                } else {
                                    g2 = self.genes.get(g2).prod.unwrap();
                                }
                            } else {
                                if self.genes.get(g2).sum.is_none() {
                                    let src = self.genes.get(g).clone();
                                    let new_gene = Gene {
                                        tp: src.tp, si: src.si, c1: src.c1, c2: src.c2,
                                        next: None, prev: None, prod: None, sum: None,
                                    };
                                    let gn = self.genes.alloc(new_gene);

                                    let brain = if mate {
                                        &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                                    } else {
                                        &mut self.bugs[bug_idx].as_mut().unwrap().brain
                                    };
                                    brain.ngenes += 1;

                                    self.genes.get_mut(g2).sum = Some(gn);
                                    let mut end = g;
                                    while self.genes.get(end).next.is_some() {
                                        end = self.genes.get(end).next.unwrap();
                                    }
                                    self.genes.get_mut(end).next = Some(gn);
                                    self.genes.get_mut(gn).prev = Some(end);

                                    if self.rng.limitedrandom(2) != 0 {
                                        self.tweakgene(gn);
                                    }
                                    break;
                                } else {
                                    g2 = self.genes.get(g2).sum.unwrap();
                                }
                            }
                        }
                    } else {
                        // Prune
                        let gene = self.genes.get(g);
                        let has_prod = gene.prod.is_some();
                        let has_sum = gene.sum.is_some();
                        let s = if has_prod && has_sum {
                            self.rng.limitedrandom(2)
                        } else if has_prod {
                            1
                        } else if has_sum {
                            0
                        } else {
                            2
                        };
                        if s == 0 {
                            let sum_idx = self.genes.get(g).sum.unwrap();
                            let dropped = self.disposebranch(sum_idx);
                            self.genes.get_mut(g).sum = None;
                            let brain = if mate {
                                &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                            } else {
                                &mut self.bugs[bug_idx].as_mut().unwrap().brain
                            };
                            brain.ngenes -= dropped as i16;
                        }
                        if s == 1 {
                            let prod_idx = self.genes.get(g).prod.unwrap();
                            let dropped = self.disposebranch(prod_idx);
                            self.genes.get_mut(g).prod = None;
                            let brain = if mate {
                                &mut self.bugs[bug_idx].as_mut().unwrap().matebrain
                            } else {
                                &mut self.bugs[bug_idx].as_mut().unwrap().brain
                            };
                            brain.ngenes -= dropped as i16;
                        }
                    }
                }
            }
            r *= 2;
        }
    }

    // ---- Bug move ----

    fn bug_move(&mut self, bug_idx: usize) {
        self.gather_senses(bug_idx);

        // Shift history
        {
            let bug = self.bugs[bug_idx].as_mut().unwrap();
            for i in (1..POSHISTORY).rev() {
                bug.pos[i] = bug.pos[i - 1];
            }
        }

        let brain = self.bugs[bug_idx].as_ref().unwrap().brain.clone();
        let action = self.bugdecide(&brain);
        self.bugs[bug_idx].as_mut().unwrap().pos[0].act = action;

        match action {
            ACTSLEEP => {
                self.costcalc(COSTSLEEP, bug_idx);
                let bug = self.bugs[bug_idx].as_mut().unwrap();
                let p = bug.pos[0].p;
                let px = p.x as usize;
                let py = p.y as usize;
                if bug.pos[0].hydrate < bug.pos[0].weight / 1024 {
                    if self.world[px][py].water > 0 {
                        bug.pos[0].hydrate += self.world[px][py].water;
                        self.world[px][py].water = 0;
                        if bug.pos[0].hydrate > bug.pos[0].weight / 1024 {
                            self.world[px][py].water = bug.pos[0].hydrate - bug.pos[0].weight / 1024;
                            bug.pos[0].hydrate = bug.pos[0].weight / 1024;
                        }
                    }
                }
            }
            ACTEAT => {
                let bug = self.bugs[bug_idx].as_mut().unwrap();
                let mut mass = (bug.pos[0].weight * EATLIMIT) / 1024;
                let p = bug.pos[0].p;
                let px = p.x as usize;
                let py = p.y as usize;
                if mass > self.world[px][py].food {
                    bug.pos[0].weight -= mass - self.world[px][py].food;
                    mass = self.world[px][py].food;
                }
                bug.pos[0].weight += mass;
                self.world[px][py].food -= mass;
                self.world[px][py].elevation -= 1;
                let rx = self.rng.limitedrandom(WORLD_X) as usize;
                let ry = self.rng.limitedrandom(WORLD_Y) as usize;
                self.world[rx][ry].elevation += 1;
                self.costcalc(COSTEAT, bug_idx);
            }
            ACTTURNCW => {
                {
                    let bug = self.bugs[bug_idx].as_mut().unwrap();
                    if bug.pos[0].face < 3 {
                        bug.pos[0].face += 1;
                    } else {
                        bug.pos[0].face = -2;
                    }
                }
                self.costcalc(COSTTURN, bug_idx);
                let p = self.bugs[bug_idx].as_ref().unwrap().pos[0].p;
                self.world[p.x as usize][p.y as usize].elevation -= 1;
                let rx = self.rng.limitedrandom(WORLD_X) as usize;
                let ry = self.rng.limitedrandom(WORLD_Y) as usize;
                self.world[rx][ry].elevation += 1;
            }
            ACTTURNCCW => {
                {
                    let bug = self.bugs[bug_idx].as_mut().unwrap();
                    if bug.pos[0].face > -2 {
                        bug.pos[0].face -= 1;
                    } else {
                        bug.pos[0].face = 3;
                    }
                }
                self.costcalc(COSTTURN, bug_idx);
                let p = self.bugs[bug_idx].as_ref().unwrap().pos[0].p;
                self.world[p.x as usize][p.y as usize].elevation -= 1;
                let rx = self.rng.limitedrandom(WORLD_X) as usize;
                let ry = self.rng.limitedrandom(WORLD_Y) as usize;
                self.world[rx][ry].elevation += 1;
            }
            ACTMOVE => {
                self.bugs[bug_idx].as_mut().unwrap().moves += 1;
                self.hist[(self.today % LHIST as i64) as usize].movement += 1;
                let bug = self.bugs[bug_idx].as_ref().unwrap();
                let old_p = bug.pos[0].p;
                let face = bug.pos[0].face;
                self.world[old_p.x as usize][old_p.y as usize].elevation -= 1;
                let rx = self.rng.limitedrandom(WORLD_X) as usize;
                let ry = self.rng.limitedrandom(WORLD_Y) as usize;
                self.world[rx][ry].elevation += 1;

                let mut p = old_p;
                hexmove(&mut p, face);
                let defender_idx = self.world[p.x as usize][p.y as usize].bug;

                self.costcalc(COSTMOVE, bug_idx);
                let bug = self.bugs[bug_idx].as_mut().unwrap();
                if bug.pos[0].weight < 0 { bug.pos[0].weight = 0; }

                if let Some(def_idx) = defender_idx {
                    if self.safety != 0 {
                        // No kills while safety is on
                        return;
                    }
                    self.hist[(self.today % LHIST as i64) as usize].collisions += 1;

                    let defender = self.bugs[def_idx].as_ref().unwrap();
                    let mut mass = defender.pos[0].weight;
                    let def_face = defender.pos[0].face;
                    let def_defends = defender.defends;
                    let attacker = self.bugs[bug_idx].as_ref().unwrap();
                    let att_face = attacker.pos[0].face;
                    let att_kills = attacker.kills;
                    let att_weight = attacker.pos[0].weight;

                    let mut i = def_face - att_face;
                    while i < -2 { i += 6; }
                    while i > 3 { i -= 6; }

                    match i {
                        0 => {
                            mass *= (def_defends / 2) + 1;
                            mass /= 128;
                        }
                        1 | -1 => {
                            mass *= (def_defends / 4) + 1;
                            mass /= 1024;
                        }
                        2 | -2 => {
                            mass *= (def_defends / 8) + 1;
                            mass /= 8192;
                            mass -= att_kills;
                        }
                        3 => {
                            mass /= 65536;
                            mass -= att_kills * att_kills;
                        }
                        _ => {}
                    }
                    if mass < 0 { mass = 0; }

                    let roll = self.rng.limitedrandom(mass + (att_weight / 1024));
                    if roll > mass {
                        // Victory
                        self.bugs[bug_idx].as_mut().unwrap().kills += 1;
                        self.kill_bug(def_idx);
                        self.world[p.x as usize][p.y as usize].bug = Some(bug_idx);
                        self.world[old_p.x as usize][old_p.y as usize].bug = None;
                        self.bugs[bug_idx].as_mut().unwrap().pos[0].p = p;
                        self.costcalc(COSTFIGHT, bug_idx);
                    } else {
                        // Defeat
                        self.bugs[def_idx].as_mut().unwrap().defends += 1;
                        let att_weight = self.bugs[bug_idx].as_ref().unwrap().pos[0].weight;
                        self.world[p.x as usize][p.y as usize].food += att_weight;
                        self.bugs[bug_idx].as_mut().unwrap().pos[0].weight = 0;
                        self.kill_bug(bug_idx);
                        self.world[p.x as usize][p.y as usize].bug = Some(def_idx);
                        // Shift defender history
                        let defender = self.bugs[def_idx].as_mut().unwrap();
                        for i in (1..POSHISTORY).rev() {
                            defender.pos[i] = defender.pos[i - 1];
                        }
                        defender.pos[0].act = ACTDEFEND;
                        return; // Bug is dead, skip rest
                    }
                } else {
                    // No defender, just move
                    self.world[p.x as usize][p.y as usize].bug = Some(bug_idx);
                    self.world[old_p.x as usize][old_p.y as usize].bug = None;
                    self.bugs[bug_idx].as_mut().unwrap().pos[0].p = p;
                }
            }
            ACTMATE => {
                let bug = self.bugs[bug_idx].as_ref().unwrap();
                let mut p = bug.pos[0].p;
                let face = bug.pos[0].face;
                hexmove(&mut p, face);

                if let Some(mate_idx) = self.world[p.x as usize][p.y as usize].bug {
                    // Evaluate mate's RESPONSEMATE using current sense[] (suitor's senses)
                    let mate = self.bugs[mate_idx].as_ref().unwrap();
                    let mate_resp_a = mate.brain.act[RESPONSEMATE as usize].a;
                    let mate_resp_b = mate.brain.act[RESPONSEMATE as usize].b;
                    let va = self.evaluate_gene(mate_resp_a);
                    let vb = self.evaluate_gene(mate_resp_b);

                    if va + vb > 0 {
                        // Success - swap matebrain
                        let bug_eth_uid = self.bugs[bug_idx].as_ref().unwrap().matebrain.eth.uid;
                        let mate_eth_uid = self.bugs[mate_idx].as_ref().unwrap().brain.eth.uid;
                        let bug_brain_eth_uid = self.bugs[bug_idx].as_ref().unwrap().brain.eth.uid;
                        let mate_matebrain_eth_uid = self.bugs[mate_idx].as_ref().unwrap().matebrain.eth.uid;

                        if bug_eth_uid != mate_eth_uid {
                            self.bugs[bug_idx].as_mut().unwrap().mate_success += 1;
                        } else {
                            self.bugs[bug_idx].as_mut().unwrap().mate_repeat += 1;
                        }
                        if bug_brain_eth_uid != mate_matebrain_eth_uid {
                            self.bugs[mate_idx].as_mut().unwrap().mate_success += 1;
                        } else {
                            self.bugs[mate_idx].as_mut().unwrap().mate_repeat += 1;
                        }

                        // Copy mate's brain to bug's matebrain
                        let mate_brain = self.bugs[mate_idx].as_ref().unwrap().brain.clone();
                        self.free_brain_genes(bug_idx, true);
                        let new_matebrain = self.copy_brain_data(&mate_brain);
                        self.bugs[bug_idx].as_mut().unwrap().matebrain = new_matebrain;

                        // Copy bug's brain to mate's matebrain
                        let bug_brain = self.bugs[bug_idx].as_ref().unwrap().brain.clone();
                        self.free_brain_genes(mate_idx, true);
                        let new_mate_matebrain = self.copy_brain_data(&bug_brain);
                        self.bugs[mate_idx].as_mut().unwrap().matebrain = new_mate_matebrain;

                        // Shift mate's history
                        let mate = self.bugs[mate_idx].as_mut().unwrap();
                        for j in (1..POSHISTORY).rev() {
                            mate.pos[j] = mate.pos[j - 1];
                        }
                        mate.pos[0].act = ACTMATED;
                        self.bugs[bug_idx].as_mut().unwrap().pos[0].act = ACTMATED;
                    } else {
                        self.bugs[bug_idx].as_mut().unwrap().mate_fails += 1;
                    }
                } else {
                    self.bugs[bug_idx].as_mut().unwrap().mate_fails += 1;
                }
                self.costcalc(self.costmate, bug_idx);
            }
            ACTDIVIDE => {
                self.do_divide(bug_idx);
            }
            _ => {}
        }

        // Check starvation/thirst/drowning
        if self.bugs[bug_idx].is_some() {
            let bug = self.bugs[bug_idx].as_ref().unwrap();
            if bug.pos[0].weight < DIETHIN || bug.pos[0].hydrate <= 0 {
                self.hist[(self.today % LHIST as i64) as usize].starvations += 1;
                self.kill_bug(bug_idx);
            } else {
                let p = bug.pos[0].p;
                if self.world[p.x as usize][p.y as usize].water > DROWN_DEPTH {
                    self.bugs[bug_idx].as_mut().unwrap().underwater += 1;
                    if self.bugs[bug_idx].as_ref().unwrap().underwater > DROWN_TIME {
                        self.hist[(self.today % LHIST as i64) as usize].drownings += 1;
                        self.kill_bug(bug_idx);
                    }
                } else {
                    self.bugs[bug_idx].as_mut().unwrap().underwater = 0;
                }
            }
        }
    }

    fn do_divide(&mut self, bug_idx: usize) {
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        let divide = bug.brain.divide;

        // Enforce agediv
        if self.forcemate & 0x10 != 0 {
            if bug.birthday + self.agediv > self.today {
                let bug = self.bugs[bug_idx].as_mut().unwrap();
                if self.forcemate & 0x40 != 0 {
                    bug.pos[0].weight /= divide;
                }
                if self.forcemate & 0x20 != 0 {
                    bug.pos[0].weight -= COSTDIVIDE;
                }
                if bug.pos[0].weight < DIETHIN {
                    bug.pos[0].weight = DIETHIN;
                }
                self.costcalc(COSTSLEEP, bug_idx);
                return;
            }
        }

        // Enforce forcemate
        let bug = self.bugs[bug_idx].as_ref().unwrap();
        if self.forcemate & 0x01 != 0 {
            if bug.brain.eth.uid == bug.matebrain.eth.uid {
                let bug = self.bugs[bug_idx].as_mut().unwrap();
                if self.forcemate & 0x08 != 0 {
                    bug.pos[0].weight /= divide;
                }
                if self.forcemate & 0x04 != 0 {
                    bug.pos[0].weight -= COSTDIVIDE;
                }
                if bug.pos[0].weight < DIETHIN {
                    bug.pos[0].weight = DIETHIN;
                }
                self.costcalc(COSTSLEEP, bug_idx);
                return;
            }
        }

        let bug = self.bugs[bug_idx].as_mut().unwrap();
        let mass = (bug.pos[0].weight / divide) - COSTDIVIDE;
        let watercons_total = bug.pos[0].hydrate;
        let mut wetness = (watercons_total / divide) - 1;
        if wetness < 0 { wetness = 0; }
        bug.pos[0].weight = mass;
        if mass < DIETHIN {
            return;
        }
        bug.pos[0].hydrate = wetness;
        let mut watercons = watercons_total - wetness;

        let parent_pos = bug.pos[0].p;
        let parent_face = bug.pos[0].face;

        // Clone parent brains for offspring creation
        let parent_brain = bug.brain.clone();
        let parent_matebrain = bug.matebrain.clone();
        let parent_eth = bug.brain.eth;
        let mate_eth = bug.matebrain.eth;

        for child_i in 1..divide {
            let mut p = parent_pos;
            let mut face = parent_face;
            match child_i {
                1 => face += 3,
                2 => face -= 2,
                3 => face += 2,
                4 => face -= 1,
                5 => face += 1,
                6 => {}
                _ => {}
            }
            hexmove(&mut p, face);

            if self.world[p.x as usize][p.y as usize].bug.is_some() {
                continue; // Cell occupied, offspring not born
            }

            self.bugs[bug_idx].as_mut().unwrap().offspring += 1;
            self.hist[(self.today % LHIST as i64) as usize].births += 1;

            let mut offspring = BugData::new(p, face, mass, wetness);
            offspring.birthday = self.today;
            offspring.brain.eth.uid = self.idcounter;
            self.idcounter += 1;

            if parent_brain.generation > parent_matebrain.generation {
                offspring.brain.generation = parent_brain.generation + 1;
            } else {
                offspring.brain.generation = parent_matebrain.generation + 1;
            }

            offspring.brain.family[0] = parent_eth;
            offspring.brain.family[1] = mate_eth;
            let mut j = 2;
            while j + 1 < FAMHIST {
                offspring.brain.family[j] = parent_brain.family[(j / 2) - 1];
                offspring.brain.family[j + 1] = parent_matebrain.family[(j / 2) - 1];
                j += 2;
            }

            Self::det_ethnicity(&mut offspring.brain.eth, &parent_eth, &mate_eth, &p);

            watercons -= wetness;

            // Build offspring brain from parent chromosomes
            let mut ngenes: i64 = 0;
            for j in 0..NDECISIONS {
                let (src_a, ea) = if self.rng.limitedrandom(2) != 0 {
                    (parent_brain.act[j].a, parent_brain.act[j].ea)
                } else {
                    (parent_brain.act[j].b, parent_brain.act[j].eb)
                };
                let mut cclp = None;
                offspring.brain.act[j].a = self.copy_chromosome(src_a, &mut cclp);
                offspring.brain.act[j].ea = ea;

                let (src_b, eb) = if self.rng.limitedrandom(2) != 0 {
                    (parent_matebrain.act[j].a, parent_matebrain.act[j].ea)
                } else {
                    (parent_matebrain.act[j].b, parent_matebrain.act[j].eb)
                };
                let mut cclp = None;
                offspring.brain.act[j].b = self.copy_chromosome(src_b, &mut cclp);
                offspring.brain.act[j].eb = eb;

                ngenes += self.countgenes(offspring.brain.act[j].a);
                ngenes += self.countgenes(offspring.brain.act[j].b);

                offspring.matebrain.act[j].a = None;
                offspring.matebrain.act[j].b = None;
            }
            offspring.brain.ngenes = ngenes as i16;

            if self.rng.limitedrandom(2) != 0 {
                offspring.brain.divide = parent_brain.divide;
            } else {
                offspring.brain.divide = parent_matebrain.divide;
            }
            offspring.brain.expression = self.rng.limitedrandom(256) as i16;

            // Copy brain to matebrain
            let offs_brain = offspring.brain.clone();
            offspring.matebrain = self.copy_brain_data(&offs_brain);

            let offs_idx = self.add_bug_to_world(offspring);

            if self.rng.limitedrandom(4) == 0 {
                self.mutatebrain_on(offs_idx, true);
            }
            if self.rng.limitedrandom(8) == 0 {
                self.mutatebrain_on(offs_idx, false);
            }
        }

        // Rain down excess water
        let rx = self.rng.limitedrandom(WORLD_X) as usize;
        let ry = self.rng.limitedrandom(WORLD_Y) as usize;
        self.world[rx][ry].water += watercons;

        if self.forcemate & 0x02 != 0 {
            let bug = self.bugs[bug_idx].as_mut().unwrap();
            bug.matebrain.eth.uid = bug.brain.eth.uid;
        }
    }

    // ---- Move all bugs ----

    pub fn move_bugs(&mut self) {
        // Build iteration order from bug_order (simulates linked list traversal)
        let order: Vec<usize> = self.bug_order.clone();
        for &bug_idx in &order {
            if self.bugs[bug_idx].is_some() {
                self.bug_move(bug_idx);
            }
        }
        // Clean up bug_order - remove dead bugs
        self.bug_order.retain(|&idx| self.bugs[idx].is_some());
    }

    // ---- Main step ----

    pub fn step(&mut self) -> bool {
        self.today += 1;

        // Dynamic challenges
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            if self.stage == 0 && self.n_bugs > 1000 { self.foodhump = 0.9; self.stage = 1; self.wait = 0; }
            if self.stage == 1 && self.n_bugs > 3000 { self.safety = 0; self.stage = 2; self.wait = 0; }
            if self.stage == 2 && self.n_bugs > 5000 { self.leak = 0; self.stage = 3; self.wait = 250; }
        }

        if self.today == 3000 { self.forcemate = 0x10; }
        if self.today == 4000 { self.forcemate = 0x30; }
        if self.today == 5000 { self.forcemate = 0x70; }
        if self.today == 6000 { self.forcemate = 0x71; }
        if self.today == 7000 { self.forcemate = 0x73; }
        if self.today == 8000 { self.forcemate = 0x77; }
        if self.today == 9000 { self.forcemate = 0x7F; }
        if self.today == 10000 { self.costmate = 24; }
        if self.today == 11000 { self.costmate = 48; }
        if self.today == 12000 { self.costmate = 96; }
        if self.today == 13000 { self.costmate = 144; }
        if self.today == 14000 { self.drink_or_die = 3; }
        if self.today == 15000 { self.drink_or_die = 2; }
        if self.today == 16000 { self.drink_or_die = 1; }

        if self.today > 3000 {
            if self.today > SEASONLENGTH {
                if self.today % 32 == 0 {
                    if self.agediv < 30 {
                        self.foodhump *= 1.001;
                    }
                    if self.agediv > 300
                        || (self.totalbug as f32 / (self.n_bugs as f32 * 1024.0)) > MASSTARGET
                    {
                        self.foodhump /= 1.001;
                    }
                }
            }

            if self.n_bugs > self.target_pop * 2
                && self.agediv
                    < (self.today
                        - self.bug_order.first()
                            .and_then(|&idx| self.bugs[idx].as_ref())
                            .map(|b| b.birthday)
                            .unwrap_or(self.today))
            {
                self.agediv += 1;
            }

            if self.today % 8 == 0 {
                self.agediv += 1;
            }
        }

        let oldest_birthday = self.bug_order.first()
            .and_then(|&idx| self.bugs[idx].as_ref())
            .map(|b| b.birthday)
            .unwrap_or(self.today);

        if (self.n_bugs < self.target_pop && self.agediv > 0)
            || self.agediv > (self.today - oldest_birthday)
        {
            self.agediv -= 1;
        }

        if self.n_bugs > POP_HARDLIMIT {
            self.agediv = self.today - oldest_birthday;
        }

        let h = (self.today % LHIST as i64) as usize;
        self.hist[h].movement = 0;
        self.hist[h].collisions = 0;
        self.hist[h].starvations = 0;
        self.hist[h].drownings = 0;
        self.hist[h].births = 0;

        self.move_bugs();
        self.grow_food();

        if self.n_bugs == 0 {
            println!("All bugs dead.");
            return false;
        }

        let h = (self.today % LHIST as i64) as usize;
        self.hist[h].n_bugs = self.n_bugs;
        self.hist[h].avgweight = self.totalbug as i64 / self.n_bugs;
        self.hist[h].avgfood = (self.totalfood * 1024) as i64 / (WORLD_X * WORLD_Y);
        self.hist[h].avggenes = (self.genecount * 1024) as i64 / self.n_bugs;

        true
    }

    pub fn should_output_frame(&self, interval: i64) -> bool {
        self.today % interval == 0
    }
}
