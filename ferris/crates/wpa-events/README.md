# wpa-events

A simple Rust crate for listening to wpa_supplicant events via wpa_cli.

## Features

- **Simple trait-based handlers** - Implement one trait method
- **Type-safe events** - Strongly typed WiFi state changes  
- **Action script mode** - Works as a wpa_cli action script
- **Minimal dependencies** - Only tokio, anyhow, async-trait, and tracing

## Usage

```rust
use wpa_events::{WpaEventMonitor, WpaEvent, WpaState, WpaEventHandler};
use anyhow::Result;
use async_trait::async_trait;

struct MyHandler;

#[async_trait]
impl WpaEventHandler for MyHandler {
    async fn handle_event(&self, event: WpaEvent) -> Result<()> {
        match event.state {
            WpaState::Connected => {
                println!("Connected to WiFi on {}", event.interface);
            }
            WpaState::ApStaConnected => {
                if let Some(mac) = event.mac_address {
                    println!("Station {} connected to AP", mac);
                }
            }
            WpaState::Disconnected => {
                println!("Disconnected from WiFi");
            }
            _ => {
                println!("WiFi event: {}", event);
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let handler = MyHandler;
    let monitor = WpaEventMonitor::new("wlan0", handler)?;
    monitor.start().await?;
    Ok(())
}
```

## How it works

This crate works by:

1. Starting `wpa_cli -i <interface> -a <your_binary>` 
2. wpa_cli calls your binary when events occur
3. Your binary parses the arguments and calls your event handler
4. The monitor continues until wpa_cli exits

The same binary acts as both the monitor starter and the event handler.

## Event Types

- `ApEnabled` - Access Point mode enabled
- `ApDisabled` - Access Point mode disabled  
- `Connected` - Connected to WiFi network (station mode)
- `Disconnected` - Disconnected from WiFi network
- `ApStaConnected` - A station connected to our AP (includes MAC address)
- `ApStaDisconnected` - A station disconnected from our AP (includes MAC address)