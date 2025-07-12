use anyhow::Result;
use async_trait::async_trait;
use crate::event::WpaEvent;

#[async_trait]
pub trait WpaEventHandler: Send + Sync {
    /// Handle a wpa_supplicant event
    async fn handle_event(&self, event: WpaEvent) -> Result<()>;
}