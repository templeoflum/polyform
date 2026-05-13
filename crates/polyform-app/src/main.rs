mod camera;
mod gpu;
#[cfg(target_os = "windows")]
mod spout;
mod ui;

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use polyform_core::animation::PipelineKind;
use polyform_core::engine::AnimationEngine;
use polyform_core::transition::{TransitionAction, TransitionTarget};
use polyform_core::frame_state::FrameState;
use polyform_core::palette::PALETTE_KEYS;
use polyform_core::preset::Preset;
use polyform_core::preset_io::{export_preset_json, make_layer};
use polyform_core::registry::Registry;
use polyform_core::types::{PhaseModulator, ShapeFunction, ToggleParam};

use gpu::GpuRenderer;
use ui::{OverlayState, ParamField, UiAction, frame_state_view_value};

// =============================================================================
// FPS counter
// =============================================================================

struct FpsCounter {
    frame_times: [f64; 60],
    index: usize,
    count: usize,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_times: [0.0; 60],
            index: 0,
            count: 0,
        }
    }

    fn push(&mut self, dt: f64) {
        self.frame_times[self.index] = dt;
        self.index = (self.index + 1) % 60;
        if self.count < 60 {
            self.count += 1;
        }
    }

    fn fps(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: f64 = self.frame_times[..self.count].iter().sum();
        if sum <= 0.0 {
            return 0.0;
        }
        self.count as f64 / sum
    }
}

// =============================================================================
// Window structs
// =============================================================================

struct OutputWindow {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct ControlWindow {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

// =============================================================================
// App
// =============================================================================

struct App {
    // Shared GPU
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    // Output window (pure wgpu rendering)
    output: Option<OutputWindow>,

    // Control window (egui)
    control: Option<ControlWindow>,

    // Lattice renderer
    renderer: Option<GpuRenderer>,

    // Core state
    engine: AnimationEngine,
    preset: Preset,
    registry: Registry,
    last_frame: Instant,
    current_dt: f64,
    frame_state: Option<FrameState>,

    // Mouse orbit state
    orbit_dragging: bool,
    last_cursor_pos: Option<(f64, f64)>,

    // Overlay state
    overlay: OverlayState,

    // Output window egui overlay
    output_egui_ctx: Option<egui::Context>,
    output_egui_state: Option<egui_winit::State>,
    output_egui_renderer: Option<egui_wgpu::Renderer>,

    // FPS counter
    fps_counter: FpsCounter,

    // Spout sender (Windows-only)
    #[cfg(target_os = "windows")]
    spout_sender: Option<spout::SpoutSender>,
    #[cfg(target_os = "windows")]
    spout_enabled: bool,
    #[cfg(target_os = "windows")]
    spout_include_overlays: bool,
}

impl App {
    fn new() -> Self {
        // Load preset from CLI arg or use default
        let preset = match std::env::args().nth(1) {
            Some(path) => match std::fs::read_to_string(&path) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                    eprintln!("Failed to parse preset {path}: {e}");
                    Preset::default()
                }),
                Err(e) => {
                    eprintln!("Failed to read preset {path}: {e}");
                    Preset::default()
                }
            },
            None => Preset::default(),
        };

