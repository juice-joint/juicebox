use crate::desktop::window::WindowEventHandle;
use std::{thread, time::Duration};
use tracing::info;

/// Manages UI state transitions for the application
#[derive(Clone)]
pub struct UIStateController {
    window_event_handle: WindowEventHandle,
}

impl UIStateController {
    pub fn new(window_event_handle: WindowEventHandle) -> Self {
        Self { window_event_handle }
    }

    /// Show the waiting for WiFi screen
    pub fn show_waiting_for_wifi(&self) {
        info!("Switching to waiting-for-wifi view");
        self.window_event_handle
            .load_url("http://localhost:8000/goldie?view=waiting-for-wifi".to_string());
    }

    /// Show the loading screen
    pub fn show_loading(&self) {
        info!("Switching to loading view");
        self.window_event_handle
            .load_url("http://localhost:8000/goldie?view=loading".to_string());
    }

    /// Show the home screen
    pub fn show_home(&self) {
        info!("Switching to home view");
        self.window_event_handle
            .load_url("http://localhost:8000/goldie?view=home".to_string());
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
        
        // Refresh window to handle https://github.com/tauri-apps/tauri/issues/9289
        self.refresh_window();
    }

    /// Refresh the window (hide/show workaround)
    fn refresh_window(&self) {
        self.window_event_handle.hide_window();
        thread::sleep(Duration::from_millis(100));
        self.window_event_handle.show_window();
    }

    /// Get the initial URL based on connectivity
    pub fn get_initial_url(is_connected: bool) -> &'static str {
        if is_connected {
            "http://localhost:8000/goldie?view=loading"
        } else {
            "http://localhost:8000/goldie?view=waiting-for-wifi"
        }
    }
}