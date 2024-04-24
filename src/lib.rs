use std::{
    thread::{sleep, spawn},
    time::Duration
};
use state::State;
use wgpu::SurfaceError;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopBuilder,
    window::WindowBuilder
};
use custom_event::CustomEvent;

mod custom_event;
mod state;

pub async fn run()
{
    env_logger::init();

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event()
        .build()
        .unwrap();

    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let event_loop_proxy = event_loop.create_proxy();

    spawn(move || loop {
        sleep(Duration::from_millis(18));
        event_loop_proxy.send_event(CustomEvent::Timer).ok();
    });

    let mut state = State::new(&window).await;

    event_loop.run(move |event, elwt| match event {
        Event::UserEvent(..) => {
            state.window.request_redraw();
        },
        Event::WindowEvent {
            window_id, event
        } if window_id == state.window.id() => match event {
            WindowEvent::CloseRequested => {
                elwt.exit();
            },
            WindowEvent::Resized(physical_size) => state.resize(physical_size),
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {},
                Err(SurfaceError::Lost) => state.resize(state.size),
                Err(SurfaceError::OutOfMemory) => elwt.exit(),
                Err(e) => eprintln!("{e:?}")
            },
            _ => {}
        }
        _ => {}
    }).expect("Error!");
}
