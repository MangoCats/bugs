use bugs_core::world::World;
use bugs_core::world::WorldStats;
use bugs_render::{GraphRenderer, VisMode, Visualizer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use std::collections::VecDeque;

#[wasm_bindgen]
pub struct WebViewer {
    visualizer: Visualizer,
    graph_renderer: GraphRenderer,
    world: Option<World>,
    stats_history: VecDeque<WorldStats>,
    pixel_buffer: Vec<u8>,
    graph_buffer: Vec<u8>,
    #[allow(dead_code)]
    main_canvas: HtmlCanvasElement,
    #[allow(dead_code)]
    graph_canvas: HtmlCanvasElement,
    main_ctx: CanvasRenderingContext2d,
    graph_ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl WebViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(main_canvas_id: &str, graph_canvas_id: &str) -> Result<WebViewer, JsValue> {
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();

        // Main canvas for world view
        let main_canvas = document
            .get_element_by_id(main_canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        let main_ctx = main_canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Graph canvas for time-series
        let graph_canvas = document
            .get_element_by_id(graph_canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        let graph_ctx = graph_canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let visualizer = Visualizer::new(VisMode::BugMap);
        let graph_renderer = GraphRenderer::new();

        let pixel_buffer = vec![0u8; visualizer.width() * visualizer.height() * 4];
        let graph_buffer = vec![0u8; graph_renderer.width() * graph_renderer.height() * 4];

        main_canvas.set_width(visualizer.width() as u32);
        main_canvas.set_height(visualizer.height() as u32);

        graph_canvas.set_width(graph_renderer.width() as u32);
        graph_canvas.set_height(graph_renderer.height() as u32);

        Ok(WebViewer {
            visualizer,
            graph_renderer,
            world: None,
            stats_history: VecDeque::new(),
            pixel_buffer,
            graph_buffer,
            main_canvas,
            graph_canvas,
            main_ctx,
            graph_ctx,
        })
    }

    #[wasm_bindgen]
    pub fn set_mode(&mut self, bug_map: bool) {
        let mode = if bug_map {
            VisMode::BugMap
        } else {
            VisMode::EnvironmentMap
        };
        self.visualizer.set_mode(mode);
    }

    #[wasm_bindgen]
    pub fn set_world(&mut self, world_data: &[u8]) -> Result<(), JsValue> {
        // Deserialize world from binary data
        match bincode::deserialize(world_data) {
            Ok(world) => {
                self.world = Some(world);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to deserialize world: {}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn add_stats(&mut self, stats_data: &[u8]) -> Result<(), JsValue> {
        // Deserialize stats from binary data
        match bincode::deserialize::<WorldStats>(stats_data) {
            Ok(stats) => {
                if self.stats_history.len() >= 1300 {
                    self.stats_history.pop_front();
                }
                self.stats_history.push_back(stats);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to deserialize stats: {}", e))),
        }
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        // Render main world view
        if let Some(ref world) = self.world {
            self.visualizer.render_to_rgba(world, &mut self.pixel_buffer);

            let image_data = ImageData::new_with_u8_clamped_array(
                wasm_bindgen::Clamped(&self.pixel_buffer),
                self.visualizer.width() as u32,
            )?;

            self.main_ctx.put_image_data(&image_data, 0.0, 0.0)?;
        }

        // Render graph
        if !self.stats_history.is_empty() {
            self.graph_renderer.render_to_rgba(&self.stats_history, &mut self.graph_buffer);

            let graph_image_data = ImageData::new_with_u8_clamped_array(
                wasm_bindgen::Clamped(&self.graph_buffer),
                self.graph_renderer.width() as u32,
            )?;

            self.graph_ctx.put_image_data(&graph_image_data, 0.0, 0.0)?;
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn scroll_graph(&mut self, delta: i32) {
        self.graph_renderer.scroll(delta);
    }

    #[wasm_bindgen]
    pub fn set_graph_offset(&mut self, offset: usize) {
        self.graph_renderer.set_offset(offset);
    }

    #[wasm_bindgen]
    pub fn get_stats(&self) -> String {
        if let Some(ref world) = self.world {
            let stats = world.stats();
            format!(
                "{{\"tick\":{},\"bug_count\":{},\"avg_mass\":{},\"avg_genes\":{:.2}}}",
                stats.tick, stats.bug_count, stats.avg_bug_mass, stats.avg_genes
            )
        } else {
            String::from("{}")
        }
    }

    #[wasm_bindgen]
    pub fn get_stats_count(&self) -> usize {
        self.stats_history.len()
    }

    #[wasm_bindgen]
    pub fn get_graph_offset(&self) -> usize {
        self.graph_renderer.get_offset()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
