use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Represents a single network device
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub alias: Option<String>,           // User-assigned name
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub vendor: Option<String>,          // MAC vendor lookup
    pub is_static: bool,                 // Is DHCP static
}

/// Manages device discovery, tracking, and persistence
pub struct DeviceManager {
    devices: Arc<Mutex<HashMap<String, Device>>>,
    aliases: Arc<Mutex<HashMap<String, String>>>,  // MAC -> Alias
    config_path: PathBuf,
}

/// Persisted device alias data
#[derive(Debug, Serialize, Deserialize)]
struct DeviceConfig {
    #[serde(default)]
    devices: Vec<DeviceAlias>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceAlias {
    mac: String,
    alias: Option<String>,
    #[serde(default)]
    is_static: bool,
}

impl DeviceManager {
    /// Create a new DeviceManager instance
    pub fn new() -> anyhow::Result<Self> {
        let config_path = Self::get_config_dir()?;
        
        let manager = DeviceManager {
            devices: Arc::new(Mutex::new(HashMap::new())),
            aliases: Arc::new(Mutex::new(HashMap::new())),
            config_path,
        };

        // Load existing aliases from disk
        match manager.load_from_file() {
            Ok(_) => info!("Loaded device aliases from config"),
            Err(e) => debug!("No existing device config or error loading: {}", e),
        }

        Ok(manager)
    }

