use cocoa::{appkit::NSView, base::id as cocoa_id};
use core_graphics_types::geometry::CGSize;
use metal::*;
use metal_rendering_engine::Renderer;
use objc::{rc::autoreleasepool, runtime::YES};
use std::mem;
use winit::platform::macos::WindowExtMacOS;
use winit::{
    dpi::LogicalSize,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod shader_bindings;

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 800;

fn main() {
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Metal Rendering Engine".to_string())
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new();

    let layer = MetalLayer::new();
    layer.set_device(&renderer.device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    unsafe {
        let view = window.ns_view() as cocoa_id;
        view.setWantsLayer(YES);
        view.setLayer(mem::transmute(layer.as_ref()));
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        autoreleasepool(|| match event {
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
                // WindowEvent::CursorMoved {
                //     device_id,
                //     position,
                //     modifiers,
                // } => {
                //     renderer.rotate((position.x, position.y));
                // }
                _ => {}
            },
            Event::DeviceEvent {
                device_id,
                ref event,
            } => match event {
                // DeviceEvent::MouseMotion { delta } => {
                //     renderer.rotate(*delta);
                // }
                DeviceEvent::MouseWheel { delta } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        renderer.zoom(*y);
                    }
                    MouseScrollDelta::PixelDelta(_) => {}
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                let drawable = match layer.next_drawable() {
                    Some(drawable) => drawable,
                    None => return,
                };

                renderer.draw(drawable);
            }
            Event::RedrawRequested(_) => {}
            _ => {}
        });
    })
}
