use std::net::SocketAddr;
use std::sync::Arc;

use actors::song_coordinator::SongActorHandle;
use axum::routing::post;
use axum::serve;
use axum::{routing::get, Router};
use routes::healthcheck::healthcheck;
use state::AppState;
use tokio::net::TcpListener;
use tokio::sync::{self, oneshot};

use actors::video_downloader::VideoDlActorHandle;
use actors::video_searcher::VideoSearcherActorHandle;
use routes::admin::{get_key, key_down, key_up, remove_song, reposition_song, restart_song, toggle_playback};
use routes::karaoke::{current_song, play_next_song, queue_song, search, song_list};
use routes::sse::sse;
use routes::streaming::serve_dash_file;
use routes::sys::server_ip;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use utils::yt_downloader::YtDownloader;
use utils::yt_searcher::YtSearcher;

use rust_embed::RustEmbed;
use axum_embed::ServeEmbed;

pub mod actors;
pub mod routes;
pub mod utils;
mod state;
pub mod globals;

// Include the built React apps using rust-embed
#[derive(RustEmbed, Clone)]
#[folder = "assets/goldie"]
struct GoldieAssets;

#[derive(RustEmbed, Clone)]
#[folder = "assets/phippy"]
struct PhippyAssets;

fn create_api_router() -> Router {
    let yt_downloader = Arc::new(YtDownloader {});
    let yt_searcher = Arc::new(YtSearcher {});

    let (sse_broadcaster, _) = sync::broadcast::channel(10);
    let sse_broadcaster = Arc::new(sse_broadcaster);

    let song_actor_handle = Arc::new(SongActorHandle::new(sse_broadcaster.clone()));
    let videodl_actor_handle = Arc::new(VideoDlActorHandle::new(
        String::from("./assets"),
        yt_downloader,
    ));
    let videosearcher_actor_handle = Arc::new(VideoSearcherActorHandle::new(yt_searcher));

    let app_state = AppState::new(
        song_actor_handle,
        videodl_actor_handle,
        videosearcher_actor_handle,
        sse_broadcaster.clone(),
    );

    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/server_ip", get(server_ip))
        .route("/queue_song", post(queue_song))
        .route("/play_next", post(play_next_song))
        .route("/song_list", get(song_list))
        .route("/current_song", get(current_song))
        .route("/dash/{song_name}/{file}", get(serve_dash_file))
        .route("/sse", get(sse))
        .route("/toggle_playback", post(toggle_playback))
        .route("/key_up", post(key_up))
        .route("/key_down", post(key_down))
        .route("/get_key", get(get_key))
        .route("/reposition_song", post(reposition_song))
        .route("/remove_song", post(remove_song))
        .route("/restart", post(restart_song))
        .route("/search", get(search))
        .with_state(app_state)
}

pub async fn run_server(addr: SocketAddr, ready_tx: oneshot::Sender<()>) {
    let api_router = create_api_router();

    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new() 
        .merge(api_router)
        // Use ServeEmbed for Goldie assets
        .nest_service(
            "/goldie", 
            ServeEmbed::<GoldieAssets>::new()
        )
        // Use ServeEmbed for Phippy assets
        .nest_service(
            "/phippy", 
            ServeEmbed::<PhippyAssets>::new()
        )
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http());

    // Run the server
    info!("Starting server on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    // Signal that the server is ready
    let _ = ready_tx.send(());

    match serve(listener, app).await {
        Ok(_) => info!("Server shutdown gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}