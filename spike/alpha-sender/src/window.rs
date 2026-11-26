//! The windowed path: a transparent, undecorated window over an opaque backdrop this process
//! also owns, so the colour behind the test frame is known rather than whatever was on screen.
//!
//! Expected composite values are in `REFERENCE.md`, "What the compositor must show".

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::{report, HEIGHT, WIDTH};

/// Opaque backdrop colour. Channels are 0 or 255 so every expected composite is an exact
/// integer — see `REFERENCE.md`.
const BACKDROP: wgpu::Color = wgpu::Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

/// Margin of backdrop visible around the test window, so a screenshot can confirm the backdrop
/// is actually where we think it is before trusting any probe inside it.
const MARGIN: u32 = 40;

struct Surface {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

pub struct App {
    straight: bool,
    srgb: bool,
    solo: bool,
    chosen_format: Option<wgpu::TextureFormat>,
    instance: wgpu::Instance,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    pipeline: Option<wgpu::RenderPipeline>,
    backdrop: Option<Surface>,
    test: Option<Surface>,
    reported: bool,
}

impl App {
    pub fn new(backends: wgpu::Backends, straight: bool, srgb: bool, solo: bool) -> Self {
        Self {
            straight,
            srgb,
            solo,
            chosen_format: None,
            instance: {
                let mut d = wgpu::InstanceDescriptor::new_without_display_handle();
                d.backends = backends;
                wgpu::Instance::new(d)
            },
            device: None,
            queue: None,
            pipeline: None,
            backdrop: None,
            test: None,
            reported: false,
        }
    }

    fn make(
        &self,
        event_loop: &ActiveEventLoop,
        title: &str,
        w: u32,
        h: u32,
        transparent: bool,
    ) -> Arc<Window> {
        let attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(w, h))
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(transparent);
        Arc::new(event_loop.create_window(attrs).expect("create window"))
    }

    fn format(&self) -> wgpu::TextureFormat {
        // The experiment: vkcapture reports the swapchain's colour space to OBS, but the dmabuf
        // carries a DRM format. If the format is non-sRGB the sampler does no decode while OBS
        // still encodes on output, which is the 128 -> 188 shift. An sRGB format should make the
        // two agree.
        if self.srgb {
            wgpu::TextureFormat::Bgra8UnormSrgb
        } else {
            wgpu::TextureFormat::Bgra8Unorm
        }
    }

