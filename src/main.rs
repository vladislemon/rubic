use std::ffi::CStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::thread;
use std::thread::JoinHandle;
use gl33::*;
use gl33::global_loader::*;
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::{ContextBuilder, GlRequest, NotCurrent, WindowedContext};
use glutin::dpi::{LogicalSize, PhysicalSize};
use glutin::event::{DeviceEvent, Event, WindowEvent};
use glutin::window::WindowBuilder;
use crate::settings::Settings;

mod settings;

fn main() {
    let settings = Settings::new("settings.toml");
    let event_loop = EventLoop::new();
    let windowed_context = create_windowed_context(&settings, &event_loop);

    let (stop_signal_sender, stop_signal_receiver) = mpsc::sync_channel(0);
    let (resize_signal_sender, resize_signal_receiver) = mpsc::channel();
    let render_thread_handle = thread::spawn(move || render_entry_point(windowed_context, stop_signal_receiver, resize_signal_receiver));
    let mut render_thread_handle_opt = Some(render_thread_handle);

    event_loop.run(move |event, target, control_flow|
        event_loop_function(event, target, control_flow, &stop_signal_sender, &resize_signal_sender, &mut render_thread_handle_opt));
}

fn event_loop_function<T>(
    event: Event<T>,
    _target: &EventLoopWindowTarget<T>,
    control_flow: &mut ControlFlow,
    stop_signal_sender: &SyncSender<i32>,
    resize_signal_sender: &Sender<PhysicalSize<u32>>,
    render_thread_handle_opt: &mut Option<JoinHandle<()>>
) {
    *control_flow = ControlFlow::Wait;
    match event {
        Event::LoopDestroyed => render_thread_handle_opt.take().unwrap()
            .join()
            .expect("Failed to join render thread"),
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(physical_size) => resize_signal_sender
                .send(physical_size)
                .expect("Failed to send resize signal to render thread"),
            WindowEvent::CloseRequested => {
                stop_signal_sender.send(0).expect("Failed to send stop signal to render thread");
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        },
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::Button { button, state: _state } => println!("Button event: {}", button),
            DeviceEvent::Key(input) => println!("Key event: {}", input.scancode),
            _ => ()
        }
        _ => (),
    }
}

fn render_entry_point(
    windowed_context: WindowedContext<NotCurrent>,
    stop_signal_receiver: Receiver<i32>,
    resize_signal_receiver: Receiver<PhysicalSize<u32>>,
) {
    let current_context = unsafe { windowed_context.make_current() }
        .expect("Failed to make current context");
    unsafe {
        load_global_gl(&|ptr| {
            let c_str = CStr::from_ptr(ptr as *const i8);
            let r_str = c_str.to_str().unwrap();
            current_context.get_proc_address(r_str) as _
        });
    }
    let version = unsafe {
        let data = CStr::from_ptr(glGetString(GL_VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };
    println!("OpenGL version {}", version);
    //current_context.window().set_title(&*format!("cubic; OpenGL version {}", version));

    while stop_signal_receiver.try_recv().is_err() {
        resize_signal_receiver.try_recv().map(|physical_size| {
            println!("Resize to {}x{}", physical_size.width, physical_size.height);
            current_context.resize(physical_size);
            unsafe {
                glViewport(0, 0, physical_size.width as i32, physical_size.height as i32);
            }
        }).ok();
        current_context.swap_buffers().expect("Failed to swap buffers");
    }
}

fn create_windowed_context(settings: &Settings, event_loop: &EventLoop<()>) -> WindowedContext<NotCurrent> {
    let window_builder = WindowBuilder::new()
        .with_title(settings.get_title())
        .with_inner_size(LogicalSize::new(settings.get_width(), settings.get_height()));
    let context_builder = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_vsync(settings.get_vsync());
    let windowed_context = context_builder
        .build_windowed(window_builder, event_loop)
        .expect("Failed to build windowed context");
    return windowed_context;
}
