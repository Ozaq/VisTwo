use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{Display, Frame, Surface};
use imgui::{Condition, Context, MenuItem, Ui, Window};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::window::Fullscreen;

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}
glium::implement_vertex!(Vertex, position);

#[derive(Clone, Copy)]
struct Offset {
    offset: [f32; 2],
}
glium::implement_vertex!(Offset, offset);

#[derive(Clone, Copy)]
pub struct Timer {
    last: std::time::Instant,
    pub delta_time: f32,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last: std::time::Instant::now(),
            delta_time: 0f32,
        }
    }

    pub fn advance(&mut self) {
        let now = std::time::Instant::now();
        let duration = now - self.last;
        self.delta_time = duration.as_secs_f32();
        self.last = now;
    }
}

pub struct System {
    pub display: Display,
    pub imgui_ctx: Context,
    pub event_loop: EventLoop<()>,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub timer: Timer,
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
        let timer = Timer::new();

        System {
            display,
            imgui_ctx,
            event_loop,
            platform,
            renderer,
            timer,
        }
    }

    pub fn enter_main_loop<Fn1, Fn2>(self, mut draw_ui: Fn1, mut draw_content: Fn2)
    where
        Fn1: FnMut(&mut bool, &mut Ui) + 'static,
        Fn2: FnMut(&mut Frame, f32) + 'static,
    {
        let Self {
            display,
            mut imgui_ctx,
            event_loop,
            mut platform,
            mut renderer,
            mut timer,
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
                timer.advance();
                draw_content(&mut target, timer.delta_time);
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
    let shape = vec![
        Vertex {
            position: [-0.1, -0.1],
        },
        Vertex {
            position: [0.0, 0.1],
        },
        Vertex {
            position: [0.1, -0.05],
        },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&system.display, &shape).unwrap();
    let offsets = vec![
        Offset {
            offset: [-0.3, 0.0],
        },
        Offset { offset: [0.3, 0.0] },
        Offset {
            offset: [0.0, -0.3],
        },
        Offset { offset: [0.0, 0.3] },
    ];
    let offset_buffer = glium::VertexBuffer::new(&system.display, &offsets).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 offset;
        uniform float t;

        void main() {
            vec2 pos = position + offset;
            pos.x += t;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;
    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;
    let program = glium::Program::from_source(
        &system.display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    )
    .unwrap();

    let mut offset = 0f32;

    system.enter_main_loop(
        move |_, ui| {
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
        },
        move |target, elapsed| {
            offset += elapsed;
            if offset >= 1f32 {
                offset -= 1.5f32
            }
            target
                .draw(
                    (&vertex_buffer, offset_buffer.per_instance().unwrap()),
                    &indices,
                    &program,
                    &glium::uniform! { t: offset },
                    &Default::default(),
                )
                .unwrap();
        },
    );
}