    fn configure(&self, window: Arc<Window>, device: &wgpu::Device, alpha: wgpu::CompositeAlphaMode) -> Surface {
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("create surface");
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // Linear, never sRGB. REFERENCE.md explains why every number depends on it.
            format: self.chosen_format.unwrap_or_else(|| self.format()),
            // Auto, which resolves to Srgb for this format. That is what a real application
            // gets, so it is what the spike must measure. The format itself is linear, so the
            // shader's values reach the surface unencoded; the colour space only tells the
            // compositor how to read them. Recorded because it is a wgpu 30 knob that did not
            // exist before and sits directly on top of what this spike measures.
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: alpha,
            view_formats: vec![],
        };
        surface.configure(device, &config);
        Surface {
            window,
            surface,
            config,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.device.is_some() {
            return;
        }

        // Backdrop first so the test window stacks above it. Skipped in solo mode: a capture
        // route grabs a surface directly and needs no backdrop, and leaving a second window
        // around lets the route capture the wrong one — which it did, silently, until the
        // harness's own size check caught an 848x336 frame of flat magenta.
        let backdrop_win = if self.solo {
            None
        } else {
            Some(self.make(
                event_loop,
                "alpha-sender backdrop",
                WIDTH + MARGIN * 2,
                HEIGHT + MARGIN * 2,
                false,
            ))
        };
        let test_win = self.make(event_loop, "alpha-sender test", WIDTH, HEIGHT, true);

        let surface = self
            .instance
            .create_surface(test_win.clone())
            .expect("probe surface");
        let adapter = pollster::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            },
        ))
        .expect("no adapter");

        let caps = surface.get_capabilities(&adapter);
        let info = adapter.get_info();

        // Everything the platform offers, printed before anything can fail. On Windows the
        // interesting failure is a surface that cannot carry alpha at all, and a crash with no
        // output is indistinguishable from a crash for some other reason.
        report("adapter", &format!("{} ({:?}, {:?})", info.name, info.device_type, info.backend));
        report("surface formats", &format!("{:?}", caps.formats));
        report("surface alpha modes", &format!("{:?}", caps.alpha_modes));
        report("surface present modes", &format!("{:?}", caps.present_modes));

        if caps.alpha_modes.is_empty() || caps.formats.is_empty() {
            report("FATAL", "the surface offers no formats or no alpha modes; cannot continue");
            event_loop.exit();
            return;
        }

        // Which alpha modes the surface offers is itself a finding: if PreMultiplied is absent,
        // the platform cannot carry premultiplied frames through the swapchain at all.
        let want = if self.straight {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else {
            wgpu::CompositeAlphaMode::PreMultiplied
        };
        let alpha = if caps.alpha_modes.contains(&want) {
            want
        } else {
            report(
                "WARNING",
                &format!(
                    "{:?} is not offered; falling back to {:?}. A transparent window is then \
                     impossible through the swapchain, and that is the result, not a bug.",
                    want, caps.alpha_modes[0]
                ),
            );
            caps.alpha_modes[0]
        };
        report("using alpha mode", &format!("{alpha:?}"));

        // Configuring with an unsupported format is a validation failure that aborts the
        // process, which is what a bare crash with no message looks like from outside.
        let wanted_format = self.format();
        let format = if caps.formats.contains(&wanted_format) {
            wanted_format
        } else {
            report(
                "WARNING",
                &format!(
                    "{:?} is not offered; falling back to {:?}. Every number in REFERENCE.md \
                     assumes a linear format, so scores against it are suspect.",
                    wanted_format, caps.formats[0]
                ),
            );
            caps.formats[0]
        };
        report("using format", &format!("{format:?}"));
        self.chosen_format = Some(format);

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("alpha-sender window"),
            ..Default::default()
        }))
        .expect("device");

        self.pipeline = Some(crate::build_pipeline(&device, format, self.straight));
        drop(surface);
        // Place the test window exactly MARGIN inside the backdrop. Without this the window
        // manager positions the two independently, the visible margin is whatever it happens to
        // be, and anything deriving the test frame's position from the backdrop reads the wrong
        // pixels. The Windows leg has no window coordinates to work from and locates the frame
        // by finding the backdrop, so this is what makes that possible.
        if let Some(b) = &backdrop_win {
            if let Ok(bp) = b.outer_position() {
                test_win.set_outer_position(winit::dpi::PhysicalPosition::new(
                    bp.x + MARGIN as i32,
                    bp.y + MARGIN as i32,
                ));
            }
        }

        self.backdrop = backdrop_win
            .map(|w| self.configure(w, &device, wgpu::CompositeAlphaMode::Opaque));
        self.test = Some(self.configure(test_win, &device, alpha));
        self.device = Some(device);
        self.queue = Some(queue);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let (Some(device), Some(queue), Some(pipeline)) =
                    (&self.device, &self.queue, &self.pipeline)
                else {
                    return;
                };

                for (s, is_test) in [
                    (self.backdrop.as_ref(), false),
                    (self.test.as_ref(), true),
                ] {
                    let Some(s) = s else { continue };
                    if s.window.id() != id {
                        continue;
                    }
                    let frame = match s.surface.get_current_texture() {
                        wgpu::CurrentSurfaceTexture::Success(f)
                        | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                        other => {
                            eprintln!("surface texture unavailable: {other:?}");
                            return;
                        }
                    };
                    let view = frame.texture.create_view(&Default::default());
                    let mut enc = device.create_command_encoder(&Default::default());
                    {
                        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                depth_slice: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(if is_test {
                                        wgpu::Color::TRANSPARENT
                                    } else {
                                        BACKDROP
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                        if is_test {
                            pass.set_pipeline(pipeline);
                            pass.draw(0..3, 0..1);
                        }
                    }
                    queue.submit([enc.finish()]);
                    // wgpu 30 moved presentation onto the queue; `SurfaceTexture::present` is
                    // gone and a frame that is merely dropped is discarded, not shown.
                    queue.present(frame);
                    let _ = s.config.width;
                }

                // Report geometry once both windows have drawn, so a harness can probe absolute
                // screen coordinates rather than guessing where the WM put us.
                if !self.reported {
                    if let Some(t) = &self.test {
                        if let Ok(tp) = t.window.outer_position() {
                            if let Some(b) = &self.backdrop {
                                if let Ok(bp) = b.window.outer_position() {
                                    println!("BACKDROP_AT {} {}", bp.x, bp.y);
                                } else {
                                    return;
                                }
                            }
                            println!("TEST_AT {} {}", tp.x, tp.y);
                            println!("READY");
                            self.reported = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(b) = &self.backdrop {
            b.window.request_redraw();
        }
        if let Some(t) = &self.test {
            t.window.request_redraw();
        }
    }
}

pub fn run(backends: wgpu::Backends, straight: bool, srgb: bool, solo: bool) {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(backends, straight, srgb, solo);
    event_loop.run_app(&mut app).expect("run");
}
