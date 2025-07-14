use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Fullscreen, WindowBuilder},
};
use tracing::{info, error};
use wry::WebViewBuilder;

pub enum AppEvent {
    LoadUrl(String),
    Hide,
    Show,
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

    pub fn hide_window(&self) {
        let _ = self.event_proxy.send_event(AppEvent::Hide);
    }

    pub fn show_window(&self) {
        let _ = self.event_proxy.send_event(AppEvent::Show);
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

    window.set_cursor_visible(false);

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

    // Run the event loop, for some reason this can't be in a different function...
    let first_load_url = Arc::new(AtomicBool::new(true));
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(app_event) => match app_event {
                AppEvent::LoadUrl(url) => {
                    info!("Webview LoadUrl requested");
                    
                    // Refresh window on first load to handle https://github.com/tauri-apps/tauri/issues/9289
                    if first_load_url.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                        info!("First LoadUrl detected, applying visibility workaround");
                        window.set_visible(false);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        window.set_visible(true);
                    }
                    
                    match webview.load_url(&url) {
                        Ok(_) => info!("Successfully loaded url {} in webview", url),
                        Err(_) => error!("Error loading url {} in webview", url),
                    }
                }
                AppEvent::Hide => {
                    info!("Window hide requested");
                    window.set_visible(false);
                }
                AppEvent::Show => {
                    info!("Window show requested");
                    window.set_visible(true);
                }
            },
            _ => (),
        }
    });
}