    /// Get or create the ~/.piwifi directory
    fn get_config_dir() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".piwifi");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
            info!("Created config directory: {}", config_dir.display());
        }

        Ok(config_dir)
    }

    /// Get the path to devices.json
    fn get_devices_file_path(&self) -> PathBuf {
        self.config_path.join("devices.json")
    }

    /// Add or update a device
    pub fn add_or_update(&self, device: Device) -> anyhow::Result<()> {
        let mac = device.mac.to_uppercase();
        
        // Preserve existing alias if not updating it
        let alias = device.alias.clone().or_else(|| {
            if let Ok(devices) = self.devices.lock() {
                devices.get(&mac).and_then(|d| d.alias.clone())
            } else {
                None
            }
        });

        let mut updated_device = device;
        updated_device.mac = mac.clone();
        updated_device.alias = alias.clone();

        // Lookup vendor if not already set
        if updated_device.vendor.is_none() {
            updated_device.vendor = Self::lookup_vendor(&updated_device.mac);
        }

        if let Ok(mut devices) = self.devices.lock() {
            devices.insert(mac.clone(), updated_device);
            debug!("Added/updated device: {}", mac);
        }

        if let Some(alias_str) = alias {
            if let Ok(mut aliases) = self.aliases.lock() {
                aliases.insert(mac, alias_str);
            }
        }

        Ok(())
    }

    /// Set or update a device alias
    pub fn set_alias(&self, mac: &str, alias: &str) -> anyhow::Result<()> {
        let mac = mac.to_uppercase();
        
        if let Ok(mut devices) = self.devices.lock() {
            if let Some(device) = devices.get_mut(&mac) {
                device.alias = Some(alias.to_string());
                info!("Set alias for {}: {}", mac, alias);
            } else {
                return Err(anyhow::anyhow!("Device {} not found", mac));
            }
        }

        if let Ok(mut aliases) = self.aliases.lock() {
            aliases.insert(mac, alias.to_string());
        }

        self.save_to_file()?;
        Ok(())
    }

    /// Get all devices
    pub fn get_all(&self) -> anyhow::Result<Vec<Device>> {
        let devices = self.devices.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock devices: {}", e))?;
        
        let mut all_devices: Vec<Device> = devices.values().cloned().collect();
        all_devices.sort_by(|a, b| a.mac.cmp(&b.mac));
        
        Ok(all_devices)
    }

    /// Get a device by MAC address
    pub fn get_by_mac(&self, mac: &str) -> anyhow::Result<Option<Device>> {
        let mac = mac.to_uppercase();
        let devices = self.devices.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock devices: {}", e))?;
        
        Ok(devices.get(&mac).cloned())
    }

    /// Load device aliases from disk
    pub fn load_from_file(&self) -> anyhow::Result<()> {
        let path = self.get_devices_file_path();

        if !path.exists() {
            debug!("Devices file does not exist: {}", path.display());
            return Ok(());
        }

        let contents = fs::read_to_string(&path)?;
        let config: DeviceConfig = serde_json::from_str(&contents)?;

        let mut aliases = self.aliases.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock aliases: {}", e))?;
        
        for device_alias in config.devices {
            let mac = device_alias.mac.to_uppercase();
            if let Some(alias) = device_alias.alias {
                aliases.insert(mac, alias);
            }
        }

        info!("Loaded {} device aliases from file", aliases.len());
        Ok(())
    }

    /// Save device aliases to disk
    pub fn save_to_file(&self) -> anyhow::Result<()> {
        let devices = self.devices.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock devices: {}", e))?;

        let device_list: Vec<DeviceAlias> = devices
            .values()
            .map(|d| DeviceAlias {
                mac: d.mac.clone(),
                alias: d.alias.clone(),
                is_static: d.is_static,
            })
            .collect();

        let config = DeviceConfig {
            devices: device_list,
        };

        let path = self.get_devices_file_path();
        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&path, json)?;

        info!("Saved {} devices to {}", config.devices.len(), path.display());
        Ok(())
    }

    /// Lookup MAC vendor by OUI (first 6 characters)
    pub fn lookup_vendor(mac: &str) -> Option<String> {
        let oui = mac.split(':').take(3).collect::<Vec<&str>>().join(":").to_uppercase();
        
        // Built-in MAC OUI database
        let vendors: HashMap<&str, &str> = [
            // Apple
            ("00:1A:2B", "Apple"),
            ("00:3E:E0", "Apple"),
            ("00:50:F4", "Apple"),
            ("00:62:6E", "Apple"),
            ("00:A0:D2", "Apple"),
            ("00:B0:D0", "Apple"),
            ("00:D0:B7", "Apple"),
            ("00:E0:4C", "Apple"),
            ("00:F4:B9", "Apple"),
            ("08:00:07", "Apple"),
            ("10:93:E9", "Apple"),
            ("14:CC:20", "Apple"),
            ("1C:52:16", "Apple"),
            ("34:15:9E", "Apple"),
            ("38:C0:86", "Apple"),
            ("3C:07:54", "Apple"),
            ("40:6C:8F", "Apple"),
            ("44:2A:60", "Apple"),
            ("48:D7:05", "Apple"),
            ("4C:B1:99", "Apple"),
            ("50:EA:D6", "Apple"),
            ("5C:5A:92", "Apple"),
            ("60:A4:B7", "Apple"),
            ("64:1C:41", "Apple"),
            ("68:A8:6D", "Apple"),
            ("6C:40:08", "Apple"),
            ("70:73:CB", "Apple"),
            ("74:E5:0B", "Apple"),
            ("78:31:C1", "Apple"),
            ("7C:6D:62", "Apple"),
            ("80:E6:50", "Apple"),
            ("84:B1:53", "Apple"),
            ("88:63:DF", "Apple"),
            ("8C:85:90", "Apple"),
            ("90:84:2B", "Apple"),
            ("94:E9:79", "Apple"),
            ("98:03:47", "Apple"),
            ("9C:29:19", "Apple"),
            ("A0:88:B4", "Apple"),
            ("A4:12:69", "Apple"),
            ("A8:5B:78", "Apple"),
            ("AC:BC:32", "Apple"),
            ("B0:34:95", "Apple"),
            ("B4:0B:44", "Apple"),
            ("B8:09:8A", "Apple"),
            ("BC:52:B3", "Apple"),
            ("C0:25:06", "Apple"),
            ("C4:2C:03", "Apple"),
            ("C8:27:8D", "Apple"),
            ("CC:2D:E0", "Apple"),
            ("D0:23:BE", "Apple"),
            ("D4:61:9D", "Apple"),
            ("D8:96:95", "Apple"),
            ("DC:2B:61", "Apple"),
            ("E0:AC:CB", "Apple"),
            ("E4:8B:F5", "Apple"),
            ("E8:8D:28", "Apple"),
            ("EC:22:80", "Apple"),
            ("F0:18:98", "Apple"),
            ("F4:0F:24", "Apple"),
            ("F8:FF:C2", "Apple"),
            ("FC:3F:DB", "Apple"),
            
            // Raspberry Pi
            ("B8:27:EB", "Raspberry Pi"),
            ("DC:A6:32", "Raspberry Pi"),
            ("2C:CF:67", "Raspberry Pi"),
            ("E4:5F:01", "Raspberry Pi"),
            
            // Google
            ("00:1F:CC", "Google"),
            ("00:25:86", "Google"),
            ("00:34:C7", "Google"),
            ("00:56:2F", "Google"),
            ("00:1A:11", "Google"),
            ("AC:DE:48", "Google"),
            
            // Intel
            ("00:19:B9", "Intel"),
            ("00:1F:3C", "Intel"),
            ("00:25:86", "Intel"),
            ("08:60:6E", "Intel"),
            
            // Samsung
            ("00:07:AB", "Samsung"),
            ("00:0F:B5", "Samsung"),
            ("00:12:FB", "Samsung"),
            ("00:16:6B", "Samsung"),
            ("00:19:A0", "Samsung"),
            ("00:1E:74", "Samsung"),
            ("00:21:4C", "Samsung"),
            ("00:23:D8", "Samsung"),
            ("00:25:D3", "Samsung"),
            ("00:26:C6", "Samsung"),
            ("00:E0:64", "Samsung"),
            ("08:08:C2", "Samsung"),
            ("A0:21:95", "Samsung"),
            
            // LG Electronics
            ("00:05:B3", "LG Electronics"),
            ("00:1E:8E", "LG Electronics"),
            ("00:23:FA", "LG Electronics"),
            ("00:3F:0E", "LG Electronics"),
            ("00:48:CA", "LG Electronics"),
            
            // Sony
            ("00:02:B3", "Sony"),
            ("00:0C:6E", "Sony"),
            ("00:12:6D", "Sony"),
            ("00:1A:80", "Sony"),
            ("00:1F:A7", "Sony"),
            
            // Qualcomm (Atheros)
            ("00:0B:85", "Qualcomm"),
            ("00:13:10", "Qualcomm"),
            ("00:1F:CA", "Qualcomm"),
            ("00:22:6B", "Qualcomm"),
            ("00:24:2B", "Qualcomm"),
            
            // Broadcom
            ("00:10:BD", "Broadcom"),
            ("00:13:10", "Broadcom"),
            ("00:14:85", "Broadcom"),
            ("00:19:E3", "Broadcom"),
            
            // Motorola
            ("00:04:C1", "Motorola"),
            ("00:12:2F", "Motorola"),
            ("00:1E:67", "Motorola"),
            ("00:25:43", "Motorola"),
            
            // Arista Networks
            ("00:1C:73", "Arista Networks"),
            
            // HP
            ("00:01:E6", "HP"),
            ("00:04:EA", "HP"),
            ("00:07:01", "HP"),
            ("00:09:6B", "HP"),
            ("00:0C:02", "HP"),
            
            // Dell
            ("00:0F:1F", "Dell"),
            ("00:0F:8F", "Dell"),
            ("00:12:3F", "Dell"),
            ("00:14:4F", "Dell"),
            
            // Cisco
            ("00:00:0C", "Cisco"),
            ("00:01:42", "Cisco"),
            ("00:01:63", "Cisco"),
            ("00:01:96", "Cisco"),
            ("00:01:CA", "Cisco"),
        ]
        .iter()
        .cloned()
        .collect();

        vendors.get(oui.as_str()).map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_lookup() {
        assert_eq!(
            DeviceManager::lookup_vendor("B8:27:EB:00:00:00"),
            Some("Raspberry Pi".to_string())
        );
        assert_eq!(
            DeviceManager::lookup_vendor("00:1A:2B:00:00:00"),
            Some("Apple".to_string())
        );
        assert_eq!(
            DeviceManager::lookup_vendor("FF:FF:FF:00:00:00"),
            None
        );
    }

    #[test]
    fn test_mac_normalization() {
        let manager = DeviceManager::new().unwrap();
        let device = Device {
            mac: "aa:bb:cc:dd:ee:ff".to_string(),
            ip: "192.168.1.100".to_string(),
            hostname: "test-device".to_string(),
            alias: Some("My Device".to_string()),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            vendor: None,
            is_static: false,
        };

        manager.add_or_update(device).unwrap();
        let retrieved = manager.get_by_mac("AA:BB:CC:DD:EE:FF").unwrap();
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().mac, "AA:BB:CC:DD:EE:FF");
    }
}
