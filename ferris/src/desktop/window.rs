use serde::ser::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Fullscreen, WindowBuilder},
};
use wry::WebViewBuilder;

pub enum AppEvent {
    LoadUrl(String),
    Exit,
}

#[derive(Clone)]
pub struct WindowEventHandle {
    event_proxy: EventLoopProxy<AppEvent>,
}

impl WindowEventHandle {
    pub fn new(event_proxy: EventLoopProxy<AppEvent>) -> Self {
        Self { event_proxy }
    }

    pub fn load_url(&self, url: String) {
        let _ = self.event_proxy.send_event(AppEvent::LoadUrl(url));
    }
}

pub fn create_desktop_webview(
    url: &str,
    event_loop: EventLoop<AppEvent>,
) -> Result<(WindowEventHandle, wry::WebView, EventLoop<AppEvent>), wry::Error> {
    // Build the window
    let window = WindowBuilder::new()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("Failed to build window");

    // Create the webview builder
    let builder = WebViewBuilder::new()
        .with_url(url)
        .with_initialization_script("console.log('Desktop app initialized');");

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
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(app_event) => match app_event {
                AppEvent::LoadUrl(url) => {
                    println!("laoding url");
                    let test = webview.load_url(&url);
                }
                AppEvent::Exit => {}
            },
            _ => (),
        }
    });
}