        Self {
            device: None,
            queue: None,
            output: None,
            control: None,
            renderer: None,
            engine: AnimationEngine::new(),
            preset,
            registry: Registry::new(),
            last_frame: Instant::now(),
            current_dt: 1.0 / 60.0,
            frame_state: None,
            orbit_dragging: false,
            last_cursor_pos: None,
            overlay: OverlayState::default(),
            output_egui_ctx: None,
            output_egui_state: None,
            output_egui_renderer: None,
            fps_counter: FpsCounter::new(),
            #[cfg(target_os = "windows")]
            spout_sender: None,
            #[cfg(target_os = "windows")]
            spout_enabled: false,
            #[cfg(target_os = "windows")]
            spout_include_overlays: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.output.is_some() {
            return;
        }

        // ---- Create shared GPU resources ----
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Compute side-by-side layout based on primary monitor
        let output_w: i32 = 1280;
        let control_w: i32 = 900;
        let (output_x, output_y, control_x, control_y) =
            if let Some(monitor) = event_loop.primary_monitor() {
                let screen = monitor.size();
                let total_w = output_w + control_w;
                let x0 = ((screen.width as i32) - total_w) / 2;
                let y0 = ((screen.height as i32) - 720) / 2;
                (x0.max(0), y0.max(0), (x0 + output_w).max(0), y0.max(0))
            } else {
                (0, 0, output_w, 0)
            };

        // Create output window first (need it for adapter selection)
        let output_attrs = WindowAttributes::default()
            .with_title("Polyform Output")
            .with_inner_size(LogicalSize::new(output_w as u32, 720u32))
            .with_position(PhysicalPosition::new(output_x, output_y));
        let output_window = Arc::new(
            event_loop
                .create_window(output_attrs)
                .expect("Failed to create output window"),
        );

        let output_surface = instance
            .create_surface(output_window.clone())
            .expect("Failed to create output surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&output_surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("polyform_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed to create device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // ---- Configure output surface ----
        let output_size = output_window.inner_size();
        let output_caps = output_surface.get_capabilities(&adapter);
        let output_format = output_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(output_caps.formats[0]);

        let output_surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: output_format,
            width: output_size.width.max(1),
            height: output_size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: output_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        output_surface.configure(&device, &output_surface_config);

        // ---- Create GpuRenderer (decoupled from surface) ----
        let renderer = GpuRenderer::new(device.clone(), queue.clone(), output_format);

        // ---- Create control window (positioned to the right of output) ----
        let control_attrs = WindowAttributes::default()
            .with_title("Polyform Controls")
            .with_inner_size(LogicalSize::new(control_w as u32, 800u32))
            .with_position(PhysicalPosition::new(control_x, control_y));
        let control_window = Arc::new(
            event_loop
                .create_window(control_attrs)
                .expect("Failed to create control window"),
        );

        let control_surface = instance
            .create_surface(control_window.clone())
            .expect("Failed to create control surface");

        let control_size = control_window.inner_size();
        let control_caps = control_surface.get_capabilities(&adapter);
        let control_format = control_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(control_caps.formats[0]);

        let control_surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: control_format,
            width: control_size.width.max(1),
            height: control_size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: control_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        control_surface.configure(&device, &control_surface_config);

        // ---- Initialize egui ----
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            &control_window,
            Some(control_window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, control_format, None, 1, false);

        // ---- Initialize egui for output window overlay ----
        let output_egui_ctx = egui::Context::default();
        let output_egui_state = egui_winit::State::new(
            output_egui_ctx.clone(),
            output_egui_ctx.viewport_id(),
            &output_window,
            Some(output_window.scale_factor() as f32),
            None,
            None,
        );
        let output_egui_renderer = egui_wgpu::Renderer::new(&device, output_format, None, 1, false);

        // ---- Store everything ----
        self.device = Some(device);
        self.queue = Some(queue);

        self.output = Some(OutputWindow {
            window: output_window,
            surface: output_surface,
            surface_config: output_surface_config,
        });

        self.control = Some(ControlWindow {
            window: control_window,
            surface: control_surface,
            surface_config: control_surface_config,
            egui_ctx,
            egui_state,
            egui_renderer,
        });

        self.output_egui_ctx = Some(output_egui_ctx);
        self.output_egui_state = Some(output_egui_state);
        self.output_egui_renderer = Some(output_egui_renderer);

        self.renderer = Some(renderer);
        self.last_frame = Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let is_output = self
            .output
            .as_ref()
            .is_some_and(|o| o.window.id() == window_id);
        let is_control = self
            .control
            .as_ref()
            .is_some_and(|c| c.window.id() == window_id);

        // Close either window exits the app
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }

        if is_output {
            self.handle_output_event(event);
        } else if is_control {
            self.handle_control_event(event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Tick the engine once per event loop iteration
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        self.current_dt = dt;

        self.fps_counter.push(dt);

        let (mut frame_state, actions) = self.engine.tick(&self.preset, dt, dt);
        self.engine.apply_actions(&mut self.preset, &actions);
        frame_state.show_caustic = self.overlay.show_caustic;
        frame_state.color_mode = if self.overlay.show_domain_coloring { 1 }
            else if self.overlay.show_curl { 2 }
            else if self.overlay.show_conformal { 3 }
            else { 0 };
        frame_state.normal_shading = self.overlay.show_normal_shading;
        self.frame_state = Some(frame_state);

        // Render output window directly here — request_redraw() is unreliable
        // for unfocused windows on Windows.
        self.render_output();

        // Control window still uses event-driven redraw (it's typically focused)
        if let Some(control) = &self.control {
            control.window.request_redraw();
        }
    }
}

impl App {
    fn handle_output_event(&mut self, event: WindowEvent) {
        let device = match &self.device {
            Some(d) => d.clone(),
            None => return,
        };

        match event {
            WindowEvent::Resized(new_size) => {
                if let Some(output) = &mut self.output {
                    if new_size.width > 0 && new_size.height > 0 {
                        output.surface_config.width = new_size.width;
                        output.surface_config.height = new_size.height;
                        output.surface.configure(&device, &output.surface_config);
                    }
                }
            }
            WindowEvent::MouseInput { button: winit::event::MouseButton::Left, state, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        self.orbit_dragging = true;
                    }
                    winit::event::ElementState::Released => {
                        self.orbit_dragging = false;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let current = (position.x, position.y);
                if self.orbit_dragging {
                    if let Some((lx, ly)) = self.last_cursor_pos {
                        let dx = current.0 - lx;
                        let dy = current.1 - ly;
                        // Orbit sensitivity: 0.01 radians per pixel
                        self.preset.camera.position.orbit_x.param.value += dx * 0.01;
                        self.preset.camera.position.orbit_y.param.value += dy * 0.01;
                    }
                }
                self.last_cursor_pos = Some(current);
            }
            WindowEvent::CursorLeft { .. } => {
                self.orbit_dragging = false;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y / 100.0,
                };
                self.preset.camera.position.distance.param.value -= scroll * 0.1;
                // Clamp to distance param range
                let d = &self.preset.camera.position.distance.param;
                self.preset.camera.position.distance.param.value = d.value.clamp(d.min, d.max);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        match code {
                            winit::keyboard::KeyCode::KeyF => {
                                self.overlay.show_fps = !self.overlay.show_fps;
                            }
                            winit::keyboard::KeyCode::KeyS => {
                                self.overlay.show_signal_flow = !self.overlay.show_signal_flow;
                            }
                            winit::keyboard::KeyCode::KeyD => {
                                self.overlay.show_dim_count = !self.overlay.show_dim_count;
                            }
                            winit::keyboard::KeyCode::KeyA => {
                                self.overlay.show_active_layers = !self.overlay.show_active_layers;
                            }
                            winit::keyboard::KeyCode::KeyC => {
                                self.overlay.show_caustic = !self.overlay.show_caustic;
                            }
                            winit::keyboard::KeyCode::KeyV => {
                                // Toggle domain coloring (radio: turns off curl/conformal)
                                self.overlay.show_domain_coloring = !self.overlay.show_domain_coloring;
                                if self.overlay.show_domain_coloring {
                                    self.overlay.show_curl = false;
                                    self.overlay.show_conformal = false;
                                }
                            }
                            winit::keyboard::KeyCode::KeyR => {
                                // Toggle curl (radio: turns off domain/conformal)
                                self.overlay.show_curl = !self.overlay.show_curl;
                                if self.overlay.show_curl {
                                    self.overlay.show_domain_coloring = false;
                                    self.overlay.show_conformal = false;
                                }
                            }
                            winit::keyboard::KeyCode::KeyX => {
                                // Toggle conformal defect (radio: turns off domain/curl)
                                self.overlay.show_conformal = !self.overlay.show_conformal;
                                if self.overlay.show_conformal {
                                    self.overlay.show_domain_coloring = false;
                                    self.overlay.show_curl = false;
                                }
                            }
                            winit::keyboard::KeyCode::KeyN => {
                                // Toggle normal shading (independent)
                                self.overlay.show_normal_shading = !self.overlay.show_normal_shading;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Render the output window from the current frame state.
    /// Called directly from about_to_wait() to avoid Windows focus issues.
    fn render_output(&mut self) {
        let frame_state = match &self.frame_state {
            Some(fs) => fs,
            None => return,
        };
        let device = match &self.device {
            Some(d) => d.clone(),
            None => return,
        };
        let queue = match &self.queue {
            Some(q) => q.clone(),
            None => return,
        };

        // Render the lattice (borrows self.renderer + self.output)
        let vp_w = self.preset.viewport.width;
        let vp_h = self.preset.viewport.height;
        let render_result = if let (Some(renderer), Some(output)) =
            (&mut self.renderer, &self.output)
        {
            match renderer.render(frame_state, vp_w, vp_h, self.current_dt, &output.surface, &output.surface_config) {
                Ok(surface_texture) => {
                    // Spout readback before overlays (clean lattice only)
                    #[cfg(target_os = "windows")]
                    if self.spout_enabled && !self.spout_include_overlays {
                        spout_send(
                            &mut self.spout_sender,
                            renderer,
                            vp_w,
                            vp_h,
                        );
                    }

                    Some(surface_texture)
                }
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    output.surface.configure(&device, &output.surface_config);
                    None
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    eprintln!("Out of GPU memory");
                    None
                }
                Err(e) => {
                    eprintln!("Render error: {e}");
                    None
                }
            }
        } else {
            None
        };

        // Overlay + present (renderer borrow is released)
        if let Some(surface_texture) = render_result {
            self.render_output_overlay(&device, &queue, &surface_texture);

            // Spout readback after overlays — reads from offscreen texture
            #[cfg(target_os = "windows")]
            if self.spout_enabled && self.spout_include_overlays {
                if let Some(renderer) = &mut self.renderer {
                    spout_send(
                        &mut self.spout_sender,
                        renderer,
                        vp_w,
                        vp_h,
                    );
                }
            }

            surface_texture.present();
        }
    }

    /// Render overlay text on top of the output window using a second egui pass.
    fn render_output_overlay(
        &mut self,
        device: &Arc<wgpu::Device>,
        queue: &Arc<wgpu::Queue>,
        surface_texture: &wgpu::SurfaceTexture,
    ) {
        let output = match &self.output {
            Some(o) => o,
            None => return,
        };
        let egui_ctx = match &self.output_egui_ctx {
            Some(c) => c.clone(),
            None => return,
        };
        let egui_state = match &mut self.output_egui_state {
            Some(s) => s,
            None => return,
        };

        // Check if any overlays are visible
        if !self.overlay.show_signal_flow && !self.overlay.show_dim_count
            && !self.overlay.show_active_layers && !self.overlay.show_fps
        {
            return;
        }

        // Build overlay content
        let raw_input = egui_state.take_egui_input(&output.window);

        // Collect overlay data before the closure
        let signal_flow_text = if self.overlay.show_signal_flow {
            use polyform_core::notation::{composition_string, CameraFlags};
            let flags = CameraFlags {
                quat: self.preset.quaternion.on,
                az: self.preset.camera.position.orbit_x.on,
                el: self.preset.camera.position.orbit_y.on,
                d: self.preset.camera.position.distance.on,
                tx: self.preset.camera.position.target_x.on,
                ty: self.preset.camera.position.target_y.on,
                roll: self.preset.camera.position.roll.on,
                f: self.preset.camera.lens.focal_length.on,
                sensor: self.preset.camera.lens.sensor_size.on,
                ortho: self.preset.camera.lens.ortho_zoom.on,
                anam: self.preset.camera.lens.anamorphic.on,
                sx: self.preset.camera.lens.shift_x.on,
                sy: self.preset.camera.lens.shift_y.on,
                k1: self.preset.camera.distortion.k1.on,
                k2: self.preset.camera.distortion.k2.on,
            };
            Some(composition_string(
                &self.preset.pipeline.operations,
                &self.preset.pipeline.lift,
                &self.registry,
                &flags,
            ))
        } else {
            None
        };

        // Compute DOFs once — used by both dim count and active layers overlays
        use polyform_core::notation::{count_dofs, ViewParam};
        let dof = if self.overlay.show_dim_count || self.overlay.show_active_layers {
            let mut view_params = Vec::new();
            let view_entries: Vec<(&str, bool, bool)> = vec![
                ("azimuth", self.preset.camera.position.orbit_x.on, self.preset.camera.position.orbit_x.param.anim),
                ("elevation", self.preset.camera.position.orbit_y.on, self.preset.camera.position.orbit_y.param.anim),
                ("distance", self.preset.camera.position.distance.on, self.preset.camera.position.distance.param.anim),
                ("target x", self.preset.camera.position.target_x.on, self.preset.camera.position.target_x.param.anim),
                ("target y", self.preset.camera.position.target_y.on, self.preset.camera.position.target_y.param.anim),
                ("roll", self.preset.camera.position.roll.on, self.preset.camera.position.roll.param.anim),
                ("focal length", self.preset.camera.lens.focal_length.on, self.preset.camera.lens.focal_length.param.anim),
                ("sensor size", self.preset.camera.lens.sensor_size.on, self.preset.camera.lens.sensor_size.param.anim),
                ("ortho zoom", self.preset.camera.lens.ortho_zoom.on, self.preset.camera.lens.ortho_zoom.param.anim),
                ("anamorphic", self.preset.camera.lens.anamorphic.on, self.preset.camera.lens.anamorphic.param.anim),
                ("shift x", self.preset.camera.lens.shift_x.on, self.preset.camera.lens.shift_x.param.anim),
                ("shift y", self.preset.camera.lens.shift_y.on, self.preset.camera.lens.shift_y.param.anim),
                ("k1", self.preset.camera.distortion.k1.on, self.preset.camera.distortion.k1.param.anim),
                ("k2", self.preset.camera.distortion.k2.on, self.preset.camera.distortion.k2.param.anim),
            ];
            for (label, on, anim) in view_entries {
                view_params.push(ViewParam {
                    label: label.to_string(),
                    on,
                    structural: 1,
                    anim,
                });
            }
            Some(count_dofs(
                &self.preset.pipeline.operations,
                &self.preset.pipeline.lift,
                &view_params,
                &self.registry,
            ))
        } else {
            None
        };

        let dim_count_text = if self.overlay.show_dim_count {
            dof.as_ref().map(|d| format!("dim {} ({}S + {}T)", d.total, d.structural, d.temporal))
        } else {
            None
        };

        let active_layers_text = if self.overlay.show_active_layers {
            if let Some(ref d) = dof {
                if d.active.is_empty() {
                    Some("(no active layers)".to_string())
                } else {
                    let lines: Vec<String> = d.active.iter().map(|entry| {
                        let dims = entry.structural + entry.temporal;
                        if entry.temporal > 0 {
                            format!("{}: {}d ({}S+{}T)", entry.label, dims, entry.structural, entry.temporal)
                        } else {
                            format!("{}: {}S", entry.label, entry.structural)
                        }
                    }).collect();
                    Some(lines.join("\n"))
                }
            } else {
                None
            }
        } else {
            None
        };

        let fps_text = if self.overlay.show_fps {
            Some(format!("{:.0} fps", self.fps_counter.fps()))
        } else {
            None
        };

        let full_output = egui_ctx.run(raw_input, |ctx| {
            // Semi-transparent background style for overlay text
            let overlay_frame = egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(160))
                .inner_margin(egui::Margin::same(6));

            // Signal flow: bottom-left
            if let Some(ref text) = signal_flow_text {
                egui::Area::new(egui::Id::new("signal_flow_overlay"))
                    .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0))
                    .interactable(false)
                    .show(ctx, |ui| {
                        overlay_frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(text.as_str())
                                .monospace()
                                .color(egui::Color32::from_rgb(200, 200, 200))
                                .size(13.0));
                        });
                    });
            }

            // Active layers: top-left
            if let Some(ref text) = active_layers_text {
                egui::Area::new(egui::Id::new("active_layers_overlay"))
                    .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
                    .interactable(false)
                    .show(ctx, |ui| {
                        overlay_frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(text.as_str())
                                .color(egui::Color32::from_rgb(200, 200, 200))
                                .size(12.0));
                        });
                    });
            }

            // Dim count: top-right
            if let Some(ref text) = dim_count_text {
                egui::Area::new(egui::Id::new("dim_count_overlay"))
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
                    .interactable(false)
                    .show(ctx, |ui| {
                        overlay_frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(text.as_str())
                                .color(egui::Color32::from_rgb(200, 200, 200))
                                .size(13.0));
                        });
                    });
            }

            // FPS: bottom-right
            if let Some(ref text) = fps_text {
                egui::Area::new(egui::Id::new("fps_overlay"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-8.0, -8.0))
                    .interactable(false)
                    .show(ctx, |ui| {
                        overlay_frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(text.as_str())
                                .monospace()
                                .color(egui::Color32::from_rgb(200, 200, 200))
                                .size(12.0));
                        });
                    });
            }
        });

        let egui_renderer = match &mut self.output_egui_renderer {
            Some(r) => r,
            None => return,
        };

        // Tessellate and render
        let tris = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(device, queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [output.surface_config.width, output.surface_config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("overlay_encoder"),
        });

        egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear — overlay on top
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );
        }

        queue.submit(std::iter::once(encoder.finish()));

        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }
    }

    fn handle_control_event(&mut self, event: WindowEvent) {
        let device = match &self.device {
            Some(d) => d.clone(),
            None => return,
        };
        let queue = match &self.queue {
            Some(q) => q.clone(),
            None => return,
        };

        // Take control out temporarily so we can access self.preset/registry
        // independently of self.control
        let mut control = match self.control.take() {
            Some(c) => c,
            None => return,
        };

        // Forward event to egui
        let response = control.egui_state.on_window_event(&control.window, &event);

        match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    control.surface_config.width = new_size.width;
                    control.surface_config.height = new_size.height;
                    control.surface.configure(&device, &control.surface_config);
                }
            }
            WindowEvent::RedrawRequested => {
                // Build egui UI and collect actions
                let raw_input = control.egui_state.take_egui_input(&control.window);
                let ui_actions = std::cell::RefCell::new(Vec::new());
                let frame_state_ref = self.frame_state.as_ref();
                let overlay_cell = std::cell::RefCell::new(&mut self.overlay);
                let full_output = control.egui_ctx.run(raw_input, |ctx| {
                    let actions = ui::build_ui(
                        ctx,
                        &self.preset,
                        &self.registry,
                        frame_state_ref,
                        *overlay_cell.borrow_mut(),
                    );
                    *ui_actions.borrow_mut() = actions;
                });
                let ui_actions = ui_actions.into_inner();

                // Put control back, process actions, then get it again for rendering
                self.control = Some(control);
                self.process_ui_actions(ui_actions);
                let control = self.control.as_mut().unwrap();

                // Handle egui platform output
                control
                    .egui_state
                    .handle_platform_output(&control.window, full_output.platform_output);

                // Render egui
                let tris = control
                    .egui_ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);
                for (id, image_delta) in &full_output.textures_delta.set {
                    control
                        .egui_renderer
                        .update_texture(&device, &queue, *id, image_delta);
                }

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [control.surface_config.width, control.surface_config.height],
                    pixels_per_point: full_output.pixels_per_point,
                };

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("egui_encoder"),
                    });

                control.egui_renderer.update_buffers(
                    &device,
                    &queue,
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                let output_texture = match control.surface.get_current_texture() {
                    Ok(t) => t,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        control.surface.configure(&device, &control.surface_config);
                        return;
                    }
                    Err(e) => {
                        eprintln!("Control surface error: {e}");
                        return;
                    }
                };
                let view = output_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                {
                    let render_pass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("egui_render_pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.1,
                                        b: 0.1,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                    control.egui_renderer.render(
                        &mut render_pass.forget_lifetime(),
                        &tris,
                        &screen_descriptor,
                    );
                }

                queue.submit(std::iter::once(encoder.finish()));
                output_texture.present();

                for id in &full_output.textures_delta.free {
                    control.egui_renderer.free_texture(id);
                }
                return; // control is already in self.control from the put-back above
            }
            _ => {
                if response.repaint {
                    control.window.request_redraw();
                }
            }
        }

        // Put control back for non-RedrawRequested events
        self.control = Some(control);
    }

    // =========================================================================
    // UI Action dispatch
    // =========================================================================

    fn process_ui_actions(&mut self, actions: Vec<UiAction>) {
        for action in actions {
            match action {
                UiAction::SetParam(ref field, value) => {
                    // If this is a blend slider drag, cancel any active
                    // blend transition so the user's input isn't overridden.
                    if let ParamField::LayerBlend { kind, layer_id } = field {
                        self.engine.cancel_blend_transition(*kind, layer_id);
                    }
                    self.set_param_field(field, value);
                }
                UiAction::SetParamAnim(field, anim) => {
                    let old_anim = self.resolve_param_mut(&field).map(|p| p.anim);

                    if !anim && old_anim == Some(true) {
                        // Turning anim OFF: freeze at current swept value
                        if let Some(swept) = self.lookup_swept_value(&field) {
                            if let Some(param) = self.resolve_param_mut(&field) {
                                param.value = swept;
                            }
                        }
                    }

                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.anim = anim;
                    }

                    // Weight transitions for smooth anim on/off.
                    // ToggleParams use a ".anim" suffix path so the anim weight
                    // doesn't conflict with the on/off toggle weight.
                    let path = match &field {
                        ParamField::View(vk) if !vk.starts_with("quaternion.") => {
                            polyform_core::animation::ParamPath::View(
                                format!("{}.anim", vk),
                            )
                        }
                        _ => field_to_param_path(&field),
                    };
                    if anim {
                        if self.engine.has_weight(&path) {
                            self.engine.set_weight_target(path, 1.0, 0.3);
                        } else {
                            self.engine.start_weight_transition(path, 0.0, 1.0, 0.3);
                        }
                    } else {
                        self.engine.set_weight_target(path, 0.0, 0.3);
                    }
                }
                UiAction::SetParamShape(field, shape) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.shape = shape;
                    }
                }
                UiAction::SetParamRatio(field, ratio) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.ratio = ratio;
                    }
                }
                UiAction::SetParamMin(field, v) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.min = v;
                    }
                }
                UiAction::SetParamMax(field, v) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.max = v;
                    }
                }
                UiAction::SetParamOffset(field, v) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        param.offset = v;
                    }
                }
                UiAction::SetModulatorDepth(field, depth) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        if let Some(ref mut m) = param.modulator {
                            m.depth = depth;
                        } else if depth.abs() > 1e-12 {
                            param.modulator = Some(PhaseModulator {
                                ratio: 1.0,
                                depth,
                                shape: ShapeFunction::Cosine,
                            });
                        }
                    }
                }
                UiAction::SetModulatorRatio(field, ratio) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        if let Some(ref mut m) = param.modulator {
                            m.ratio = ratio;
                        } else {
                            param.modulator = Some(PhaseModulator {
                                ratio,
                                depth: 0.0,
                                shape: ShapeFunction::Cosine,
                            });
                        }
                    }
                }
                UiAction::SetModulatorShape(field, shape) => {
                    if let Some(param) = self.resolve_param_mut(&field) {
                        if let Some(ref mut m) = param.modulator {
                            m.shape = shape;
                        } else {
                            param.modulator = Some(PhaseModulator {
                                ratio: 1.0,
                                depth: 0.0,
                                shape,
                            });
                        }
                    }
                }
                UiAction::SetLayerEnabled(kind, layer_id, enabled) => {
                    let layers = match kind {
                        PipelineKind::Operations => &self.preset.pipeline.operations,
                        PipelineKind::Lift => &self.preset.pipeline.lift,
                    };
                    // Only toggle if state actually changes
                    if let Some(layer) = layers.iter().find(|l| l.id == layer_id) {
                        if layer.enabled != enabled {
                            let blend_time = self.preset.defaults.blend_time;
                            self.engine.toggle_layer(
                                &mut self.preset,
                                kind,
                                &layer_id,
                                blend_time,
                            );
                        }
                    }
                }
                UiAction::AddLayer(kind, type_key) => {
                    if let Some(mut layer) = make_layer(&type_key, &self.registry) {
                        // Start at blend=0 and fade in smoothly
                        let layer_id = layer.id.clone();
                        layer.blend.value = 0.0;
                        match kind {
                            PipelineKind::Operations => {
                                self.preset.pipeline.operations.push(layer)
                            }
                            PipelineKind::Lift => self.preset.pipeline.lift.push(layer),
                        }
                        let blend_time = self.preset.defaults.blend_time;
                        let target = TransitionTarget::LayerBlend {
                            kind,
                            layer_id: layer_id.clone(),
                        };
                        self.engine.transitions.start(
                            target,
                            0.0,
                            1.0,
                            blend_time,
                            Some(TransitionAction::RestoreBlendValue {
                                kind,
                                layer_id,
                                value: 1.0,
                            }),
                            None,
                        );
                    }
                }
                UiAction::RemoveLayer(kind, idx) => {
                    let layers = match kind {
                        PipelineKind::Operations => &mut self.preset.pipeline.operations,
                        PipelineKind::Lift => &mut self.preset.pipeline.lift,
                    };
                    if idx < layers.len() {
                        layers.remove(idx);
                    }
                }
                UiAction::MoveLayer(kind, idx, direction) => {
                    let layers = match kind {
                        PipelineKind::Operations => &mut self.preset.pipeline.operations,
                        PipelineKind::Lift => &mut self.preset.pipeline.lift,
                    };
                    let new_idx = (idx as i32 + direction) as usize;
                    if new_idx < layers.len() {
                        layers.swap(idx, new_idx);
                    }
                }
                UiAction::ChangeLayerType(kind, idx, new_type_key) => {
                    if let Some(new_layer) = make_layer(&new_type_key, &self.registry) {
                        let layers = match kind {
                            PipelineKind::Operations => &mut self.preset.pipeline.operations,
                            PipelineKind::Lift => &mut self.preset.pipeline.lift,
                        };
                        if idx < layers.len() {
                            let old_id = layers[idx].id.clone();
                            let old_enabled = layers[idx].enabled;
                            let old_blend = layers[idx].blend.clone();
                            layers[idx] = new_layer;
                            layers[idx].id = old_id;
                            layers[idx].enabled = old_enabled;
                            layers[idx].blend = old_blend;
                        }
                    }
                }
                UiAction::SetPalette(key) => {
                    if PALETTE_KEYS.contains(&key.as_str()) {
                        self.preset.palette = key;
                    }
                }
                UiAction::SetProjection(mode) => {
                    self.preset.camera.lens.projection = mode;
                }
                UiAction::SetQuatOn(on) => {
                    let blend_time = self.preset.defaults.blend_time;
                    let path = polyform_core::animation::ParamPath::View(
                        "quaternion.blend".to_string(),
                    );
                    if on && !self.preset.quaternion.on {
                        // Turning ON: ramp blend weight toward 1
                        self.preset.quaternion.on = true;
                        if self.engine.has_weight(&path) {
                            // Mid-transition: ramp from current value
                            self.engine.set_weight_target(path, 1.0, blend_time);
                        } else {
                            self.engine.start_weight_transition(
                                path, 0.0, 1.0, blend_time,
                            );
                        }
                    } else if !on && self.preset.quaternion.on {
                        // Turning OFF: ramp blend weight toward 0
                        // (angle * blend smoothly reaches 0 = identity)
                        self.preset.quaternion.on = false;
                        if self.engine.has_weight(&path) {
                            self.engine.set_weight_target(path, 0.0, blend_time);
                        } else {
                            self.engine.start_weight_transition(
                                path, 1.0, 0.0, blend_time,
                            );
                        }
                    }
                }
                UiAction::SetQuatMode(mode) => {
                    // Capture current rendered quaternion for smooth transition
                    if let Some(ref fs) = self.frame_state {
                        self.engine.set_prev_mode_quat(Some(fs.quat));
                    }
                    let blend_time = self.preset.defaults.blend_time;
                    let path = polyform_core::animation::ParamPath::View(
                        "quaternion.mode_blend".to_string(),
                    );
                    self.engine.start_weight_transition(path, 0.0, 1.0, blend_time);
                    self.preset.quaternion.mode = mode;
                }
                UiAction::SetQuatDriftSpeed(speed) => {
                    self.preset.quaternion.drift_speed = speed;
                }
                UiAction::ParamGrabStart(ref field) => {
                    // User started dragging an animated slider.
                    // Freeze param.value to current swept output so the slider
                    // snaps to where the animation was, then set weight to 0
                    // so the static value is used while dragging.
                    if let Some(swept) = self.lookup_swept_value(field) {
                        if let Some(param) = self.resolve_param_mut(field) {
                            param.value = swept;
                        }
                    }
                    // Determine the weight path
                    let path = match field {
                        ParamField::View(vk) if !vk.starts_with("quaternion.") => {
                            polyform_core::animation::ParamPath::View(
                                format!("{}.anim", vk),
                            )
                        }
                        _ => field_to_param_path(field),
                    };
                    // Instant weight to 0 (no transition duration)
                    self.engine.start_weight_transition(path, 0.0, 0.0, 0.0);
                }
                UiAction::ParamGrabEnd(ref field) => {
                    // User released the slider. Smoothly return to animation.
                    let path = match field {
                        ParamField::View(vk) if !vk.starts_with("quaternion.") => {
                            polyform_core::animation::ParamPath::View(
                                format!("{}.anim", vk),
                            )
                        }
                        _ => field_to_param_path(field),
                    };
                    self.engine.start_weight_transition(path, 0.0, 1.0, 0.3);
                }
                UiAction::SetToggleParamOn(view_key, on) => {
                    // Toggle on/off uses only the weight system.
                    // resolve_toggle_param blends between neutral (weight=0)
                    // and active value (weight=1), so we just ramp the weight.
                    let blend_time = self.preset.defaults.blend_time;
                    let path = polyform_core::animation::ParamPath::View(
                        view_key.clone(),
                    );

                    // Cancel any lingering transition overlay
                    let target_key = TransitionTarget::ViewParam(view_key.clone());
                    self.engine.transitions.cancel_for(&target_key);
                    self.engine.transition_overlay.remove(&target_key);

                    if let Some(tp) = resolve_toggle_param_mut(&mut self.preset, &view_key) {
                        tp.on = on;
                    }

                    let target = if on { 1.0 } else { 0.0 };
                    if self.engine.has_weight(&path) {
                        // Mid-transition: ramp from current weight
                        self.engine.set_weight_target(path, target, blend_time);
                    } else {
                        // First toggle: start from the opposite end
                        let from = if on { 0.0 } else { 1.0 };
                        self.engine.start_weight_transition(
                            path, from, target, blend_time,
                        );
                    }
                }
                UiAction::SetPresetName(name) => {
                    self.preset.metadata.name = name;
                }
                UiAction::SavePreset => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Polyform Preset", &["json"])
                        .save_file()
                    {
                        let json = export_preset_json(&self.preset, true);
                        if let Err(e) = std::fs::write(&path, json) {
                            eprintln!("Failed to save preset: {e}");
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                UiAction::SetSpoutEnabled(enabled) => {
                    self.spout_enabled = enabled;
                    if !enabled {
                        // Drop the sender to release D3D11 resources
                        self.spout_sender = None;
                    }
                }
                #[cfg(target_os = "windows")]
                UiAction::SetSpoutOverlays(include) => {
                    self.spout_include_overlays = include;
                }
                UiAction::SetViewportSize(w, h) => {
                    self.preset.viewport.width = w;
                    self.preset.viewport.height = h;
                }
                UiAction::SetLatticeType(ty) => {
                    self.preset.sampling.lattice_type = ty;
                }
                UiAction::SetLatticeSpokeCount(n) => {
                    self.preset.sampling.lattice_params.spoke_count = n;
                }
                UiAction::SetLatticeInnerRadius(v) => {
                    self.preset.sampling.lattice_params.inner_radius = v;
                }
                UiAction::SetLatticeFocusDistance(v) => {
                    self.preset.sampling.lattice_params.focus_distance = v;
                }
                UiAction::SetLatticeTauCount(n) => {
                    self.preset.sampling.lattice_params.tau_count = n;
                }
                UiAction::LoadPreset => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Polyform Preset", &["json"])
                        .pick_file()
                    {
                        if let Ok(json) = std::fs::read_to_string(&path) {
                            if let Ok(preset) = serde_json::from_str::<Preset>(&json) {
                                self.preset = preset;
                                self.engine.reset();
                            } else {
                                eprintln!("Failed to parse preset from {}", path.display());
                            }
                        } else {
                            eprintln!("Failed to read file {}", path.display());
                        }
                    }
                }
            }
        }
    }

    fn set_param_field(&mut self, field: &ParamField, value: f64) {
        match field {
            ParamField::LayerParam {
                kind,
                layer_id,
                key,
            } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut self.preset.pipeline.operations,
                    PipelineKind::Lift => &mut self.preset.pipeline.lift,
                };
                if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                    if let Some(p) = layer.params.get_mut(key.as_str()) {
                        p.value = value;
                    }
                }
            }
            ParamField::LayerBlend { kind, layer_id } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut self.preset.pipeline.operations,
                    PipelineKind::Lift => &mut self.preset.pipeline.lift,
                };
                if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                    layer.blend.value = value;
                }
            }
            ParamField::Tempo => {
                self.preset.tempo.value = value;
            }
            ParamField::View(view_key) => {
                // Handle quaternion params directly
                match view_key.as_str() {
                    "quaternion.angle" => { self.preset.quaternion.angle.value = value; return; }
                    "quaternion.ax" => { self.preset.quaternion.ax.value = value; return; }
                    "quaternion.ay" => { self.preset.quaternion.ay.value = value; return; }
                    "quaternion.az" => { self.preset.quaternion.az.value = value; return; }
                    _ => {}
                }
                if let Some(tp) = resolve_toggle_param_mut(&mut self.preset, view_key) {
                    tp.param.value = value;
                }
            }
        }
    }

    fn resolve_param_mut(
        &mut self,
        field: &ParamField,
    ) -> Option<&mut polyform_core::types::Param> {
        match field {
            ParamField::LayerParam {
                kind,
                layer_id,
                key,
            } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut self.preset.pipeline.operations,
                    PipelineKind::Lift => &mut self.preset.pipeline.lift,
                };
                layers
                    .iter_mut()
                    .find(|l| l.id == *layer_id)
                    .and_then(|l| l.params.get_mut(key.as_str()))
            }
            ParamField::LayerBlend { kind, layer_id } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut self.preset.pipeline.operations,
                    PipelineKind::Lift => &mut self.preset.pipeline.lift,
                };
                layers
                    .iter_mut()
                    .find(|l| l.id == *layer_id)
                    .map(|l| &mut l.blend)
            }
            ParamField::Tempo => Some(&mut self.preset.tempo),
            ParamField::View(view_key) => {
                // Handle quaternion params directly (they're full Params, not ToggleParams)
                match view_key.as_str() {
                    "quaternion.angle" => return Some(&mut self.preset.quaternion.angle),
                    "quaternion.ax" => return Some(&mut self.preset.quaternion.ax),
                    "quaternion.ay" => return Some(&mut self.preset.quaternion.ay),
                    "quaternion.az" => return Some(&mut self.preset.quaternion.az),
                    _ => {}
                }
                resolve_toggle_param_mut(&mut self.preset, view_key).map(|tp| &mut tp.param)
            }
        }
    }

    /// Look up the current swept/visual value for a param from the latest frame state.
    fn lookup_swept_value(&self, field: &ParamField) -> Option<f64> {
        let fs = self.frame_state.as_ref()?;
        match field {
            ParamField::LayerParam { kind, layer_id, key } => {
                let layers = match kind {
                    PipelineKind::Operations => &self.preset.pipeline.operations,
                    PipelineKind::Lift => &self.preset.pipeline.lift,
                };
                let idx = layers.iter().position(|l| l.id == *layer_id)?;
                let flat_layers = match kind {
                    PipelineKind::Operations => &fs.operations,
                    PipelineKind::Lift => &fs.lifts,
                };
                flat_layers.get(idx)?.params.get(key.as_str()).copied()
            }
            ParamField::LayerBlend { kind, layer_id } => {
                let layers = match kind {
                    PipelineKind::Operations => &self.preset.pipeline.operations,
                    PipelineKind::Lift => &self.preset.pipeline.lift,
                };
                let idx = layers.iter().position(|l| l.id == *layer_id)?;
                let flat_layers = match kind {
                    PipelineKind::Operations => &fs.operations,
                    PipelineKind::Lift => &fs.lifts,
                };
                Some(flat_layers.get(idx)?.blend)
            }
            ParamField::View(view_key) => {
                frame_state_view_value(fs, view_key)
            }
            ParamField::Tempo => None, // tempo isn't swept
        }
    }
}

