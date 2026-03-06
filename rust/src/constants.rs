// Constants from 0.28/bugs.c - must match exactly for pixel-perfect output

pub const WORLD_X: i64 = 1120;
pub const WORLD_Y: i64 = 880;
pub const LEFTBAR: i64 = 80;
pub const RIGHTBAR: i64 = 80;
pub const SIDEBAR: i64 = LEFTBAR + RIGHTBAR;
pub const BOTTOMBAR: i64 = 80;
pub const SEASONLENGTH: i64 = 16384;
pub const FOODCAP: i64 = 1024000;
pub const FOODGROW: i64 = 1044;
pub const FOODSHADOW: i64 = 973;
pub const FOODSPREAD: i64 = 10;
pub const FOODSTART: i64 = 128000;
pub const FOODDECAY: i64 = 115;
pub const COSTSLEEP: i64 = 12;
pub const COSTEAT: i64 = 48;
pub const COSTTURN: i64 = 16;
pub const COSTMOVE: i64 = 96;
pub const COSTFIGHT: i64 = 36;
pub const COSTMATE: i64 = 12;
pub const COSTDIVIDE: i64 = 25600;
pub const NOMMASS: i64 = 1024;
pub const GENECOST: i64 = 128;
pub const GENEKNEE: i64 = 96;
pub const EATLIMIT: i64 = 205;
pub const DIETHIN: i64 = 102400;
pub const MASSCAP: i64 = 10240000;
pub const MASSTARGET: f32 = 600.0;

pub const ACTSLEEP: i64 = 0;
pub const ACTEAT: i64 = 1;
pub const ACTTURNCW: i64 = 2;
pub const ACTTURNCCW: i64 = 3;
pub const ACTMOVE: i64 = 4;
pub const ACTMATE: i64 = 5;
pub const ACTDIVIDE: i64 = 6;
pub const RESPONSEMATE: i64 = 7;
pub const ACTMATED: i64 = 7;
pub const ACTDEFEND: i64 = 8;
pub const NACT: usize = 9;
pub const NDECISIONS: usize = 8;
pub const POSHISTORY: usize = 32;

pub const ITEMFOOD: usize = 0;
pub const ITEMBUG: usize = 1;
pub const ITEMBUGFACE: usize = 2;
pub const ITEMBUGMATCH: usize = 3;

pub const DIR_E: i64 = 0;
pub const DIR_NE: i64 = -1;
pub const DIR_NW: i64 = -2;
pub const DIR_SE: i64 = 1;
pub const DIR_SW: i64 = 2;
pub const DIR_W: i64 = 3;
pub const DIR_CW: i64 = 1;
pub const DIR_CCW: i64 = -1;

pub const NSENSECELLS: usize = 12;
pub const SENSESELF: usize = NSENSECELLS * 6;
pub const SPAWNWEIGHTNORM: usize = NSENSECELLS * 6 + NACT;
pub const STARVEWEIGHTNORM: usize = SPAWNWEIGHTNORM + 1;
pub const SELFAGE: usize = STARVEWEIGHTNORM + 1;
pub const THIRSTSENSE: usize = SELFAGE + 1;
pub const NSENSES: usize = THIRSTSENSE + 1;

pub const GENECONST: i64 = 1;
pub const GENESENSE: i64 = 2;
pub const GENELIMIT: i64 = 3;
pub const GENECOMPARE: i64 = 4;
pub const GENEMATCH: i64 = 5;

pub const FAMHIST: usize = 126;
pub const LHIST: usize = 1300;
pub const ETHNIC_DUR: i64 = 120;

pub const POP_TARGET: i64 = (WORLD_X * WORLD_Y) / 50;
pub const POP_HARDLIMIT: i64 = (WORLD_X * WORLD_Y) / 5;
pub const MAX_SLOPE: i64 = DIETHIN / 4096;
pub const DROWN_TIME: i64 = 8;
pub const INIT_DEPTH: i64 = DIETHIN / 1024;
pub const DROWN_DEPTH: i64 = DIETHIN / 256;
