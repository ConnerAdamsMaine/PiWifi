use crate::system::SystemCommand;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub eth_ip: String,              // e.g., "192.168.100.1"
    pub eth_netmask: String,         // e.g., "255.255.255.0"
    pub eth_cidr: u8,                // e.g., 24
    pub dhcp_start: String,          // e.g., "192.168.100.50"
    pub dhcp_end: String,            // e.g., "192.168.100.200"
    pub dns_upstream: Vec<String>,   // Upstream DNS servers
    pub dns_domain: String,          // Local domain
    pub nat_enabled: bool,
    pub firewall_enabled: bool,
    // DHCP Options for edge router deployment
    pub dhcp_option_60: Option<String>,  // Vendor Class Identifier
    pub dhcp_option_61: Option<String>,  // Client Identifier (hardware address)
    pub dhcp_option_vendor_name: Option<String>, // Human-readable vendor name
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            eth_ip: "192.168.100.1".to_string(),
            eth_netmask: "255.255.255.0".to_string(),
            eth_cidr: 24,
            dhcp_start: "192.168.100.50".to_string(),
            dhcp_end: "192.168.100.200".to_string(),
            dns_upstream: vec![
                "8.8.8.8".to_string(),
                "8.8.4.4".to_string(),
            ],
            dns_domain: "piwifi.local".to_string(),
            nat_enabled: true,
            firewall_enabled: true,
            dhcp_option_60: Some("PiWifi-EdgeRouter".to_string()),
            dhcp_option_61: None,
            dhcp_option_vendor_name: Some("PiWifi".to_string()),
        }
    }
}

pub struct NetworkManager;

impl NetworkManager {
    const ETH_INTERFACE: &'static str = "eth0";
    const WLAN_INTERFACE: &'static str = "wlan0";

    /// Configure Ethernet interface with static IP
    pub fn configure_eth(config: &NetworkConfig) -> Result<()> {
        // Flush existing addresses
        let _ = SystemCommand::run_sudo("ip", &["addr", "flush", "dev", Self::ETH_INTERFACE]);

        // Set IP address on eth0
        SystemCommand::run_sudo(
            "ip",
            &[
                "addr",
                "add",
                &format!("{}/{}", &config.eth_ip, config.eth_cidr),
                "dev",
                Self::ETH_INTERFACE,
            ],
        )?;

        // Bring interface up
        SystemCommand::run_sudo("ip", &["link", "set", Self::ETH_INTERFACE, "up"])?;

        // Set routes
        SystemCommand::run_sudo(
            "ip",
            &["route", "add", "default", "via", &config.eth_ip],
        ).ok();

        // Enable IP forwarding
        SystemCommand::run_sudo("sysctl", &["-w", "net.ipv4.ip_forward=1"])?;

        Ok(())
    }

