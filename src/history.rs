use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

/// WiFi connection history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub ssid: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub duration_seconds: Option<u64>, // How long the connection lasted
    pub disconnection_reason: Option<String>, // Why it disconnected
}

/// WiFi connection history manager
pub struct HistoryManager {
    entries: Arc<Mutex<Vec<ConnectionEntry>>>,
    max_entries: usize,
}

impl HistoryManager {
    /// Create a new history manager with max 100 entries
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max_entries: 100,
        }
    }

    /// Record a successful connection
    pub async fn record_connection(&self, ssid: String) {
        let entry = ConnectionEntry {
            ssid,
            timestamp: Utc::now(),
            success: true,
            duration_seconds: None,
            disconnection_reason: None,
        };

        let mut entries = self.entries.lock().await;
        entries.push(entry);

        // Keep only latest 100 entries
        if entries.len() > self.max_entries {
            entries.remove(0);
        }

        tracing::info!("Connection history recorded");
    }

    /// Record a failed connection
    pub async fn record_failure(&self, ssid: String, reason: String) {
        let entry = ConnectionEntry {
            ssid,
            timestamp: Utc::now(),
            success: false,
            duration_seconds: None,
            disconnection_reason: Some(reason),
        };

        let mut entries = self.entries.lock().await;
        entries.push(entry);

        if entries.len() > self.max_entries {
            entries.remove(0);
        }

        tracing::warn!("Connection failure recorded");
    }

    /// Record disconnection with duration
    pub async fn record_disconnection(&self, ssid: String, duration: u64) {
        let mut entries = self.entries.lock().await;

        // Find the matching connection and update it
        if let Some(entry) = entries.iter_mut().rfind(|e| e.ssid == ssid && e.success) {
            entry.duration_seconds = Some(duration);
            tracing::info!("Disconnection recorded: {} after {}s", ssid, duration);
        }
    }

    /// Get all history entries (newest first)
    pub async fn get_all(&self) -> Vec<ConnectionEntry> {
        let entries = self.entries.lock().await;
        let mut result = entries.clone();
        result.reverse();
        result
    }

    /// Get history for a specific SSID
    pub async fn get_by_ssid(&self, ssid: &str) -> Vec<ConnectionEntry> {
        let entries = self.entries.lock().await;
        let mut result: Vec<_> = entries.iter()
            .filter(|e| e.ssid == ssid)
            .cloned()
            .collect();
        result.reverse();
        result
    }

    /// Get favorite networks (most frequently successful connections)
    pub async fn get_favorites(&self, limit: usize) -> Vec<(String, usize)> {
        let entries = self.entries.lock().await;
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for entry in entries.iter().filter(|e| e.success) {
            *counts.entry(entry.ssid.clone()).or_insert(0) += 1;
        }

        let mut favorites: Vec<_> = counts.into_iter().collect();
        favorites.sort_by(|a, b| b.1.cmp(&a.1));
        
        favorites.into_iter().take(limit).collect()
    }

    /// Clear all history
    pub async fn clear(&self) {
        let mut entries = self.entries.lock().await;
        entries.clear();
        tracing::info!("Connection history cleared");
    }

    /// Get success rate for a specific SSID
    pub async fn get_success_rate(&self, ssid: &str) -> Option<f64> {
        let entries = self.entries.lock().await;
        let filtered: Vec<_> = entries.iter().filter(|e| e.ssid == ssid).collect();
        
        if filtered.is_empty() {
            return None;
        }

        let successes = filtered.iter().filter(|e| e.success).count();
        Some(successes as f64 / filtered.len() as f64 * 100.0)
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_connection() {
        let history = HistoryManager::new();
        history.record_connection("MyWiFi".to_string()).await;
        
        let all = history.get_all().await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].ssid, "MyWiFi");
        assert!(all[0].success);
    }

    #[tokio::test]
    async fn test_favorites() {
        let history = HistoryManager::new();
        history.record_connection("WiFi1".to_string()).await;
        history.record_connection("WiFi2".to_string()).await;
        history.record_connection("WiFi1".to_string()).await;
        
        let favorites = history.get_favorites(2).await;
        assert_eq!(favorites[0].0, "WiFi1");
        assert_eq!(favorites[0].1, 2);
    }
}
