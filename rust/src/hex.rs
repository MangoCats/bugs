use crate::constants::*;
use crate::types::Pos;

pub fn east(p: &mut Pos) {
    if p.x < WORLD_X - 1 {
        p.x += 1;
    } else {
        p.x = 0;
    }
}

pub fn southeast(p: &mut Pos) {
    if p.y % 2 == 0 {
        if p.x < WORLD_X - 1 {
            p.x += 1;
        } else {
            p.x = 0;
        }
    }
    if p.y < WORLD_Y - 1 {
        p.y += 1;
    } else {
        p.y = 0;
    }
}

pub fn northeast(p: &mut Pos) {
    if p.y % 2 == 0 {
        if p.x < WORLD_X - 1 {
            p.x += 1;
        } else {
            p.x = 0;
        }
    }
    if p.y > 0 {
        p.y -= 1;
    } else {
        p.y = WORLD_Y - 1;
    }
}

pub fn west(p: &mut Pos) {
    if p.x > 0 {
        p.x -= 1;
    } else {
        p.x = WORLD_X - 1;
    }
}

pub fn southwest(p: &mut Pos) {
    if p.y % 2 != 0 {
        if p.x > 0 {
            p.x -= 1;
        } else {
            p.x = WORLD_X - 1;
        }
    }
    if p.y < WORLD_Y - 1 {
        p.y += 1;
    } else {
        p.y = 0;
    }
}

pub fn northwest(p: &mut Pos) {
    if p.y % 2 != 0 {
        if p.x > 0 {
            p.x -= 1;
        } else {
            p.x = WORLD_X - 1;
        }
    }
    if p.y > 0 {
        p.y -= 1;
    } else {
        p.y = WORLD_Y - 1;
    }
}

pub fn hexmove(p: &mut Pos, mut dir: i64) {
    // Compensate for rollovers - matching C exactly
    while dir < DIR_SW {
        dir += 6;
    }
    while dir > DIR_W {
        dir -= 6;
    }
    // Note: DIR_SW = 2, DIR_W = 3, so valid range is -2..=3
    match dir {
        DIR_NW => northwest(p),  // -2
        DIR_NE => northeast(p),  // -1
        DIR_E => east(p),        //  0
        DIR_SE => southeast(p),  //  1
        DIR_SW => southwest(p),  //  2
        DIR_W => west(p),        //  3
        _ => unreachable!(),
    }
}
