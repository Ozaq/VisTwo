use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{Display, Surface};
use imgui::{Condition, Context, MenuItem, Ui, Window};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::window::Fullscreen;

pub struct System {
    pub display: Display,
    pub imgui_ctx: Context,
    pub event_loop: EventLoop<()>,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let wb = WindowBuilder::new()
            //.with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
            .with_inner_size(LogicalSize::new(1024.0, 768.0))
            .with_title("Hello world");
        let cb = ContextBuilder::new().with_vsync(true);
        let display = Display::new(wb, cb, &event_loop).expect("Failed to initialize display!");
        let mut imgui_ctx = Context::create();
        imgui_ctx.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui_ctx);
        platform.attach_window(
            imgui_ctx.io_mut(),
            display.gl_window().window(),
            HiDpiMode::Default,
        );

        let renderer =
            Renderer::init(&mut imgui_ctx, &display).expect("Failed to initialize renderer!");

        System {
            display,
            imgui_ctx,
            event_loop,
            platform,
            renderer,
        }
    }

    pub fn enter_main_loop<F>(self, mut draw_ui: F)
    where
        F: FnMut(&mut bool, &mut Ui) + 'static,
    {
        let Self {
            display,
            mut imgui_ctx,
            event_loop,
            mut platform,
            mut renderer,
        } = self;

        let mut last_frame = std::time::Instant::now();
        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui_ctx.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui_ctx.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame!");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let mut ui = imgui_ctx.frame();
                let mut keep_running = true;
                draw_ui(&mut keep_running, &mut ui);
                if !keep_running {
                    *control_flow = ControlFlow::Exit;
                }
                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 0.0, 0.0, 1.0);
                platform.prepare_render(&ui, gl_window.window());
                let draw_data = ui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed!");
                target.finish().expect("Falied to swap buffers!");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                platform.handle_event(imgui_ctx.io_mut(), display.gl_window().window(), &event)
            }
        });
    }
}

fn main() {
    let system = System::new();
    system.enter_main_loop(move |_, ui| {
        let io = ui.io();
        Window::new("Hello!")
            .size(io.display_size, Condition::Always)
            .position([0f32, 0f32], Condition::Always)
            .no_decoration()
            .menu_bar(true)
            .movable(false)
            .build(ui, || {
                ui.menu_bar(|| {
                    let file_clicked = MenuItem::new("File").build(ui);
                    if file_clicked {
                        println!("FILE CLICKED!!");
                    }
                    MenuItem::new("About").build(ui);
                    MenuItem::new("Exit").build(ui);
                })
            });
    });
}
