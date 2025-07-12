use crate::desktop::window::WindowEventHandle;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tracing::info;

/// Manages UI state transitions for the application
#[derive(Clone)]
pub struct UIStateController {
    window_event_handle: WindowEventHandle,
    current_url: Arc<Mutex<Option<String>>>,
}

impl UIStateController {
    pub fn new(window_event_handle: WindowEventHandle) -> Self {
        Self { 
            window_event_handle,
            current_url: Arc::new(Mutex::new(None)),
        }
    }

    /// Load a URL only if it's not already loaded
    fn load_url_if_different(&self, url: String) {
        let mut current = self.current_url.lock().unwrap();
        if current.as_ref() != Some(&url) {
            self.window_event_handle.load_url(url.clone());
            *current = Some(url);

            // Refresh window to handle https://github.com/tauri-apps/tauri/issues/9289
            self.refresh_window();
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

    /// Handle connectivity being restored (from waiting-for-wifi to loading)
    pub fn handle_connectivity_restored(&self) {
        info!("Internet connection restored, switching to loading view");
        self.show_loading();
    }

    /// Handle initialization completion (from loading to home)
    pub fn handle_initialization_complete(&self) {
        info!("Binary initialization complete, switching to home view");
        self.show_home();
    }

    /// Refresh the window (hide/show workaround)
    fn refresh_window(&self) {
        self.window_event_handle.hide_window();
        thread::sleep(Duration::from_millis(2000));
        self.window_event_handle.show_window();
    }

    /// Get the initial URL based on connectivity
    pub fn get_initial_url(is_connected: bool) -> &'static str {
        "http://localhost:8000/goldie?view=loading"
    }

    /// Load the initial URL and refresh window
    pub fn load_initial_url(&self, is_connected: bool) {
        let url = Self::get_initial_url(is_connected);
        info!("Loading initial URL: {}", url);
        self.window_event_handle.load_url(url.to_string());
        let mut current = self.current_url.lock().unwrap();
        *current = Some(url.to_string());
        drop(current);
        self.refresh_window();
    }
}