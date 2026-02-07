use crate::system::SystemCommand;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: i32, // dBm, negative value (-100 to 0)
    pub security: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiStatus {
    pub connected: bool,
    pub ssid: Option<String>,
    pub ip: Option<String>,
    pub signal: Option<i32>,
}

pub struct WifiManager;

impl WifiManager {
    const WLAN_INTERFACE: &'static str = "wlan0";

    /// Scan for available networks
    pub fn scan() -> Result<Vec<WifiNetwork>> {
        // Using nmcli for scanning
        let output = SystemCommand::run_sudo("nmcli", &["dev", "wifi", "list"])?;

        let mut networks = Vec::new();
        let ssid_re = Regex::new(r"(\S+)\s+([A-F0-9:]{17})\s+(\d+)").unwrap();

        for line in output.lines().skip(1) {
            // Skip header
            if let Some(caps) = ssid_re.captures(line) {
                networks.push(WifiNetwork {
                    ssid: caps[1].to_string(),
                    signal: caps[3].parse().unwrap_or(-100),
                    security: "WPA2".to_string(), // TODO: parse actual security type
                });
            }
        }

        Ok(networks)
    }

    /// Connect to a WiFi network
    pub fn connect(ssid: &str, password: &str) -> Result<()> {
        // Delete existing connection if present
        let _ = SystemCommand::run_sudo(
            "nmcli",
            &["connection", "delete", ssid],
        );

        // Create new connection
        SystemCommand::run_sudo(
            "nmcli",
            &[
                "device",
                "wifi",
                "connect",
                ssid,
                "password",
                password,
                "ifname",
                Self::WLAN_INTERFACE,
            ],
        )?;

        Ok(())
    }

    /// Get current WiFi connection status
    pub fn status() -> Result<WifiStatus> {
        let output = SystemCommand::run_sudo("nmcli", &[
            "connection",
            "show",
            "--active",
        ])?;

        let ssid_re = Regex::new(r"connection\.id:\s+(.+)").unwrap();
        let ssid = output
            .lines()
            .find_map(|line| ssid_re.captures(line).map(|c| c[1].to_string()));

        // Get IP address
        let ip_output = SystemCommand::run(
            "ip",
            &["-4", "addr", "show", Self::WLAN_INTERFACE],
        ).unwrap_or_default();
        let ip_re = Regex::new(r"inet\s+(\S+)").unwrap();
        let ip = ip_output
            .lines()
            .find_map(|line| ip_re.captures(line).map(|c| c[1].to_string()));

        // Get signal strength
        let signal_output = SystemCommand::run_sudo("nmcli", &[
            "device",
            "wifi",
            "list",
            "--rescan",
            "no",
        ])?;
        let signal_re = Regex::new(r"(\d+)$").unwrap();
        let signal = signal_output
            .lines()
            .find_map(|line| signal_re.captures(line).map(|c| c[1].parse().ok()))
            .flatten();

        Ok(WifiStatus {
            connected: ssid.is_some(),
            ssid,
            ip,
            signal,
        })
    }

    /// Disconnect from current WiFi
    pub fn disconnect() -> Result<()> {
        SystemCommand::run_sudo("nmcli", &["device", "disconnect", Self::WLAN_INTERFACE])?;
        Ok(())
    }
}

/// Monitor WiFi connection and auto-reconnect on disconnect
/// 
/// This function runs as a background task that:
/// - Checks WiFi status every 30 seconds
/// - Automatically reconnects with exponential backoff if disconnected
/// - Logs connection status changes
/// 
/// # Arguments
/// * `credentials` - Thread-safe storage of (SSID, Password) to use for reconnection
/// 
/// # Errors
/// Returns early if no saved credentials are available
pub async fn start_monitor(credentials: Arc<Mutex<Option<(String, String)>>>) -> Result<()> {
    tracing::info!("WiFi monitor started");
    
    // Exponential backoff delays (in seconds)
    let backoff_delays = [5, 10, 20, 30];
    let mut backoff_index = 0;
    let mut was_connected = false;

    loop {
        sleep(Duration::from_secs(30)).await;

        // Check current connection status
        match WifiManager::status() {
            Ok(status) => {
                if status.connected {
                    if !was_connected {
                        tracing::info!(
                            ssid = ?status.ssid,
                            ip = ?status.ip,
                            "WiFi connected"
                        );
                        backoff_index = 0; // Reset backoff on successful connection
                        was_connected = true;
                    }
                } else {
                    if was_connected {
                        tracing::info!("WiFi disconnected, attempting to reconnect...");
                        was_connected = false;
                    }

                    // Try to reconnect with credentials
                    let creds = credentials.lock().await;
                    if let Some((ssid, password)) = creds.as_ref() {
                        let ssid = ssid.clone();
                        let password = password.clone();
                        drop(creds); // Release lock before connecting

                        let delay = backoff_delays
                            .get(backoff_index)
                            .copied()
                            .unwrap_or(30);

                        tracing::info!(
                            ssid = %ssid,
                            retry_delay_seconds = delay,
                            "Reconnecting to WiFi..."
                        );

                        sleep(Duration::from_secs(delay)).await;

                        match WifiManager::connect(&ssid, &password) {
                            Ok(_) => {
                                tracing::info!(ssid = %ssid, "WiFi reconnection successful");
                                backoff_index = 0;
                                was_connected = true;
                            }
                            Err(e) => {
                                tracing::warn!(
                                    ssid = %ssid,
                                    error = %e,
                                    "WiFi reconnection failed"
                                );
                                if backoff_index < backoff_delays.len() - 1 {
                                    backoff_index += 1;
                                }
                            }
                        }
                    } else {
                        tracing::debug!("No saved WiFi credentials available for reconnection");
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to check WiFi status");
            }
        }
    }
}
