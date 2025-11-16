// World dimensions
pub const WORLD_X: usize = 1760;
pub const WORLD_Y: usize = 1000;

// Visualization margins (from original bugs.c)
pub const LEFTBAR: usize = 80;   // Left margin for activity visualization
pub const RIGHTBAR: usize = 0;   // Right margin (unused in Rust version)
pub const SIDEBAR: usize = LEFTBAR + RIGHTBAR;
pub const BOTTOMBAR: usize = 80; // Bottom margin for graphs

// Full render dimensions
pub const RENDER_WIDTH: usize = WORLD_X + SIDEBAR;
pub const RENDER_HEIGHT: usize = WORLD_Y; // BOTTOMBAR is separate component

// Action indices
pub const ACT_SLEEP: usize = 0;
pub const ACT_EAT: usize = 1;
pub const ACT_TURN_CW: usize = 2;
pub const ACT_TURN_CCW: usize = 3;
pub const ACT_MOVE: usize = 4;
pub const ACT_MATE: usize = 5;
pub const ACT_DIVIDE: usize = 6;
pub const RESPONSE_MATE: usize = 7;
pub const ACT_DEFEND: usize = 8;
pub const N_ACTIONS: usize = 9;
pub const N_DECISIONS: usize = 8;

// Directions
pub const DIR_E: i8 = 0;
pub const DIR_SE: i8 = 1;
pub const DIR_SW: i8 = 2;
pub const DIR_W: i8 = 3;
pub const DIR_NE: i8 = -1;
pub const DIR_NW: i8 = -2;

// Sensing
pub const N_SENSE_CELLS: usize = 12;
pub const SENSE_SELF: usize = N_SENSE_CELLS * 6;
pub const SPAWN_WEIGHT_NORM: usize = SENSE_SELF + N_ACTIONS;
pub const STARVE_WEIGHT_NORM: usize = SPAWN_WEIGHT_NORM + 1;
pub const SELF_AGE: usize = STARVE_WEIGHT_NORM + 1;
pub const THIRST_SENSE: usize = SELF_AGE + 1;
pub const N_SENSES: usize = THIRST_SENSE + 1;

// Position history
pub const POS_HISTORY: usize = 32;

// Items in cells
pub const ITEM_FOOD: usize = 0;
pub const ITEM_BUG: usize = 1;
pub const ITEM_BUG_FACE: usize = 2;
pub const ITEM_BUG_MATCH: usize = 3;

// Genetics
pub const FAM_HIST: usize = 126;
pub const ETHNIC_DUR: i32 = 120;
pub const GENE_KNEE: i32 = 48;

// Population
pub const POP_TARGET: usize = (WORLD_X * WORLD_Y) / 50;
pub const POP_HARD_LIMIT: usize = (WORLD_X * WORLD_Y) / 5;

// Environment
pub const FOOD_CAP: i32 = 2048;
pub const FOOD_START: i32 = 1024;
pub const FOOD_SPREAD: i32 = 10;
pub const DIE_THIN: i32 = 10240;
pub const DROWN_TIME: i32 = 8;
pub const INIT_DEPTH: i32 = DIE_THIN / 1024;
pub const DROWN_DEPTH: i32 = DIE_THIN / 256;
pub const MAX_SLOPE: i32 = DIE_THIN / 4096;

// Costs
pub const COST_SLEEP: i32 = 10;
pub const COST_EAT: i32 = 0;
pub const COST_TURN: i32 = 15;
pub const COST_MOVE: i32 = 35;
pub const COST_MATE_INITIAL: i32 = 15;
pub const COST_DIVIDE: i32 = 0;

// Season
pub const SEASON_LENGTH: i32 = 8192;

// History tracking
pub const L_HIST: usize = 1300;
