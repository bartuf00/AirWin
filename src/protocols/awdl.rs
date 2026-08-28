//! AWDL Protocol Integration
//!
//! This module provides the AWDL (Apple Wireless Direct Link) manager API used
//! across AirWin. The original implementation relied on the external `OWDL`
//! crate which is no longer available, so this module now ships as a
//! lightweight stub: it keeps the public API surface intact and gracefully
//! reports AWDL as unavailable instead of failing to compile.
//!
//! AWDL requires low-level Wi-Fi frame injection that is not practical on
//! stock Windows drivers anyway; the rest of AirWin (AirDrop over HTTPS,
//! AirPlay, mDNS discovery, BLE) is fully functional without it.

use crate::utils::AirWinResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// AWDL Manager for AirWin integration
#[derive(Debug)]
pub struct AwdlManager {
    /// Manager configuration
    config: AwdlManagerConfig,
    /// Current state
    state: Arc<RwLock<AwdlManagerState>>,
}

/// AWDL Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwdlManagerConfig {
    /// Enable AWDL protocol
    pub enabled: bool,
    /// Network interface to use (None for auto-detection)
    pub interface: Option<String>,
    /// Device name for AWDL
    pub device_name: String,
    /// Service name for discovery
    pub service_name: String,
    /// Auto-start daemon on initialization
    pub auto_start: bool,
    /// Peer discovery interval in seconds
    pub discovery_interval: u64,
    /// Maximum number of peers to maintain
    pub max_peers: usize,
}

/// AWDL Manager state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AwdlManagerState {
    /// Manager is stopped
    Stopped,
    /// Manager is initializing
    Initializing,
    /// Manager is starting
    Starting,
    /// Manager is running
    Running,
    /// Manager is stopping
    Stopping,
    /// Manager encountered an error
    Error,
}

/// AWDL peer information for AirWin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwdlPeerInfo {
    /// Peer MAC address
    pub mac_address: [u8; 6],
    /// Peer device name
    pub device_name: String,
    /// Peer service name
    pub service_name: String,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Signal strength (if available)
    pub signal_strength: Option<i32>,
    /// Peer capabilities
    pub capabilities: Vec<String>,
}

impl Default for AwdlManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: None,
            device_name: "AirWin-Device".to_string(),
            service_name: "_airwin._tcp".to_string(),
            auto_start: true,
            discovery_interval: 30,
            max_peers: 50,
        }
    }
}

impl AwdlManager {
    /// Create new AWDL manager
    pub fn new(config: AwdlManagerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(AwdlManagerState::Stopped)),
        }
    }

    /// Initialize the AWDL manager.
    ///
    /// The OWDL backend is unavailable, so AWDL support is disabled and the
    /// manager stays stopped. This is not an error for the rest of the app.
    pub async fn initialize(&mut self) -> AirWinResult<()> {
        if !self.config.enabled {
            info!("AWDL protocol is disabled in configuration");
            return Ok(());
        }

        self.set_state(AwdlManagerState::Initializing).await;
        warn!(
            "AWDL backend (OWDL) is not available in this build; \
             continuing without AWDL support"
        );
        self.config.enabled = false;
        self.set_state(AwdlManagerState::Stopped).await;
        Ok(())
    }

    /// Start the AWDL manager (no-op when AWDL is unavailable)
    pub async fn start(&mut self) -> AirWinResult<()> {
        if self.config.enabled {
            self.set_state(AwdlManagerState::Running).await;
        }
        Ok(())
    }

    /// Stop the AWDL manager
    pub async fn stop(&mut self) -> AirWinResult<()> {
        self.set_state(AwdlManagerState::Stopping).await;
        self.set_state(AwdlManagerState::Stopped).await;
        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> AwdlManagerState {
        *self.state.read().await
    }

    /// Get discovered peers (always empty without the OWDL backend)
    pub async fn get_peers(&self) -> Vec<AwdlPeerInfo> {
        Vec::new()
    }

    /// Send data to a specific peer (no-op without the OWDL backend)
    pub async fn send_data(&self, _peer_mac: [u8; 6], _data: &[u8]) -> AirWinResult<()> {
        Ok(())
    }

    /// Broadcast data to all peers (no-op without the OWDL backend)
    pub async fn broadcast_data(&self, _data: &[u8]) -> AirWinResult<()> {
        Ok(())
    }

    /// Update configuration
    pub async fn update_config(&mut self, config: AwdlManagerConfig) -> AirWinResult<()> {
        self.config = config;
        Ok(())
    }

    /// Set manager state
    async fn set_state(&self, state: AwdlManagerState) {
        *self.state.write().await = state;
    }
}

/// AWDL protocol utilities
pub struct AwdlUtils;

impl AwdlUtils {
    /// Check if AWDL is supported on this system
    pub fn is_supported() -> bool {
        false
    }

    /// Get available network interfaces for AWDL
    pub fn get_available_interfaces() -> Vec<String> {
        Vec::new()
    }

    /// Validate MAC address format
    pub fn validate_mac_address(mac: &[u8; 6]) -> bool {
        // Check for valid MAC address (not all zeros, not broadcast)
        *mac != [0; 6] && *mac != [0xff; 6]
    }

    /// Format MAC address for display
    pub fn format_mac_address(mac: &[u8; 6]) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awdl_manager_config_default() {
        let config = AwdlManagerConfig::default();
        assert!(config.enabled);
        assert!(config.auto_start);
        assert_eq!(config.device_name, "AirWin-Device");
        assert_eq!(config.service_name, "_airwin._tcp");
    }

    #[test]
    fn test_awdl_utils_mac_validation() {
        assert!(!AwdlUtils::validate_mac_address(&[0; 6]));
        assert!(!AwdlUtils::validate_mac_address(&[0xff; 6]));
        assert!(AwdlUtils::validate_mac_address(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
    }

    #[test]
    fn test_awdl_utils_mac_formatting() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let formatted = AwdlUtils::format_mac_address(&mac);
        assert_eq!(formatted, "00:11:22:33:44:55");
    }

    #[tokio::test]
    async fn test_awdl_manager_creation() {
        let config = AwdlManagerConfig::default();
        let manager = AwdlManager::new(config);
        assert_eq!(manager.get_state().await, AwdlManagerState::Stopped);
    }
}
