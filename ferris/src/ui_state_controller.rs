use crate::desktop::window::WindowEventHandle;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tracing::info;
use tracing_subscriber::fmt::init;

/// Manages UI state transitions for the application
#[derive(Clone)]
pub struct UIStateController {
    window_event_handle: WindowEventHandle,
    current_url: Arc<Mutex<Option<String>>>,
}

impl UIStateController {
    pub fn new(window_event_handle: WindowEventHandle, initial_url: &'static str) -> Self {
        Self { 
            window_event_handle,
            current_url: Arc::new(Mutex::new(Some(initial_url.to_string()))),
        }
    }

    /// Load a URL only if it's not already loaded
    fn load_url_if_different(&self, url: String) {
        let mut current = self.current_url.lock().unwrap();
        if current.as_ref() != Some(&url) {
            self.window_event_handle.load_url(url.clone());
            *current = Some(url);
        }
    }

    /// Show the waiting for WiFi screen
    pub fn show_waiting_for_wifi(&self) {
        info!("Switching to waiting-for-wifi view");
        self.load_url_if_different("http://localhost:8000/goldie?view=waiting-for-wifi".to_string());
    }

    /// Show the loading screen
    pub fn show_loading(&self) {
        info!("Switching to loading view");
        self.load_url_if_different("http://localhost:8000/goldie?view=loading".to_string());
    }

    /// Show the home screen
    pub fn show_home(&self) {
        info!("Switching to home view");
        self.load_url_if_different("http://localhost:8000/goldie?view=home".to_string());
    }

    /// Refresh the window (hide/show workaround)
    fn refresh_window(&self) {
        self.window_event_handle.hide_window();
        thread::sleep(Duration::from_millis(2000));
        self.window_event_handle.show_window();
    }
}