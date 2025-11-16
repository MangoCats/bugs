use bugs_core::bug::Pos;
use bugs_core::constants::*;
use bugs_core::world::World;

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
}

/// Visualization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisMode {
    /// Show bugs with ethnicity coloring
    BugMap,
    /// Show environment (food/water/terrain)
    EnvironmentMap,
}

/// Renderer for bugs world
pub struct Visualizer {
    width: usize,
    height: usize,
    mode: VisMode,
}

impl Visualizer {
    pub fn new(mode: VisMode) -> Self {
        Self {
            width: WORLD_X,
            height: WORLD_Y,
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

    /// Render world to RGBA pixel buffer
    pub fn render_to_rgba(&self, world: &World, buffer: &mut [u8]) {
        assert_eq!(buffer.len(), self.width * self.height * 4);

        match self.mode {
            VisMode::BugMap => self.render_bug_map(world, buffer),
            VisMode::EnvironmentMap => self.render_environment_map(world, buffer),
        }
    }

    /// Render bugs with position history trails
    fn render_bug_map(&self, world: &World, buffer: &mut [u8]) {
        // Clear to black
        buffer.fill(0);

        // Render position history for all bugs
        for bug in world.bugs.values() {
            for (i, state) in bug.position_history.iter().enumerate() {
                let fade = (POS_HISTORY - i) as f32 / POS_HISTORY as f32;

                let r = ((bug.brain.ethnicity.r as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;
                let g = ((bug.brain.ethnicity.g as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;
                let b = ((bug.brain.ethnicity.b as f32 * fade * 255.0) / ETHNIC_DUR as f32) as u8;

                let pos = state.pos.wrap();
                if let Some(idx) = self.pixel_index(pos) {
                    buffer[idx] = r;
                    buffer[idx + 1] = g;
                    buffer[idx + 2] = b;
                    buffer[idx + 3] = 255;
                }
            }
        }
    }

    /// Render environment (food, water, bugs)
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

                if let Some(idx) = self.pixel_index(pos) {
                    buffer[idx] = r;
                    buffer[idx + 1] = g;
                    buffer[idx + 2] = b;
                    buffer[idx + 3] = 255;
                }
            }
        }
    }

    /// Get pixel buffer index for position
    fn pixel_index(&self, pos: Pos) -> Option<usize> {
        let x = pos.x as usize;
        let y = pos.y as usize;

        if x < self.width && y < self.height {
            Some((y * self.width + x) * 4)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let viz = Visualizer::new(VisMode::BugMap);
        assert_eq!(viz.width(), WORLD_X);
        assert_eq!(viz.height(), WORLD_Y);
    }

    #[test]
    fn test_pixel_index() {
        let viz = Visualizer::new(VisMode::BugMap);
        let idx = viz.pixel_index(Pos::new(0, 0)).unwrap();
        assert_eq!(idx, 0);

        let idx = viz.pixel_index(Pos::new(1, 0)).unwrap();
        assert_eq!(idx, 4);

        let idx = viz.pixel_index(Pos::new(0, 1)).unwrap();
        assert_eq!(idx, WORLD_X * 4);
    }
}
