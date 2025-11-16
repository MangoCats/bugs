use bugs_core::simulation::{SimConfig, Simulation};
use bugs_render::{VisMode, Visualizer};
use egui::{Color32, Context};
use egui_wgpu::Renderer;
use egui_winit::State;
use pollster::block_on;
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct App {
    simulation: Simulation,
    visualizer: Visualizer,
    pixel_buffer: Vec<u8>,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    is_paused: bool,
    speed: u32,
    ticks_per_frame: u32,
}

impl App {
    fn new(seed: u64) -> Self {
        let config = SimConfig {
            seed,
            max_ticks: None,
        };

        let simulation = Simulation::new(config);
        let visualizer = Visualizer::new(VisMode::BugMap);
        let pixel_buffer = vec![0u8; visualizer.width() * visualizer.height() * 4];

        Self {
            simulation,
            visualizer,
            pixel_buffer,
            texture: None,
            texture_view: None,
            is_paused: false,
            speed: 1,
            ticks_per_frame: 1,
        }
    }

    fn update(&mut self) {
        if !self.is_paused {
            for _ in 0..self.ticks_per_frame {
                if !self.simulation.step() {
                    self.is_paused = true;
                    break;
                }
            }
        }
    }

    fn render_simulation(&mut self, device: &Device, queue: &Queue) {
        // Render to pixel buffer
        self.visualizer
            .render_to_rgba(&self.simulation.world, &mut self.pixel_buffer);

        // Update texture
        if self.texture.is_none() {
            self.create_texture(device);
        }

        if let Some(texture) = &self.texture {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.pixel_buffer,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.visualizer.width() as u32 * 4),
                    rows_per_image: Some(self.visualizer.height() as u32),
                },
                wgpu::Extent3d {
                    width: self.visualizer.width() as u32,
                    height: self.visualizer.height() as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn create_texture(&mut self, device: &Device) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("simulation_texture"),
            size: wgpu::Extent3d {
                width: self.visualizer.width() as u32,
                height: self.visualizer.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.texture = Some(texture);
        self.texture_view = Some(view);
    }

    fn ui(&mut self, ctx: &Context) {
        egui::Window::new("Controls").show(ctx, |ui| {
            ui.heading("Simulation");

            let stats = self.simulation.stats();
            ui.label(format!("Tick: {}", stats.tick));
            ui.label(format!("Bugs: {}", stats.bug_count));
            ui.label(format!("Avg Mass: {}", stats.avg_bug_mass));
            ui.label(format!("Avg Genes: {:.2}", stats.avg_genes));

            ui.separator();

            if ui.button(if self.is_paused { "Resume" } else { "Pause" }).clicked() {
                self.is_paused = !self.is_paused;
            }

            ui.horizontal(|ui| {
                ui.label("Speed:");
                if ui.button("1x").clicked() {
                    self.ticks_per_frame = 1;
                }
                if ui.button("10x").clicked() {
                    self.ticks_per_frame = 10;
                }
                if ui.button("100x").clicked() {
                    self.ticks_per_frame = 100;
                }
            });

            ui.separator();

            ui.label("Visualization:");
            if ui.button("Bug Map").clicked() {
                self.visualizer.set_mode(VisMode::BugMap);
            }
            if ui.button("Environment").clicked() {
                self.visualizer.set_mode(VisMode::EnvironmentMap);
            }
        });
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Bugs Viewer")
            .with_inner_size(winit::dpi::PhysicalSize::new(1760, 1000))
            .build(&event_loop)
            .unwrap(),
    );

    let mut app = App::new(42);

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let mut config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    // Initialize egui
    let mut egui_ctx = Context::default();
    let mut egui_state = State::new(egui_ctx.clone(), egui_ctx.viewport_id(), &window, None, None);
    let mut egui_renderer = Renderer::new(&device, surface_format, None, 1);

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                let _ = egui_state.on_window_event(&window, &event);

                match event {
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    WindowEvent::Resized(size) => {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&device, &config);
                    }
                    WindowEvent::RedrawRequested => {
                        // Update simulation
                        app.update();
                        app.render_simulation(&device, &queue);

                        // Prepare egui
                        let raw_input = egui_state.take_egui_input(&window);
                        let output = egui_ctx.run(raw_input, |ctx| {
                            app.ui(ctx);
                        });

                        egui_state.handle_platform_output(&window, output.platform_output);

                        let paint_jobs = egui_ctx.tessellate(output.shapes, output.pixels_per_point);

                        // Render
                        let frame = surface.get_current_texture().unwrap();
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("render_encoder"),
                            });

                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("render_pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            // TODO: Render simulation texture to screen
                        }

                        // Render egui
                        for (id, image_delta) in &output.textures_delta.set {
                            egui_renderer.update_texture(&device, &queue, *id, image_delta);
                        }

                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [config.width, config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        egui_renderer.update_buffers(&device, &queue, &mut encoder, &paint_jobs, &screen_descriptor);

                        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui_render_pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &frame_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }

                        for id in &output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        queue.submit(std::iter::once(encoder.finish()));
                        frame.present();
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
