use cocoa::{appkit::NSView, base::id as cocoa_id, base::YES};
use core_graphics_types::geometry::CGSize;
use metal::*;
use std::mem;
use winit::platform::macos::WindowExtMacOS;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;

fn main() {
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Metal Rendering Engine".to_string())
        .build(&event_loop)
        .unwrap();

    #[rustfmt::skip]
    let positions: [f32; 12] =
    [
         0.0,  0.5, 0.0, 1.0,
        -0.5, -0.5, 0.0, 1.0,
         0.5, -0.5, 0.0, 1.0,
    ];

    #[rustfmt::skip]
    let colors: [f32; 12] =
    [
        1.0, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
    ];

    let device = Device::system_default().expect("no device found");

    let layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    layer.set_presents_with_transaction(false);

    unsafe {
        let view = window.ns_view() as cocoa_id;
        view.setWantsLayer(YES);
        view.setLayer(mem::transmute(layer.as_ref()));
    }

    let draw_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(draw_size.width as f64, draw_size.height as f64));

    let library_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/shaders.metallib");

    let library = device.new_library_with_file(library_path).unwrap();
    let vertex_function = library.get_function("vertex_main", None).unwrap();
    let fragment_function = library.get_function("fragment_main", None).unwrap();

    let pipeline_state_descriptor = RenderPipelineDescriptor::new();
    pipeline_state_descriptor.set_vertex_function(Some(&vertex_function));
    pipeline_state_descriptor.set_fragment_function(Some(&fragment_function));

    let color_attachment = pipeline_state_descriptor
        .color_attachments()
        .object_at(0)
        .unwrap();
    color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    pipeline_state_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Invalid);

    let pipeline_state = device
        .new_render_pipeline_state(&pipeline_state_descriptor)
        .unwrap();

    let command_queue = device.new_command_queue();

    let position_buffer = device.new_buffer_with_data(
        positions.as_ptr() as *const _,
        mem::size_of::<[f32; 12]>() as u64,
        MTLResourceOptions::CPUCacheModeDefaultCache,
    );

    let color_buffer = device.new_buffer_with_data(
        colors.as_ptr() as *const _,
        mem::size_of::<[f32; 12]>() as u64,
        MTLResourceOptions::CPUCacheModeDefaultCache,
    );

    event_loop.run(move |event, _, control_flow| match event {
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
            _ => {}
        },
        Event::RedrawRequested(_) => {
            let drawable = match layer.next_drawable() {
                Some(drawable) => drawable,
                None => return,
            };

            let render_pass_descriptor = RenderPassDescriptor::new();

            let color_attachment = render_pass_descriptor
                .color_attachments()
                .object_at(0)
                .unwrap();

            color_attachment.set_texture(Some(&drawable.texture()));
            color_attachment.set_load_action(MTLLoadAction::Clear);
            color_attachment.set_clear_color(MTLClearColor::new(0.2, 0.2, 0.25, 1.0));
            color_attachment.set_store_action(MTLStoreAction::Store);

            let command_buffer = command_queue.new_command_buffer();
            let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
            render_encoder.set_vertex_buffer(0, Some(&position_buffer), 0);
            render_encoder.set_vertex_buffer(1, Some(&color_buffer), 0);

            render_encoder.set_render_pipeline_state(&pipeline_state);
            render_encoder.draw_primitives_instanced(MTLPrimitiveType::Triangle, 0, 3, 1);
            render_encoder.end_encoding();

            command_buffer.present_drawable(&drawable);
            command_buffer.commit();
        }
        _ => {}
    });
}
