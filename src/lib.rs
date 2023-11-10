#![allow(unused_imports)]
mod app;
mod render_env;
mod raytracing;
mod screen;
mod world;
mod common;

use std::time::{Instant, Duration};
use anyhow::Context;
use app::App;
use render_env::RenderEnv;
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent, StartCause}, dpi::PhysicalSize};

pub async fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().context("Failed to start event loop")?;
    let window = WindowBuilder::new()
        .with_title("Ray tracer")
        .with_inner_size(PhysicalSize::new(3840/2, 2160/2))
        .build(&event_loop)?;
    let mut render_env = RenderEnv::new(window).await?;
    let mut app = App::new(&render_env)?;

    let mut frame_counter = 0;
    let wait_len = Duration::from_millis(1000);
    // event_loop.set_control_flow(ControlFlow::wait_duration(wait_len));
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, elwt| {
        frame_counter += 1;
        // if frame_counter % 1000 == 0 {
        //     dbg!(frame_counter);
        // }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                    println!("The close button was pressed; stopping");
                    elwt.exit();
                },
            Event::AboutToWait => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                // elwt.set_control_flow(ControlFlow::wait_duration(wait_len));
                render_env.window.request_redraw();
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                // render_env.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in AboutToWait, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.
                    app.update(&render_env);
                    app.render(&render_env).unwrap();
                },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                render_env.resize();
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput {event, .. }, .. } => {
                app.on_input(&render_env, event);
            }
            _ => ()
        }
    })?;

    Ok(())
}

fn next_frame_time(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}
