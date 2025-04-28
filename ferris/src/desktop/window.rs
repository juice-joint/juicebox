use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};
use wry::WebViewBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn run_desktop_app(url: &str) -> wry::Result<()> {
    // Create the event loop
    let event_loop = EventLoop::new();
    
    // Build the window
    let window = WindowBuilder::new()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("Failed to build window");
    
    // Create a shared flag for application exit
    let exit_requested = Arc::new(AtomicBool::new(false));
    let exit_flag = exit_requested.clone();
    
    // Create the webview builder
    let builder = WebViewBuilder::new()
        .with_url(url)
        .with_initialization_script(
            "console.log('Desktop app initialized');"
        );

    // Build the webview
    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let webview = builder.build(&window)?;
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        builder.build_gtk(vbox)?
    };
    
    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
       
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                tracing::info!("Window close requested");
                exit_flag.store(true, Ordering::SeqCst);
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}