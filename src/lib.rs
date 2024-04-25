use std::{
    thread::{sleep, spawn},
    time::Duration
};
use state::State;
use wgpu::SurfaceError;
use winit::{
    event::{Event, WindowEvent}, event_loop::EventLoopBuilder, window::WindowBuilder
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use custom_event::CustomEvent;

mod custom_event;
mod state;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run()
{
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event()
        .build()
        .unwrap();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::dpi::PhysicalSize;
            use winit::platform::web::WindowBuilderExtWebSys;
            use winit::platform::web::WindowExtWebSys;

            let window = WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(450, 400))
                .with_canvas(None)
                .build(&event_loop)
                .unwrap();

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let body = doc.body()?;
                    let canvas = web_sys::Element::from(window.canvas().unwrap());
                    body.append_child(&canvas).ok()?;
                    Some(())
                }).expect("Couldn't append canvas to document body.");
        } else {
            let window = WindowBuilder::new()
                .build(&event_loop)
                .unwrap();
    
            let event_loop_proxy = event_loop.create_proxy();
    
            spawn(move || loop {
                sleep(Duration::from_millis(18));
                event_loop_proxy.send_event(CustomEvent::Timer).ok();
            });
        }
    }

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
