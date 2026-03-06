use crate::constants::*;
use crate::simulation::Simulation;
use crate::types::HistoryData;

/// Renders the simulation state into an RGBA pixel buffer matching the C libgd output.
/// Image dimensions: (WORLD_X + SIDEBAR) x (WORLD_Y + BOTTOMBAR)
pub struct Renderer {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            width: (WORLD_X + SIDEBAR) as u32,
            height: (WORLD_Y + BOTTOMBAR) as u32,
        }
    }

    /// Render a bug map frame (b*.jpg equivalent)
    pub fn render_bug_frame(&self, sim: &Simulation) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut pixels = vec![0u8; w * h * 3]; // RGB

        // Plot bugs with position history trail
        for i in (0..POSHISTORY).rev() {
            for &bug_idx in &sim.bug_order {
                if let Some(ref bug) = sim.bugs[bug_idx] {
                    let r = (255 * ((bug.brain.eth.r as i16) * (POSHISTORY as i16 - i as i16) / POSHISTORY as i16)) / ETHNIC_DUR as i16;
                    let g = (255 * ((bug.brain.eth.g as i16) * (POSHISTORY as i16 - i as i16) / POSHISTORY as i16)) / ETHNIC_DUR as i16;
                    let b = (255 * ((bug.brain.eth.b as i16) * (POSHISTORY as i16 - i as i16) / POSHISTORY as i16)) / ETHNIC_DUR as i16;

                    let px = LEFTBAR as usize + bug.pos[i].p.x as usize;
                    let py = bug.pos[i].p.y as usize;
                    if px < w && py < h {
                        let offset = (py * w + px) * 3;
                        pixels[offset] = r.clamp(0, 255) as u8;
                        pixels[offset + 1] = g.clamp(0, 255) as u8;
                        pixels[offset + 2] = b.clamp(0, 255) as u8;
                    }
                }
            }
        }

        self.render_graphs(sim, &mut pixels);
        pixels
    }

    /// Render an environment map frame (e*.jpg equivalent)
    pub fn render_env_frame(&self, sim: &Simulation) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut pixels = vec![0u8; w * h * 3];

        for x in 0..WORLD_X as usize {
            for y in 0..WORLD_Y as usize {
                let r;
                let g;
                let b;

                if let Some(bug_idx) = sim.world[x][y].bug {
                    if let Some(ref bug) = sim.bugs[bug_idx] {
                        let rv = 255 + (bug.birthday - sim.today) / 4;
                        r = rv.max(128).min(255) as u8;
                    } else {
                        r = 0;
                    }
                } else {
                    r = 0;
                }

                let gv = (sim.world[x][y].food * 192) / FOODCAP;
                g = gv.min(255).max(0) as u8;

                if sim.world[x][y].water == 0 {
                    b = 0;
                } else {
                    let bv = sim.world[x][y].water + 64;
                    b = bv.min(255).max(0) as u8;
                }

                let px = x + LEFTBAR as usize;
                let py = y;
                let offset = (py * w + px) * 3;
                pixels[offset] = r;
                pixels[offset + 1] = g;
                pixels[offset + 2] = b;
            }
        }

        self.render_graphs(sim, &mut pixels);
        pixels
    }

    fn render_graphs(&self, sim: &Simulation, pixels: &mut Vec<u8>) {
        let w = self.width as usize;
        let h = self.height as usize;

        // Bottom graph
        let mut maxbugs: i64 = 1;
        let mut maxbd: i64 = 1;
        let mut maxmass: i64 = 1;
        let mut maxgenes: i64 = 1;
        let minmass: i64 = 0;
        let mut mingenes: i64 = sim.hist[(sim.today % LHIST as i64) as usize].avggenes;

        let graph_width = WORLD_X + SIDEBAR;
        let y_range = if graph_width > sim.today { sim.today } else { graph_width };

        for x in 0..y_range {
            let idx = ((sim.today - x) % LHIST as i64) as usize;
            if sim.hist[idx].n_bugs > maxbugs { maxbugs = sim.hist[idx].n_bugs; }

            let poppct = (1024 * sim.hist[idx].n_bugs) / (WORLD_X * WORLD_Y);

            if sim.hist[idx].births > maxbd { maxbd = sim.hist[idx].births; }
            let deaths = sim.hist[idx].collisions + sim.hist[idx].drownings + sim.hist[idx].starvations;
            if deaths > maxbd { maxbd = deaths; }
            if sim.hist[idx].movement * poppct / 1024 > maxbd {
                maxbd = sim.hist[idx].movement * poppct / 1024;
            }

            if sim.hist[idx].avgweight > maxmass { maxmass = sim.hist[idx].avgweight; }
            if sim.hist[idx].avgfood > maxmass { maxmass = sim.hist[idx].avgfood; }
            if sim.hist[idx].avgweight < minmass { /* minmass = sim.hist[idx].avgweight; */ }

            if sim.hist[idx].avggenes > maxgenes { maxgenes = sim.hist[idx].avggenes; }
            if sim.hist[idx].avggenes < mingenes { mingenes = sim.hist[idx].avggenes; }
        }
        if maxgenes == mingenes { maxgenes += 1; }
        if maxmass == minmass { /* maxmass += 1 handled below */ }
        let maxmass = if maxmass == minmass { maxmass + 1 } else { maxmass };
        if maxbd == 0 { /* maxbd = 1 handled below */ }
        let maxbd = if maxbd == 0 { 1 } else { maxbd };

        // Plot bottom graph
        for x in 0..y_range {
            let idx = ((sim.today - x) % LHIST as i64) as usize;
            let px = (WORLD_X + SIDEBAR - 1 - x) as usize;

            // White n_bugs background
            let bug_height = (sim.hist[idx].n_bugs * BOTTOMBAR) / maxbugs;
            for row in 0..bug_height as usize {
                let py = (WORLD_Y + BOTTOMBAR - 1) as usize - row;
                if px < w && py < h {
                    let offset = (py * w + px) * 3;
                    pixels[offset] = 255;
                    pixels[offset + 1] = 255;
                    pixels[offset + 2] = 255;
                }
            }

            if x > 0 {
                let idx_prev = ((sim.today - x + 1) % LHIST as i64) as usize;
                let px_prev = px + 1;

                // Draw line segments for the various metrics
                self.draw_line_segment(pixels, w, h, px_prev, px, &sim.hist, idx_prev, idx,
                    |h| ((h.avggenes - mingenes) * BOTTOMBAR) / (maxgenes - mingenes),
                    96, 96, 96);

                self.draw_line_segment(pixels, w, h, px_prev, px, &sim.hist, idx_prev, idx,
                    |h| ((h.avgfood - minmass) * BOTTOMBAR) / (maxmass - minmass),
                    0, 255, 0);

                self.draw_line_segment(pixels, w, h, px_prev, px, &sim.hist, idx_prev, idx,
                    |h| ((h.avgweight - minmass) * BOTTOMBAR) / (maxmass - minmass),
                    0, 0, 255);

                let poppct = (1024 * sim.hist[idx].n_bugs) / (WORLD_X * WORLD_Y);
                // Movement
                {
                    let y1 = (sim.hist[idx_prev].movement * (1024 * sim.hist[idx_prev].n_bugs / (WORLD_X * WORLD_Y)) / 1024 * BOTTOMBAR) / maxbd;
                    let y2 = (sim.hist[idx].movement * poppct / 1024 * BOTTOMBAR) / maxbd;
                    self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, 0, 255, 128);
                }

                // Starvations (on top of collisions)
                {
                    let y1 = ((sim.hist[idx_prev].collisions + sim.hist[idx_prev].starvations) * BOTTOMBAR) / maxbd;
                    let y2 = ((sim.hist[idx].collisions + sim.hist[idx].starvations) * BOTTOMBAR) / maxbd;
                    self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, 0, 128, 0);
                }

                // Drownings
                {
                    let y1 = (sim.hist[idx_prev].drownings * BOTTOMBAR) / maxbd;
                    let y2 = (sim.hist[idx].drownings * BOTTOMBAR) / maxbd;
                    self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, 64, 0, 192);
                }

                // Collisions (red)
                {
                    let y1 = ((sim.hist[idx_prev].drownings + sim.hist[idx_prev].collisions) * BOTTOMBAR) / maxbd;
                    let y2 = ((sim.hist[idx].drownings + sim.hist[idx].collisions) * BOTTOMBAR) / maxbd;
                    self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, 255, 0, 0);
                }

                // Births (magenta)
                {
                    let y1 = (sim.hist[idx_prev].births * BOTTOMBAR) / maxbd;
                    let y2 = (sim.hist[idx].births * BOTTOMBAR) / maxbd;
                    self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, 255, 0, 255);
                }
            }
        }

        // Right bar - activity ratios
        for y in 0..WORLD_Y as usize {
            let mut actsum = [0i64; NACT];
            let mut c: i64 = 0;
            for x in 0..WORLD_X as usize {
                if let Some(bug_idx) = sim.world[x][y].bug {
                    if let Some(ref bug) = sim.bugs[bug_idx] {
                        let mut g = bug.birthday;
                        let mut b = 0;
                        while g < sim.today && b < POSHISTORY {
                            actsum[bug.pos[b].act as usize] += 1;
                            b += 1;
                            g += 1;
                            c += 1;
                        }
                    }
                }
            }
            if c > 0 {
                let mut prev_g = 0i64;
                for r in 0..NACT {
                    prev_g += actsum[r];
                    let (cr, cg, cb) = match r as i64 {
                        ACTSLEEP => (0, 0, 255),
                        ACTEAT => (0, 255, 0),
                        ACTTURNCW => (128, 128, 0),
                        ACTTURNCCW => (128, 0, 128),
                        ACTMOVE => (255, 0, 0),
                        ACTMATE => (255, 255, 255),
                        ACTDIVIDE => (0, 255, 255),
                        ACTMATED => (128, 0, 255),
                        ACTDEFEND => (192, 255, 0),
                        _ => (255, 255, 255),
                    };
                    let x_start = (WORLD_X + LEFTBAR) as usize + ((prev_g - actsum[r]) * RIGHTBAR) as usize / c as usize;
                    let x_end = (WORLD_X + LEFTBAR) as usize + (prev_g * RIGHTBAR) as usize / c as usize;
                    for px in x_start..x_end.min(w) {
                        let offset = (y * w + px) * 3;
                        if offset + 2 < pixels.len() {
                            pixels[offset] = cr;
                            pixels[offset + 1] = cg;
                            pixels[offset + 2] = cb;
                        }
                    }
                }
            }
        }

        // Left bar - population density, age, weight, kills, genes
        let mut maxage: i64 = 1;
        let mut maxbugs: i64 = 1;
        let mut maxmass: i64 = 1;
        let mut maxkills: i64 = 1;
        let mut maxgenes: i64 = 1;
        let mut mingenes: i64 = 1024000;

        // Pre-scan
        for y in 0..WORLD_Y as usize {
            let (mut age, mut bugs, mut mass, mut kills, mut genes) = (0i64, 0i64, 0i64, 0i64, 0i64);
            for x in 0..WORLD_X as usize {
                if let Some(bug_idx) = sim.world[x][y].bug {
                    if let Some(ref bug) = sim.bugs[bug_idx] {
                        bugs += 1;
                        age += sim.today - bug.birthday;
                        mass += bug.pos[0].weight;
                        kills += bug.kills;
                        genes += bug.brain.ngenes as i64;
                    }
                }
            }
            if bugs < 1 { bugs = 1; }
            age = (age * 1024) / bugs;
            mass /= bugs;
            kills = (kills * 1024) / bugs;
            genes = (genes * 1024) / bugs;

            if bugs > maxbugs { maxbugs = bugs; }
            if age > maxage { maxage = age; }
            if mass > maxmass { maxmass = mass; }
            if kills > maxkills { maxkills = kills; }
            if genes > maxgenes { maxgenes = genes; }
            if genes > 0 && genes < mingenes { mingenes = genes; }
        }
        if mingenes >= maxgenes { maxgenes = mingenes + 1; mingenes -= 1; }

        let mut lastbugs = 0i64;
        let mut lastage = 0i64;
        let mut lastmass = 0i64;
        let mut lastkills = 0i64;
        let mut lastgenes = 0i64;

        for y in 0..WORLD_Y as usize {
            let (mut age, mut bugs, mut mass, mut kills, mut genes) = (0i64, 0i64, 0i64, 0i64, 0i64);
            for x in 0..WORLD_X as usize {
                if let Some(bug_idx) = sim.world[x][y].bug {
                    if let Some(ref bug) = sim.bugs[bug_idx] {
                        bugs += 1;
                        age += sim.today - bug.birthday;
                        mass += bug.pos[0].weight;
                        kills += bug.kills;
                        genes += bug.brain.ngenes as i64;
                    }
                }
            }
            if bugs < 1 { bugs = 1; }
            age = (age * 1024) / bugs;
            mass /= bugs;
            kills = (kills * 1024) / bugs;
            genes = (genes * 1024) / bugs;
            if genes == 0 { genes = mingenes; }

            if y > 0 {
                // Population yellow
                self.draw_leftbar_line(pixels, w, h,
                    (lastbugs * LEFTBAR) / maxbugs, (bugs * LEFTBAR) / maxbugs,
                    y - 1, y, 255, 255, 0);
                // Age white
                self.draw_leftbar_line(pixels, w, h,
                    (lastage * LEFTBAR) / maxage, (age * LEFTBAR) / maxage,
                    y - 1, y, 255, 255, 255);
                // Mass blue
                self.draw_leftbar_line(pixels, w, h,
                    (lastmass * LEFTBAR) / maxmass, (mass * LEFTBAR) / maxmass,
                    y - 1, y, 0, 0, 255);
                // Kills red
                self.draw_leftbar_line(pixels, w, h,
                    (lastkills * LEFTBAR) / maxkills, (kills * LEFTBAR) / maxkills,
                    y - 1, y, 255, 0, 0);
                // Genes green
                self.draw_leftbar_line(pixels, w, h,
                    ((lastgenes - mingenes) * LEFTBAR) / (maxgenes - mingenes),
                    ((genes - mingenes) * LEFTBAR) / (maxgenes - mingenes),
                    y - 1, y, 0, 255, 0);
            }

            lastbugs = bugs;
            lastage = age;
            lastmass = mass;
            lastkills = kills;
            lastgenes = genes;
        }
    }

    fn draw_line_segment(&self, pixels: &mut Vec<u8>, w: usize, h: usize,
        px_prev: usize, px: usize,
        hist: &[HistoryData], idx_prev: usize, idx: usize,
        metric: impl Fn(&HistoryData) -> i64,
        r: u8, g: u8, b: u8)
    {
        let y1 = metric(&hist[idx_prev]);
        let y2 = metric(&hist[idx]);
        self.draw_graph_line(pixels, w, h, px_prev, px, y1, y2, r, g, b);
    }

    fn draw_graph_line(&self, pixels: &mut Vec<u8>, w: usize, h: usize,
        px1: usize, px2: usize, y1: i64, y2: i64,
        r: u8, g: u8, b: u8)
    {
        let py1 = (WORLD_Y + BOTTOMBAR - 1) as usize - y1.max(0).min(BOTTOMBAR - 1) as usize;
        let py2 = (WORLD_Y + BOTTOMBAR - 1) as usize - y2.max(0).min(BOTTOMBAR - 1) as usize;

        // Simple bresenham-like for single pixel wide
        let steps = ((px1 as i64 - px2 as i64).abs()).max((py1 as i64 - py2 as i64).abs()).max(1);
        for s in 0..=steps {
            let px = px1 as i64 + (px2 as i64 - px1 as i64) * s / steps;
            let py = py1 as i64 + (py2 as i64 - py1 as i64) * s / steps;
            let px = px as usize;
            let py = py as usize;
            if px < w && py < h {
                let offset = (py * w + px) * 3;
                if offset + 2 < pixels.len() {
                    pixels[offset] = r;
                    pixels[offset + 1] = g;
                    pixels[offset + 2] = b;
                }
            }
        }
    }

    fn draw_leftbar_line(&self, pixels: &mut Vec<u8>, w: usize, h: usize,
        x1: i64, x2: i64, y1: usize, y2: usize,
        r: u8, g: u8, b: u8)
    {
        let x1 = x1.max(0).min(LEFTBAR - 1) as usize;
        let x2 = x2.max(0).min(LEFTBAR - 1) as usize;
        let steps = ((x1 as i64 - x2 as i64).abs()).max((y1 as i64 - y2 as i64).abs()).max(1);
        for s in 0..=steps as usize {
            let px = x1 as i64 + (x2 as i64 - x1 as i64) * s as i64 / steps;
            let py = y1 as i64 + (y2 as i64 - y1 as i64) * s as i64 / steps;
            let px = px as usize;
            let py = py as usize;
            if px < w && py < h {
                let offset = (py * w + px) * 3;
                if offset + 2 < pixels.len() {
                    pixels[offset] = r;
                    pixels[offset + 1] = g;
                    pixels[offset + 2] = b;
                }
            }
        }
    }

    /// Encode RGB pixels as JPEG matching libgd quality 95
    pub fn encode_jpeg(&self, pixels: &[u8]) -> Vec<u8> {
        use image::{ImageBuffer, Rgb, ImageEncoder};
        use image::codecs::jpeg::JpegEncoder;

        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.width,
            self.height,
            pixels.to_vec(),
        ).expect("Failed to create image buffer");

        let mut buf = Vec::new();
        let encoder = JpegEncoder::new_with_quality(&mut buf, 95);
        encoder.write_image(
            img.as_raw(),
            self.width,
            self.height,
            image::ExtendedColorType::Rgb8,
        ).expect("Failed to encode JPEG");
        buf
    }
}
