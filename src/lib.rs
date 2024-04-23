use std::{
    thread::{sleep, spawn},
    time::Duration
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopBuilder,
    window::WindowBuilder
};
use custom_event::CustomEvent;

mod custom_event;

pub fn run()
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

    event_loop.run(move |event, elwt| match event {
        Event::UserEvent(..) => {

        },
        Event::WindowEvent {
            window_id, event
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                elwt.exit();
            },
            _ => {}
        }
        _ => {}
    }).expect("Error!");
}