// =============================================================================
// View param resolution
// =============================================================================

fn resolve_toggle_param_mut<'a>(
    preset: &'a mut Preset,
    name: &str,
) -> Option<&'a mut ToggleParam> {
    match name {
        "camera.position.orbitX" => Some(&mut preset.camera.position.orbit_x),
        "camera.position.orbitY" => Some(&mut preset.camera.position.orbit_y),
        "camera.position.distance" => Some(&mut preset.camera.position.distance),
        "camera.position.targetX" => Some(&mut preset.camera.position.target_x),
        "camera.position.targetY" => Some(&mut preset.camera.position.target_y),
        "camera.position.roll" => Some(&mut preset.camera.position.roll),
        "camera.lens.focalLength" => Some(&mut preset.camera.lens.focal_length),
        "camera.lens.sensorSize" => Some(&mut preset.camera.lens.sensor_size),
        "camera.lens.orthoZoom" => Some(&mut preset.camera.lens.ortho_zoom),
        "camera.lens.anamorphic" => Some(&mut preset.camera.lens.anamorphic),
        "camera.lens.shiftX" => Some(&mut preset.camera.lens.shift_x),
        "camera.lens.shiftY" => Some(&mut preset.camera.lens.shift_y),
        "camera.distortion.k1" => Some(&mut preset.camera.distortion.k1),
        "camera.distortion.k2" => Some(&mut preset.camera.distortion.k2),
        "camera.distortion.k3" => Some(&mut preset.camera.distortion.k3),
        "camera.distortion.k4" => Some(&mut preset.camera.distortion.k4),
        "camera.distortion.p1" => Some(&mut preset.camera.distortion.p1),
        "camera.distortion.p2" => Some(&mut preset.camera.distortion.p2),
        "camera.distortion.centerX" => Some(&mut preset.camera.distortion.center_x),
        "camera.distortion.centerY" => Some(&mut preset.camera.distortion.center_y),
        "camera.distortion.squeeze" => Some(&mut preset.camera.distortion.squeeze),
        "sampling.scale" => Some(&mut preset.sampling.scale),
        "sampling.density" => Some(&mut preset.sampling.density),
        "phosphor.persistence" => Some(&mut preset.phosphor.persistence),
        "phosphor.beamWidth" => Some(&mut preset.phosphor.beam_width),
        "phosphor.beamIntensity" => Some(&mut preset.phosphor.beam_intensity),
        "phosphor.gain" => Some(&mut preset.phosphor.gain),
        "phosphor.blendMix" => Some(&mut preset.phosphor.blend_mix),
        "phosphor.saturation" => Some(&mut preset.phosphor.saturation),
        "phosphor.bloomIntensity" => Some(&mut preset.phosphor.bloom_intensity),
        "phosphor.bloomRadius" => Some(&mut preset.phosphor.bloom_radius),
        "phosphor.bloomThreshold" => Some(&mut preset.phosphor.bloom_threshold),
        "phosphor.gamma" => Some(&mut preset.phosphor.gamma),
        _ => None,
    }
}