    /// Configure NAT for WiFi -> Ethernet forwarding
    pub fn enable_nat(_config: &NetworkConfig) -> Result<()> {
        // Flush existing rules
        SystemCommand::run_sudo("iptables", &["-F", "FORWARD"])?;
        SystemCommand::run_sudo("iptables", &["-F", "INPUT"])?;
        SystemCommand::run_sudo("iptables", &["-F", "OUTPUT"])?;
        SystemCommand::run_sudo("iptables", &["-t", "nat", "-F", "POSTROUTING"])?;
        SystemCommand::run_sudo("iptables", &["-t", "nat", "-F", "PREROUTING"])?;

        // Set default policies - deny by default, allow established
        SystemCommand::run_sudo("iptables", &["-P", "INPUT", "DROP"])?;
        SystemCommand::run_sudo("iptables", &["-P", "OUTPUT", "ACCEPT"])?;
        SystemCommand::run_sudo("iptables", &["-P", "FORWARD", "DROP"])?;

        // Allow loopback
        SystemCommand::run_sudo("iptables", &["-A", "INPUT", "-i", "lo", "-j", "ACCEPT"])?;

        // Allow established/related connections
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Allow DHCP requests on eth0
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                Self::ETH_INTERFACE,
                "-p",
                "udp",
                "--dport",
                "67",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Allow DNS queries on eth0
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                Self::ETH_INTERFACE,
                "-p",
                "udp",
                "--dport",
                "53",
                "-j",
                "ACCEPT",
            ],
        )?;

        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                Self::ETH_INTERFACE,
                "-p",
                "tcp",
                "--dport",
                "53",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Forward: wlan0 -> eth0 (WiFi to LAN)
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "FORWARD",
                "-i",
                Self::WLAN_INTERFACE,
                "-o",
                Self::ETH_INTERFACE,
                "-m",
                "state",
                "--state",
                "NEW,RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Forward: eth0 -> wlan0 (LAN to WiFi - established only)
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "FORWARD",
                "-i",
                Self::ETH_INTERFACE,
                "-o",
                Self::WLAN_INTERFACE,
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )?;

        // NAT masquerade for WiFi interface (translate LAN IPs to WiFi IP)
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-o",
                Self::WLAN_INTERFACE,
                "-j",
                "MASQUERADE",
            ],
        )?;

        // Save iptables rules persistently
        Self::save_iptables_rules()?;

        Ok(())
    }

    /// Save iptables rules persistently
    pub fn save_iptables_rules() -> Result<()> {
        // Debian/Raspberry Pi OS way - iptables-persistent
        SystemCommand::run_sudo("sh", &[
            "-c",
            "iptables-save > /etc/iptables/rules.v4",
        ])?;

        Ok(())
    }

    /// Disable NAT
    pub fn disable_nat() -> Result<()> {
        SystemCommand::run_sudo("iptables", &["-F", "FORWARD"])?;
        SystemCommand::run_sudo("iptables", &["-t", "nat", "-F", "POSTROUTING"])?;
        Ok(())
    }

    /// Start DHCP server (dnsmasq) on eth0 with DNS forwarding
    pub fn start_dhcp(config: &NetworkConfig) -> Result<()> {
        // Create dnsmasq config with DNS forwarding
        let upstream_dns = config
            .dns_upstream
            .iter()
            .map(|dns| format!("server={}", dns))
            .collect::<Vec<_>>()
            .join("\n");

        // Build DHCP options for edge router deployment
        let mut dhcp_options = String::new();
        
        // Option 60: Vendor Class Identifier
        if let Some(vendor_class) = &config.dhcp_option_60 {
            dhcp_options.push_str(&format!("dhcp-option=option:vendor-class-identifier,{}\n", vendor_class));
        }
        
        // Option 61: Client Identifier (hardware address)
        if let Some(client_id) = &config.dhcp_option_61 {
            dhcp_options.push_str(&format!("dhcp-option=option:client-id,{}\n", client_id));
        }

        let dnsmasq_config = format!(
            r#"# PiWifi dnsmasq configuration
# DHCP server
interface={}
bind-interfaces
listen-address={}
dhcp-range={},{}
dhcp-lease-max=250
dhcp-option=option:router,{}
dhcp-option=option:dns-server,{}

# DHCP Edge Router Options (RFC 2132)
# Option 60: Vendor Class Identifier - identifies this as PiWifi router to upstream DHCP
# Option 61: Client Identifier - specific hardware identification
{}

# DNS settings
domain={}
local=/{}
expand-hosts

# Upstream DNS servers
{}

# Performance
cache-size=1000
log-queries
log-facility=/var/log/dnsmasq.log

# Don't read /etc/hosts
no-hosts
no-resolv

# Security
server=/google.com/8.8.8.8
address=/#/127.0.0.1
"#,
            Self::ETH_INTERFACE,
            config.eth_ip,
            config.dhcp_start,
            config.dhcp_end,
            config.eth_ip,
            config.eth_ip,
            dhcp_options.trim_end(),
            config.dns_domain,
            config.dns_domain,
            upstream_dns
        );

        // Create directory if needed
        std::fs::create_dir_all("/etc/dnsmasq.d")?;

        // Write config
        std::fs::write("/etc/dnsmasq.d/piwifi.conf", dnsmasq_config)?;

        // Enable and restart dnsmasq
        SystemCommand::run_sudo("systemctl", &["enable", "dnsmasq"])?;
        SystemCommand::run_sudo("systemctl", &["restart", "dnsmasq"])?;

        Ok(())
    }

    /// Stop DHCP server
    pub fn stop_dhcp() -> Result<()> {
        SystemCommand::run_sudo("systemctl", &["stop", "dnsmasq"])?;
        Ok(())
    }

    /// Get current network status
    pub fn status() -> Result<String> {
        let eth_status = SystemCommand::run("ip", &["-4", "addr", "show", Self::ETH_INTERFACE])?;
        let forward_enabled = SystemCommand::run("cat", &["/proc/sys/net/ipv4/ip_forward"])?;
        
        Ok(format!(
            "ETH0:\n{}\nIP Forwarding: {}",
            eth_status, forward_enabled.trim()
        ))
    }
}
