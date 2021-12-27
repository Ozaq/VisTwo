mod legacy_parsers;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{Display, Frame, Surface};
use imgui::{Condition, Context, MenuItem, Ui, Window};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use legacy_parsers::Trajectory;
use winit::window::Fullscreen;

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
glium::implement_vertex!(Vertex, position, color);

#[derive(Clone, Copy, Debug)]
struct VertexInstanceAttributes {
    offset: [f32; 2],
}
glium::implement_vertex!(VertexInstanceAttributes, offset);

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

#[derive(Debug)]
pub struct ApplicationState {
    pub trajectory: Option<Trajectory>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationState {
    pub fn new() -> Self {
        Self { trajectory: None }
    }
}

pub struct System {
    pub display: Display,
    pub imgui_ctx: Context,
    pub event_loop: EventLoop<()>,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub timer: Timer,
    pub state: ApplicationState,
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
            .with_resizable(true)
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
        let state = ApplicationState::new();

        System {
            display,
            imgui_ctx,
            event_loop,
            platform,
            renderer,
            timer,
            state,
        }
    }

    pub fn enter_main_loop<Fn1, Fn2>(self, mut draw_ui: Fn1, mut draw_content: Fn2)
    where
        Fn1: FnMut(&mut bool, &mut Ui, &mut ApplicationState) + 'static,
        Fn2: FnMut(&mut Frame, f32, &mut ApplicationState, &Display) + 'static,
    {
        let Self {
            display,
            mut imgui_ctx,
            event_loop,
            mut platform,
            mut renderer,
            mut timer,
            mut state,
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
                draw_ui(&mut keep_running, &mut ui, &mut state);
                if !keep_running {
                    *control_flow = ControlFlow::Exit;
                }
                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 0.0, 0.0, 1.0);
                platform.prepare_render(&ui, gl_window.window());
                timer.advance();
                draw_content(&mut target, timer.delta_time, &mut state, &display);
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

fn make_quad() -> Vec<Vertex> {
    let extend = 1.0;
    let top_left = [-extend, extend, 0.0];
    let top_right = [extend, extend, 0.0];
    let bottom_left = [-extend, -extend, 0.0];
    let bottom_right = [extend, -extend, 0.0];
    let red = [1.0, 0.0, 0.0];
    let green = [0.0, 1.0, 0.0];
    let blue = [0.0, 0.0, 1.0];
    let black = [0.0, 0.0, 0.0];
    vec![
        Vertex {
            position: top_left,
            color: red,
        },
        Vertex {
            position: top_right,
            color: green,
        },
        Vertex {
            position: bottom_right,
            color: blue,
        },
        Vertex {
            position: top_left,
            color: black,
        },
        Vertex {
            position: bottom_right,
            color: black,
        },
        Vertex {
            position: bottom_left,
            color: black,
        },
    ]
}

fn main() {
    let system = System::new();
    let vertex_buffer = glium::VertexBuffer::new(&system.display, &make_quad()).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 color;
        in vec2 offset;
        uniform float left;
        uniform float right;
        uniform float top;
        uniform float bottom;

        out vec3 vertex_color;

        mat4 scale(float x, float y, float z) {
            return mat4(
                x, 0, 0, 0,
                0, y, 0, 0,
                0, 0, z, 0,
                0, 0, 0, 1
            );
        }

        mat4 trans(vec3 t) {
            return mat4(
                  1,   0,   0,   0,
                  0,   1,   0,   0,
                  0,   0,   1,   0,
                t.x, t.y, t.z,   1
            );
        }

        mat4 ortho(float left, float right, float top, float bottom, float far, float near) {
            return mat4(
                              2.0/(right-left),                            0,                        0, 0,
                                             0,             2.0/(top-bottom),                        0, 0,
                                             0,                            0,          -2.0/(far-near), 0,
                -((right+left) / (right-left)), -((top+bottom)/(top-bottom)), -((far+near)/(far-near)), 1
            );
        }

        mat4 rotZ(float rad) {
            float sin_rad = sin(rad);
            float cos_rad = cos(rad);
            return mat4(
                cos_rad, -sin_rad, 0.0, 0.0,
                sin_rad,  cos_rad, 0.0, 0.0,
                    0.0,      0.0, 1.0, 0.0,
                    0.0,      0.0, 0.0, 1.0
            );
        }

        void main() {
            mat4 proj = ortho(left, right, top, bottom, -1.0, 1.0);
            gl_Position =  proj * trans(vec3(offset, 0.0)) * scale(0.25, 0.25, 0.25) * vec4(position, 1.0);
            vertex_color = color;
        }
    "#;
    let fragment_shader_src = r#"
        #version 140

        in vec3 vertex_color;
        out vec4 frag_color;

        void main() {
            frag_color = vec4(vertex_color, 1.0);
        }
    "#;
    let program = glium::Program::from_source(
        &system.display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    )
    .unwrap();

    system.enter_main_loop(
        move |keep_running, ui, state| {
            let io = ui.io();
            ui.main_menu_bar(|| {
                let file_clicked = MenuItem::new("File").build(ui);
                if file_clicked {
                    println!("{:?}", state.trajectory);
                }
                let open_clicked = MenuItem::new("Open").build(ui);
                if open_clicked {
                    state.trajectory = Some(legacy_parsers::prase_trajectory_txt(
                        std::path::Path::new("/Users/kkratz/Downloads/results/bottleneck_traj.txt"),
                    ));
                }
                *keep_running = !MenuItem::new("Exit").build(ui);
            });
            //Window::new("Hello!")
            //    .size(io.display_size, Condition::Always)
            //    .position([0f32, 0f32], Condition::Always)
            //    .no_decoration()
            //    .menu_bar(true)
            //    .movable(false)
            //    .build(ui, || {
            //        ui.menu_bar(|| {
            //            let file_clicked = MenuItem::new("File").build(ui);
            //            if file_clicked {
            //                println!("{:?}", state.trajectory);
            //            }
            //            let open_clicked = MenuItem::new("Open").build(ui);
            //            if open_clicked {
            //                state.trajectory =
            //                    Some(legacy_parsers::prase_trajectory_txt(std::path::Path::new(
            //                        "/Users/kkratz/Downloads/results/bottleneck_traj.txt",
            //                    )));
            //            }
            //            *keep_running = !MenuItem::new("Exit").build(ui);
            //        })
            //    });
        },
        move |target, elapsed, state, display| {
            let (offsets, (left, right, bottom, top)) = match state.trajectory.as_ref() {
                Some(t) => {
                    let mut o: Vec<VertexInstanceAttributes> = Vec::new();
                    o.reserve(t.frames.len());
                    for e in &t.frames[10].positions {
                        o.push(VertexInstanceAttributes { offset: *e })
                    }
                    (o, t.area())
                }
                None => (Vec::new(), (-1.0, 1.0, -1.0, 1.0)),
            };
            let offsets2 = vec![
                VertexInstanceAttributes { offset: [0.0, 0.0] },
                VertexInstanceAttributes { offset: [0.5, 0.5] },
                VertexInstanceAttributes { offset: [1.0, 1.0] },
                VertexInstanceAttributes { offset: [1.5, 1.5] },
                VertexInstanceAttributes { offset: [2.0, 2.0] },
                VertexInstanceAttributes { offset: [2.5, 2.5] },
                VertexInstanceAttributes { offset: [3.0, 3.0] },
                VertexInstanceAttributes { offset: [3.5, 3.5] },
            ];
            let offset_buffer = glium::VertexBuffer::new(display, &offsets).unwrap();
            //let offset_buffer = glium::VertexBuffer::new(display, &offsets2).unwrap();
            //let (left, right, bottom, top) = (-1.0f32, 4.5f32, -1.0f32, 4.5f32);
            println!(
                "area: left {}, right {}, top {}, bottom {}",
                left, right, top, bottom
            );
            target
                .draw(
                    (&vertex_buffer, offset_buffer.per_instance().unwrap()),
                    &indices,
                    &program,
                    &glium::uniform! { left: left, right: right, top: top, bottom: bottom },
                    &Default::default(),
                )
                .unwrap();
        },
    );
}