// =============================================================================
// ParamField -> ParamPath conversion
// =============================================================================

fn field_to_param_path(field: &ParamField) -> polyform_core::animation::ParamPath {
    use polyform_core::animation::ParamPath;
    match field {
        ParamField::LayerParam { kind, layer_id, key } => ParamPath::LayerParam {
            kind: *kind,
            layer_id: layer_id.clone(),
            key: key.clone(),
        },
        ParamField::LayerBlend { kind, layer_id } => ParamPath::LayerBlend {
            kind: *kind,
            layer_id: layer_id.clone(),
        },
        ParamField::Tempo => ParamPath::Tempo,
        ParamField::View(view_key) => ParamPath::View(view_key.clone()),
    }
}

// =============================================================================
// Spout helper (free function to avoid borrow conflicts)
// =============================================================================

#[cfg(target_os = "windows")]
fn spout_send(
    sender: &mut Option<spout::SpoutSender>,
    renderer: &mut GpuRenderer,
    w: u32,
    h: u32,
) {
    // Lazily create sender if dimensions changed or not yet created
    let needs_new = match sender {
        Some(s) => s.width() != w || s.height() != h,
        None => true,
    };
    if needs_new {
        *sender = None;
        *sender = spout::SpoutSender::new("Polyform", w, h);
        if sender.is_none() {
            log::warn!("Failed to create Spout sender");
            return;
        }
    }

    // Read back the offscreen texture at viewport resolution
    if let Some(pixels) = renderer.readback_offscreen(w, h) {
        if let Some(s) = sender {
            s.send_image(&pixels);
        }
    }
}

// =============================================================================
// Entry point
// =============================================================================

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
