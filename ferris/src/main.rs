use desktop::window::{AppEvent, WindowEventHandle};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tao::event_loop::EventLoopBuilder;
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::{info, error, warn};
use ui_state_controller::UIStateController;
use binary_initializer::BinaryInitializer;

mod binary_initializer;
mod desktop;
mod server;
mod ui_state_controller;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let config_dir = PathBuf::from("./config");

    // Check connection
    let is_connected = check_internet_connectivity().await;
    let initial_url = get_initial_url(is_connected);

    // Start the server
    let server_handle = start_server(addr).await;

    // Create window event loop and handle
    let (event_loop, window_event_handle) = create_window_components();
    let ui_controller = UIStateController::new(window_event_handle.clone(), initial_url);

    // Refresh window to handle https://github.com/tauri-apps/tauri/issues/9289
    // TODO, make 2000ms not time based
    let window_event_handle_clone = window_event_handle.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        window_event_handle_clone.hide_window();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        window_event_handle_clone.show_window();
    });

    // Always start connectivity monitoring - it will handle initialization when online
    start_connectivity_monitoring(config_dir.clone(), ui_controller.clone()).await;

    // Run the desktop window
    match run_desktop_window(event_loop, initial_url).await {
        Ok(_) => info!("Desktop app closed successfully"),
        Err(e) => error!("Desktop app error: {}", e),
    }

    // Cleanup
    server_handle.abort();
    info!("Application shutting down");
}

async fn start_server(addr: SocketAddr) -> JoinHandle<()> {
    info!("Starting server on {}", addr);

    let (tx, rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        server::run_server(addr, tx).await;
    });

    rx.await.expect("Failed to receive server ready signal");
    info!("Server is ready");

    server_handle
}


fn create_window_components() -> (tao::event_loop::EventLoop<AppEvent>, WindowEventHandle) {
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let event_loop_proxy = event_loop.create_proxy();
    let window_event_handle = WindowEventHandle::new(event_loop_proxy);

    (event_loop, window_event_handle)
}

/// Get the initial URL based on connectivity
fn get_initial_url(is_connected: bool) -> &'static str {
    if is_connected {
        "http://localhost:8000/goldie?view=loading"
    } else {
        "http://localhost:8000/goldie?view=waiting-for-wifi"
    }
}

async fn check_internet_connectivity() -> bool {
    // Try to connect to a reliable DNS server (Google's 8.8.8.8)
    use std::net::SocketAddr;
    use tokio::net::TcpStream;
    use tokio::time::timeout;
    
    let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
    let connect_timeout = Duration::from_secs(3);
    
    match timeout(connect_timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

async fn start_connectivity_monitoring(config_dir: PathBuf, ui_controller: UIStateController) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        info!("Starting connectivity monitoring");
        let mut was_connected = false;
        
        loop {
            info!("looping");
            let is_connected = check_internet_connectivity().await;

            info!("is_connected: {}", is_connected);
            info!("was_connected: {}", was_connected);
            
            if is_connected && !was_connected {
                // Connection restored or established
                info!("Connected to internet!");
                
                if BinaryInitializer::are_binaries_initialized() {
                    // Binaries already initialized, just go to home
                    ui_controller.show_home();
                } else {
                    // Need to initialize binaries
                    ui_controller.show_loading();
                    BinaryInitializer::initialize(config_dir.clone(), ui_controller.clone()).await;
                }
            } else if !is_connected && was_connected {
                // Connection lost
                info!("wtf");
                warn!("Lost internet connection");
                ui_controller.show_waiting_for_wifi();
            } else if !is_connected && !was_connected {
                ui_controller.show_waiting_for_wifi();
            }
            
            was_connected = is_connected;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

async fn run_desktop_window(
    event_loop: tao::event_loop::EventLoop<AppEvent>,
    initial_url: &'static str
) -> Result<(), Box<dyn std::error::Error>> {    
    desktop::window::create_desktop_webview(initial_url, event_loop)
        .map(|_| ())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
