use bugs_core::world::World;
use bugs_recorder::EventReader;
use bugs_render::{VisMode, Visualizer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[wasm_bindgen]
pub struct WebViewer {
    visualizer: Visualizer,
    world: Option<World>,
    pixel_buffer: Vec<u8>,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl WebViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<WebViewer, JsValue> {
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let visualizer = Visualizer::new(VisMode::BugMap);
        let pixel_buffer = vec![0u8; visualizer.width() * visualizer.height() * 4];

        canvas.set_width(visualizer.width() as u32);
        canvas.set_height(visualizer.height() as u32);

        Ok(WebViewer {
            visualizer,
            world: None,
            pixel_buffer,
            canvas,
            ctx,
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
    pub fn render(&mut self) -> Result<(), JsValue> {
        if let Some(ref world) = self.world {
            // Render to pixel buffer
            self.visualizer.render_to_rgba(world, &mut self.pixel_buffer);

            // Create ImageData and draw to canvas
            let image_data = ImageData::new_with_u8_clamped_array(
                wasm_bindgen::Clamped(&self.pixel_buffer),
                self.visualizer.width() as u32,
            )?;

            self.ctx.put_image_data(&image_data, 0.0, 0.0)?;
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_stats(&self) -> JsValue {
        if let Some(ref world) = self.world {
            let stats = world.stats();
            serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
