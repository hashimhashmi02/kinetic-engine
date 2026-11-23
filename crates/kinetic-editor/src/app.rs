//! The preview window: the renderer's output with an egui transport bar over it.

use std::sync::Arc;
use std::time::Instant;

use kinetic_core::Timeline;
use kinetic_render::wgpu;
use kinetic_render::{AdapterChoice, Frame, Gpu, Renderer};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::EditorError;

/// Playback state. The wall clock only ever advances `time`; what is drawn depends on
/// `time` alone.
#[derive(Debug)]
struct Transport {
    time: f64,
    duration: f64,
    playing: bool,
    last_tick: Instant,
}

impl Transport {
    fn tick(&mut self) {
        let now = Instant::now();
        if self.playing {
            self.time += now.duration_since(self.last_tick).as_secs_f64();
            // Loop. A zero-length scene simply stays on its only frame.
            self.time = if self.duration > 0.0 { self.time % self.duration } else { 0.0 };
        }
        self.last_tick = now;
    }
}

struct View {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    gpu: Gpu,
    renderer: Renderer,
    egui: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

struct App {
    instance: wgpu::Instance,
    timeline: Timeline,
    transport: Transport,
    view: Option<View>,
    error: Option<EditorError>,
}

impl View {
    fn new(event_loop: &ActiveEventLoop, instance: &wgpu::Instance) -> Result<Self, EditorError> {
        let attributes = Window::default_attributes()
            .with_title("Kinetic")
            .with_inner_size(LogicalSize::new(1280.0, 720.0));
        let window = Arc::new(
            event_loop.create_window(attributes).map_err(|e| EditorError::Window(e.to_string()))?,
        );
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| EditorError::Window(e.to_string()))?;
        let gpu =
            pollster::block_on(Gpu::request(instance, AdapterChoice::from_env(), Some(&surface)))
                .map_err(EditorError::Render)?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&gpu.adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| EditorError::Window("surface is not supported by the adapter".into()))?;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&gpu.device, &config);

        let renderer = Renderer::new(&gpu.device, config.format);
        let context = egui::Context::default();
        let egui = egui_winit::State::new(
            context,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(gpu.device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &gpu.device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );
        Ok(Self { window, surface, config, gpu, renderer, egui, egui_renderer })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.view.is_none() {
            match View::new(event_loop, &self.instance) {
                Ok(view) => self.view = Some(view),
                Err(error) => {
                    self.error = Some(error);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(view) = self.view.as_mut() else { return };
        let response = view.egui.on_window_event(&view.window, &event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                view.config.width = size.width.max(1);
                view.config.height = size.height.max(1);
                view.surface.configure(&view.gpu.device, &view.config);
                view.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.transport.tick();
                view.redraw(&self.timeline, &mut self.transport);
                if self.transport.playing {
                    view.window.request_redraw();
                }
            }
            _ if response.repaint => view.window.request_redraw(),
            _ => {}
        }
    }
}

impl View {
    fn redraw(&mut self, timeline: &Timeline, transport: &mut Transport) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            // Minimised, resized mid-frame or lost: reconfigure and try on the next redraw.
            _ => {
                self.surface.configure(&self.gpu.device, &self.config);
                return;
            }
        };
        let target = frame.texture.create_view(&Default::default());
        let size = (self.config.width, self.config.height);

        // Transport bar.
        let input = self.egui.take_egui_input(&self.window);
        let context = self.egui.egui_ctx().clone();
        let output = context.run_ui(input, |ui| {
            egui::Panel::bottom("transport").show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(if transport.playing { "Pause" } else { "Play" }).clicked() {
                        transport.playing = !transport.playing;
                    }
                    ui.spacing_mut().slider_width = (ui.available_width() - 90.0).max(80.0);
                    let slider = egui::Slider::new(&mut transport.time, 0.0..=transport.duration)
                        .suffix(" s")
                        .fixed_decimals(2);
                    if ui.add(slider).dragged() {
                        // Scrubbing takes over from playback.
                        transport.playing = false;
                    }
                });
            });
        });
        self.egui.handle_platform_output(&self.window, output.platform_output);
        let jobs = context.tessellate(output.shapes, output.pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.0, size.1],
            pixels_per_point: output.pixels_per_point,
        };

        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        let mut encoder = device.create_command_encoder(&Default::default());

        // The scene keeps its aspect ratio: fit the scene rectangle inside the window.
        let state = timeline.evaluate(transport.time);
        // Flatten curves to a quarter of a pixel at the current window size.
        let pixels_per_unit =
            (f64::from(size.0) / state.size.x).min(f64::from(size.1) / state.size.y).max(1e-6);
        let mut scene = Frame::from_state(&state, 0.25 / pixels_per_unit);
        let window_aspect = f64::from(size.0) / f64::from(size.1);
        let scene_aspect = scene.size.x / scene.size.y;
        if window_aspect > scene_aspect {
            scene.size.x = scene.size.y * window_aspect;
        } else {
            scene.size.y = scene.size.x / window_aspect;
        }
        self.renderer.draw(device, queue, &mut encoder, &target, size, &scene);

        // egui insists every texture change is consumed, so drain rather than borrow.
        let mut textures = output.textures_delta;
        for (id, deltas) in textures.set.drain() {
            for delta in &deltas {
                self.egui_renderer.update_texture(device, queue, id, delta);
            }
        }
        let extra = self.egui_renderer.update_buffers(device, queue, &mut encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("kinetic ui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.egui_renderer.render(&mut pass, &jobs, &screen);
        }
        for id in textures.free.drain() {
            self.egui_renderer.free_texture(&id);
        }
        queue.submit(extra.into_iter().chain([encoder.finish()]));
        self.window.pre_present_notify();
        queue.present(frame);
    }
}

/// Opens a preview window playing `timeline` on a loop and runs until it is closed.
pub fn run(timeline: Timeline) -> Result<(), EditorError> {
    let event_loop = EventLoop::new().map_err(|e| EditorError::Window(e.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
        Box::new(event_loop.owned_display_handle()),
    ));
    let mut app = App {
        instance,
        transport: Transport {
            time: 0.0,
            duration: timeline.duration,
            playing: true,
            last_tick: Instant::now(),
        },
        timeline,
        view: None,
        error: None,
    };
    event_loop.run_app(&mut app).map_err(|e| EditorError::Window(e.to_string()))?;
    app.error.map_or(Ok(()), Err)
}
