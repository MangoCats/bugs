pub mod graph;

use bugs_core::bug::Pos;
use bugs_core::constants::*;
use bugs_core::world::{World, WorldStats};
use std::collections::VecDeque;

/// RGB color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const GRAY: Color = Color::new(128, 128, 128);
}

/// Visualization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisMode {
    /// Show bugs with ethnicity coloring
    BugMap,
    /// Show environment (food/water/terrain)
    EnvironmentMap,
}

/// Renderer for bugs world with LEFTBAR activity visualization
pub struct Visualizer {
    width: usize,
    height: usize,
    mode: VisMode,
}

impl Visualizer {
    pub fn new(mode: VisMode) -> Self {
        Self {
            width: RENDER_WIDTH,  // 1760 + 80 = 1840
            height: RENDER_HEIGHT, // 1000
            mode,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_mode(&mut self, mode: VisMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> VisMode {
        self.mode
    }

    /// Render world to RGBA pixel buffer (1840×1000)
    pub fn render_to_rgba(&self, world: &World, buffer: &mut [u8]) {
        assert_eq!(buffer.len(), self.width * self.height * 4);

        // Clear to black
        buffer.fill(0);

        match self.mode {
            VisMode::BugMap => self.render_bug_map(world, buffer),
            VisMode::EnvironmentMap => self.render_environment_map(world, buffer),
        }

        // Always render LEFTBAR activity visualization
        self.render_leftbar(world, buffer);
    }

    /// Render bugs with position history trails (offset by LEFTBAR)
    fn render_bug_map(&self, world: &World, buffer: &mut [u8]) {
        // Render position history for all bugs
        for bug in world.bugs.values() {
            for (i, state) in bug.position_history.iter().enumerate() {
                let fade = (POS_HISTORY - i) as f32 / POS_HISTORY as f32;

                let r = ((bug.brain.ethnicity.r as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;
                let g = ((bug.brain.ethnicity.g as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;
                let b = ((bug.brain.ethnicity.b as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;

                let pos = state.pos.wrap();
                // Offset x by LEFTBAR
                if let Some(idx) = self.pixel_index_world(pos) {
                    buffer[idx] = r;
                    buffer[idx + 1] = g;
                    buffer[idx + 2] = b;
                    buffer[idx + 3] = 255;
                }
            }
        }
    }

    /// Render environment (food, water, bugs) offset by LEFTBAR
    fn render_environment_map(&self, world: &World, buffer: &mut [u8]) {
        for x in 0..WORLD_X {
            for y in 0..WORLD_Y {
                let pos = Pos::new(x as i32, y as i32);

                let (r, g, b) = if let Some(bug) = world.get_bug_at(pos) {
                    // Bug present - red based on age
                    let age = world.current_tick - bug.data.birthday;
                    let red = (255 + age / 4).max(128).min(255);
                    (red as u8, 0, 0)
                } else {
                    // No bug - show food and water
                    let food = world.get_cell(pos).map(|c| c.food).unwrap_or(0);
                    let water = world.get_cell(pos).map(|c| c.water).unwrap_or(0);

                    let g = ((food * 192) / FOOD_CAP).min(255) as u8;
                    let b = if water == 0 {
                        0
                    } else {
                        (water + 64).min(255) as u8
                    };

                    (0, g, b)
                };

                if let Some(idx) = self.pixel_index_world(pos) {
                    buffer[idx] = r;
                    buffer[idx + 1] = g;
                    buffer[idx + 2] = b;
                    buffer[idx + 3] = 255;
                }
            }
        }
    }

    /// Render LEFTBAR with activity ratios per row
    fn render_leftbar(&self, world: &World, buffer: &mut [u8]) {
        for y in 0..WORLD_Y {
            // Count actions for this row
            let mut action_counts = [0u32; N_ACTIONS];
            let mut total = 0u32;

            for x in 0..WORLD_X {
                let pos = Pos::new(x as i32, y as i32);
                if let Some(bug) = world.get_bug_at(pos) {
                    // Use most recent action from position history
                    let action = bug.position_history[0].action;
                    if action < N_ACTIONS {
                        action_counts[action] += 1;
                        total += 1;
                    }
                }
            }

            if total == 0 {
                continue;
            }

            // Draw colored bars showing action proportions
            let mut x_offset = 0;
            for (action, &count) in action_counts.iter().enumerate() {
                if count == 0 {
                    continue;
                }

                let width = (count * LEFTBAR as u32) / total;
                let color = self.action_color(action);

                for dx in 0..width.min(LEFTBAR as u32) {
                    let x = x_offset + dx as usize;
                    if x >= LEFTBAR {
                        break;
                    }

                    if let Some(idx) = self.pixel_index(x, y) {
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                        buffer[idx + 3] = 255;
                    }
                }

                x_offset += width as usize;
            }
        }
    }

    /// Get color for an action
    fn action_color(&self, action: usize) -> Color {
        match action {
            ACT_SLEEP => Color::rgb(32, 32, 32),        // Dark gray
            ACT_EAT => Color::rgb(0, 255, 0),           // Green
            ACT_TURN_CW => Color::rgb(128, 128, 255),   // Light blue
            ACT_TURN_CCW => Color::rgb(255, 128, 128),  // Light red
            ACT_MOVE => Color::rgb(255, 255, 0),        // Yellow
            ACT_MATE => Color::rgb(255, 0, 255),        // Magenta
            ACT_DIVIDE => Color::rgb(0, 255, 255),      // Cyan
            ACT_DEFEND => Color::rgb(255, 128, 0),      // Orange
            _ => Color::GRAY,
        }
    }

    /// Get pixel buffer index for world position (includes LEFTBAR offset)
    fn pixel_index_world(&self, pos: Pos) -> Option<usize> {
        let x = pos.x as usize + LEFTBAR; // Offset by LEFTBAR
        let y = pos.y as usize;

        if x < self.width && y < self.height {
            Some((y * self.width + x) * 4)
        } else {
            None
        }
    }

    /// Get pixel buffer index for absolute position
    fn pixel_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) * 4)
        } else {
            None
        }
    }
}

/// Graph renderer for BOTTOMBAR (separate component)
pub struct GraphRenderer {
    width: usize,
    height: usize,
    view_offset: usize, // Scroll position in ticks
    view_width: usize,  // Number of ticks visible
}

impl GraphRenderer {
    pub fn new() -> Self {
        Self {
            width: RENDER_WIDTH,
            height: BOTTOMBAR,
            view_offset: 0,
            view_width: RENDER_WIDTH,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Set scroll position
    pub fn set_offset(&mut self, offset: usize) {
        self.view_offset = offset;
    }

    /// Scroll by delta ticks
    pub fn scroll(&mut self, delta: i32) {
        if delta < 0 {
            self.view_offset = self.view_offset.saturating_sub((-delta) as usize);
        } else {
            self.view_offset += delta as usize;
        }
    }

    /// Get current scroll offset
    pub fn get_offset(&self) -> usize {
        self.view_offset
    }

    /// Render time-series graphs to RGBA buffer
    pub fn render_to_rgba(&self, stats_history: &VecDeque<WorldStats>, buffer: &mut [u8]) {
        assert_eq!(buffer.len(), self.width * self.height * 4);

        // Clear to black
        buffer.fill(0);

        if stats_history.is_empty() {
            return;
        }

        // Determine range of data to display
        let history_len = stats_history.len();
        let end_idx = history_len.saturating_sub(self.view_offset);
        let start_idx = end_idx.saturating_sub(self.view_width).max(0);

        if end_idx == 0 {
            return;
        }

        // Calculate auto-ranging for this window
        let (max_bugs, max_bd, max_mass, min_mass, max_genes, min_genes) =
            self.calculate_ranges(stats_history, start_idx, end_idx);

        // Draw graphs from right to left (most recent on right)
        for (i, x) in (start_idx..end_idx).rev().enumerate() {
            if i >= self.width {
                break;
            }

            let stats = &stats_history[x];
            let screen_x = self.width - 1 - i;

            // White background bar for population
            self.draw_vline(buffer, screen_x, 0, (stats.bug_count * BOTTOMBAR) / max_bugs, Color::WHITE);

            if i > 0 {
                let prev_stats = &stats_history[x + 1];

                // Draw lines between consecutive points
                self.draw_graph_line(buffer, screen_x, prev_stats.avg_genes as i32, stats.avg_genes as i32, min_genes, max_genes, Color::GRAY);
                self.draw_graph_line(buffer, screen_x, prev_stats.avg_food_per_cell, stats.avg_food_per_cell, min_mass, max_mass, Color::rgb(0, 255, 0));
                self.draw_graph_line(buffer, screen_x, (prev_stats.avg_bug_mass / 1024) as i32, (stats.avg_bug_mass / 1024) as i32, min_mass, max_mass, Color::rgb(0, 0, 255));

                // Event graphs
                self.draw_event_line(buffer, screen_x, prev_stats.movements, stats.movements, max_bd, Color::rgb(0, 255, 128));
                self.draw_event_line(buffer, screen_x, prev_stats.starvations, stats.starvations, max_bd, Color::rgb(0, 128, 0));
                self.draw_event_line(buffer, screen_x, prev_stats.drownings, stats.drownings, max_bd, Color::rgb(64, 0, 192));
                self.draw_event_line(buffer, screen_x, prev_stats.collisions, stats.collisions, max_bd, Color::rgb(255, 0, 0));
                self.draw_event_line(buffer, screen_x, prev_stats.births, stats.births, max_bd, Color::rgb(255, 0, 255));
            }
        }
    }

    fn calculate_ranges(&self, stats: &VecDeque<WorldStats>, start: usize, end: usize) -> (usize, u32, i32, i32, i32, i32) {
        let mut max_bugs = 1;
        let mut max_bd = 1;
        let mut max_mass = 1;
        let mut min_mass = i32::MAX;
        let mut max_genes = 1;
        let mut min_genes = i32::MAX;

        for i in start..end {
            let s = &stats[i];
            max_bugs = max_bugs.max(s.bug_count);
            max_bd = max_bd.max(s.births).max(s.starvations).max(s.collisions).max(s.drownings).max(s.movements);

            let weight = (s.avg_bug_mass / 1024) as i32;
            max_mass = max_mass.max(weight).max(s.avg_food_per_cell);
            min_mass = min_mass.min(weight);

            let genes = s.avg_genes as i32;
            max_genes = max_genes.max(genes);
            min_genes = min_genes.min(genes);
        }

        if min_mass == i32::MAX {
            min_mass = 0;
        }
        if max_genes == min_genes {
            max_genes += 1;
        }

        (max_bugs, max_bd, max_mass, min_mass, max_genes, min_genes)
    }

    fn draw_vline(&self, buffer: &mut [u8], x: usize, y_start: usize, height: usize, color: Color) {
        for y in y_start..(y_start + height).min(self.height) {
            if let Some(idx) = self.pixel_index(x, self.height - 1 - y) {
                buffer[idx] = color.r;
                buffer[idx + 1] = color.g;
                buffer[idx + 2] = color.b;
                buffer[idx + 3] = 255;
            }
        }
    }

    fn draw_graph_line(&self, buffer: &mut [u8], x: usize, prev_val: i32, curr_val: i32, min: i32, max: i32, color: Color) {
        if max == min {
            return;
        }

        let prev_y = ((prev_val - min) * self.height as i32) / (max - min);
        let curr_y = ((curr_val - min) * self.height as i32) / (max - min);

        self.draw_line_segment(buffer, x, prev_y as usize, curr_y as usize, color);
    }

    fn draw_event_line(&self, buffer: &mut [u8], x: usize, prev_val: u32, curr_val: u32, max: u32, color: Color) {
        if max == 0 {
            return;
        }

        let prev_y = (prev_val * self.height as u32) / max;
        let curr_y = (curr_val * self.height as u32) / max;

        self.draw_line_segment(buffer, x, prev_y as usize, curr_y as usize, color);
    }

    fn draw_line_segment(&self, buffer: &mut [u8], x: usize, y1: usize, y2: usize, color: Color) {
        let y_min = y1.min(y2).min(self.height - 1);
        let y_max = y1.max(y2).min(self.height - 1);

        for y in y_min..=y_max {
            if let Some(idx) = self.pixel_index(x, self.height - 1 - y) {
                buffer[idx] = color.r;
                buffer[idx + 1] = color.g;
                buffer[idx + 2] = color.b;
                buffer[idx + 3] = 255;
            }
        }
    }

    fn pixel_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) * 4)
        } else {
            None
        }
    }
}

impl Default for GraphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let viz = Visualizer::new(VisMode::BugMap);
        assert_eq!(viz.width(), RENDER_WIDTH);
        assert_eq!(viz.height(), RENDER_HEIGHT);
    }

    #[test]
    fn test_graph_renderer_creation() {
        let graph = GraphRenderer::new();
        assert_eq!(graph.width(), RENDER_WIDTH);
        assert_eq!(graph.height(), BOTTOMBAR);
    }
}
