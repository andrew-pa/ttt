use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::BackendRenderTarget;
use skia_safe::Color4f;
use skia_safe::{ColorType, Surface};

use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::{Window, WindowBuilder};

mod model;
mod presenter;
mod storage;
mod view;

use presenter::Presenter;
use view::View;

pub struct GlWindow {
    // XXX the surface must be dropped before the window.
    pub surface: glutin::surface::Surface<WindowSurface>,

    pub window: Window,
}

impl GlWindow {
    pub fn new(window: Window, config: &Config) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            config
                .display()
                .create_window_surface(config, &attrs)
                .unwrap()
        };

        Self { window, surface }
    }
}

struct Renderer {
    gr_context: skia_safe::gpu::DirectContext,
    surface: Surface,
    num_aa_samples: usize,
}

fn get_fb_info() -> FramebufferInfo {
    let mut fboid = 0;
    unsafe {
        gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid);
    }
    FramebufferInfo {
        fboid: fboid as u32,
        format: skia_safe::gpu::gl::Format::RGBA8.into(),
    }
}

fn create_surface(
    gr_context: &mut skia_safe::gpu::DirectContext,
    size: PhysicalSize<u32>,
    num_aa_samples: usize,
) -> Surface {
    let brt = BackendRenderTarget::new_gl(
        (size.width as i32, size.height as i32),
        Some(num_aa_samples),
        0,
        get_fb_info(),
    );

    Surface::from_backend_render_target(
        gr_context,
        &brt,
        skia_safe::gpu::SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("create Skia surface")
}

impl Renderer {
    fn new(
        gl_display: &glutin::display::Display,
        size: PhysicalSize<u32>,
        num_aa_samples: usize,
    ) -> Self {
        gl::load_with(|s| gl_display.get_proc_address(&CString::new(s).unwrap()));

        let mut gr_context = skia_safe::gpu::DirectContext::new_gl(
            skia_safe::gpu::gl::Interface::new_load_with(|s| {
                gl_display.get_proc_address(&CString::new(s).unwrap())
            }),
            None,
        )
        .expect("create Skia context");

        let surface = create_surface(&mut gr_context, size, num_aa_samples);

        Renderer {
            gr_context,
            surface,
            num_aa_samples,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface = create_surface(&mut self.gr_context, size, self.num_aa_samples);
    }

    fn render(&mut self, scale_factor: f64, view: &View, canvas_size: LogicalSize<f32>) {
        let canvas = self.surface.canvas();
        canvas.reset_matrix();
        canvas.scale((scale_factor as f32, scale_factor as f32));
        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 1.0));

        view.draw(canvas, canvas_size);

        self.gr_context.flush(None);
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoopBuilder::new().build();

    #[cfg(target_os = "macos")]
    unsafe {
        // work-around for https://github.com/rust-windowing/winit/issues/2051
        use cocoa::appkit::NSApplication as _;
        cocoa::appkit::NSApp().setActivationPolicy_(
            cocoa::appkit::NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular,
        );
    }

    let window_builder = Some(WindowBuilder::new().with_title("ttt"));

    // The template will match only the configurations supporting rendering to
    // windows.
    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            // Find the config with the maximum number of samples, so our triangle will
            // be smooth.
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    // The context creation part. It can be created before surface and that's how
    // it's expected in multithreaded + multiwindow operation mode, since you
    // can send NotCurrentContext, but not Surface.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
    let mut not_current_gl_context = Some(unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_config
                    .display()
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

    let mut view = View::new(Presenter::new()?);

    let mut state = None;
    let mut renderer = None;
    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let gl_window = GlWindow::new(window, &gl_config);

                // Make it current.
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_window.surface)
                    .unwrap();

                // The context needs to be current for the Renderer to set up shaders and
                // buffers. It also performs function loading, which needs a current context on
                // WGL.
                renderer.get_or_insert_with(|| {
                    Renderer::new(
                        &gl_config.display(),
                        gl_window.window.inner_size(),
                        gl_config.num_samples() as usize,
                    )
                });

                // Try setting vsync.
                if let Err(res) = gl_window
                    .surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {res:?}");
                }

                assert!(state.replace((gl_context, gl_window)).is_none());
            }
            Event::Suspended => {
                // This event is only raised on Android, where the backing NativeWindow for a GL
                // Surface can appear and disappear at any moment.
                println!("Android window removed");

                // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
                // the window back to the system.
                let (gl_context, _) = state.take().unwrap();
                assert!(not_current_gl_context
                    .replace(gl_context.make_not_current().unwrap())
                    .is_none());
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        if let Some((gl_context, gl_window)) = &state {
                            gl_window.surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            let renderer = renderer.as_mut().unwrap();
                            renderer.resize(size);
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                e => view.process_event(e, control_flow),
            },
            Event::RedrawEventsCleared => {
                if let Some((gl_context, gl_window)) = &state {
                    let renderer = renderer.as_mut().unwrap();
                    let scale_factor = gl_window.window.scale_factor();
                    renderer.render(
                        scale_factor,
                        &view,
                        gl_window.window.inner_size().to_logical(scale_factor),
                    );
                    gl_window.window.request_redraw();
                    gl_window.surface.swap_buffers(gl_context).unwrap();
                }
            }
            _ => (),
        }
    })
}
