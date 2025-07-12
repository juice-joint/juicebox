use anyhow::Result;
use std::fmt;

/// WiFi state changes reported by wpa_supplicant
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WpaState {
    /// Access Point has been enabled
    ApEnabled,
    /// Access Point has been disabled
    ApDisabled,
    /// Connected to a WiFi network (station mode)
    Connected,
    /// A station connected to our Access Point
    ApStaConnected,
    /// A station disconnected from our Access Point
    ApStaDisconnected,
    /// Disconnected from WiFi network
    Disconnected,
}

impl fmt::Display for WpaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WpaState::ApEnabled => "AP-ENABLED",
            WpaState::ApDisabled => "AP-DISABLED", 
            WpaState::Connected => "CONNECTED",
            WpaState::ApStaConnected => "AP-STA-CONNECTED",
            WpaState::ApStaDisconnected => "AP-STA-DISCONNECTED",
            WpaState::Disconnected => "DISCONNECTED",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for WpaState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "AP-ENABLED" => Ok(WpaState::ApEnabled),
            "AP-DISABLED" => Ok(WpaState::ApDisabled),
            "CONNECTED" => Ok(WpaState::Connected),
            "AP-STA-CONNECTED" => Ok(WpaState::ApStaConnected),
            "AP-STA-DISCONNECTED" => Ok(WpaState::ApStaDisconnected),
            "DISCONNECTED" => Ok(WpaState::Disconnected),
            _ => Err(anyhow::anyhow!("Unrecognized WiFi state: {}", s)),
        }
    }
}

/// A wpa_supplicant event with associated metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WpaEvent {
    /// The network interface this event occurred on
    pub interface: String,
    /// The type of state change
    pub state: WpaState,
    /// MAC address of the station (for AP events)
    pub mac_address: Option<String>,
}

impl WpaEvent {
    /// Create a new WpaEvent
    pub fn new(interface: String, state: WpaState, mac_address: Option<String>) -> Self {
        Self {
            interface,
            state,
            mac_address,
        }
    }

    /// Parse a WpaEvent from command line arguments (wpa_cli action script format)
    /// 
    /// Expected format: `[binary_name] <interface> <state> [mac_address]`
    pub fn from_args(args: Vec<String>) -> Result<Self> {
        if args.len() < 3 {
            return Err(anyhow::anyhow!(
                "Insufficient arguments. Expected: <interface> <state> [mac_address], got: {:?}",
                args
            ));
        }

        let interface = args[1].clone();
        let state: WpaState = args[2].parse()?;
        let mac_address = args.get(3).map(|s| s.clone());

        Ok(Self::new(interface, state, mac_address))
    }
}

impl fmt::Display for WpaEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.mac_address {
            Some(mac) => write!(f, "{} {} {}", self.interface, self.state, mac),
            None => write!(f, "{} {}", self.interface, self.state),
        }
    }
}