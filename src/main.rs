use metal_gltf_viewer::Renderer;
use objc::rc::autoreleasepool;
use winit::{
    dpi::LogicalSize,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod shader_bindings;

const INITIAL_WINDOW_WIDTH: u32 = 1080;
const INITIAL_WINDOW_HEIGHT: u32 = 720;

struct State {
    left_mouse_pressed: bool,
}

impl State {
    fn new() -> Self {
        Self {
            left_mouse_pressed: false,
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Metal Rendering Engine".to_string())
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window);
    let mut program_state = State::new();

    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => window.request_redraw(),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(new_inner_size.width, new_inner_size.height);
                    }
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => {
                        program_state.left_mouse_pressed = *state == ElementState::Pressed;
                    }
                    _ => {}
                },
                Event::DeviceEvent { ref event, .. } => match event {
                    DeviceEvent::MouseWheel { delta } => match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            renderer.zoom(*y);
                        }
                        MouseScrollDelta::PixelDelta(_) => {}
                    },
                    // DeviceEvent::Button {
                    //     button: 0, // right mouse button
                    //     state,
                    // } => {
                    //     program_state.left_mouse_pressed = *state == ElementState::Pressed;
                    // }
                    DeviceEvent::MouseMotion { delta } => {
                        if program_state.left_mouse_pressed {
                            renderer.rotate(*delta);
                        }
                    }
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    renderer.draw();
                }
                _ => {}
            }
        });
    })
}
