use crate::system::SystemCommand;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub protocol: String,
    pub port: u16,
    pub action: String, // ACCEPT, DROP, REJECT
}

pub struct FirewallManager;

impl FirewallManager {
    const ETH_INTERFACE: &'static str = "eth0";
    const WLAN_INTERFACE: &'static str = "wlan0";

    /// Initialize firewall with secure defaults
    pub fn init() -> Result<()> {
        // Flush all chains
        SystemCommand::run_sudo("iptables", &["-F"])?;
        SystemCommand::run_sudo("iptables", &["-X"])?;
        SystemCommand::run_sudo("iptables", &["-t", "nat", "-F"])?;
        SystemCommand::run_sudo("iptables", &["-t", "nat", "-X"])?;
        SystemCommand::run_sudo("iptables", &["-t", "mangle", "-F"])?;
        SystemCommand::run_sudo("iptables", &["-t", "mangle", "-X"])?;

        // Set default policies
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

        // Allow SSH from eth0 (for management)
        Self::allow_port("eth0", "tcp", 22)?;

        // Allow HTTP from eth0
        Self::allow_port("eth0", "tcp", 80)?;

        // Allow HTTPS from eth0
        Self::allow_port("eth0", "tcp", 443)?;

        // Allow DHCP on eth0
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
                "67:68",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Allow DNS on eth0
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

        // Allow ICMP (ping) for diagnostics
        SystemCommand::run_sudo("iptables", &["-A", "INPUT", "-p", "icmp", "-j", "ACCEPT"])?;

        // Enable FORWARD for WiFi -> Ethernet
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

        // Enable FORWARD for Ethernet -> WiFi (established only)
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

        Ok(())
    }

    /// Allow incoming traffic on a specific port
    pub fn allow_port(interface: &str, protocol: &str, port: u16) -> Result<()> {
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                interface,
                "-p",
                protocol,
                "--dport",
                &port.to_string(),
                "-m",
                "state",
                "--state",
                "NEW",
                "-j",
                "ACCEPT",
            ],
        )?;
        Ok(())
    }

    /// Block incoming traffic on a specific port
    pub fn block_port(interface: &str, protocol: &str, port: u16) -> Result<()> {
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                interface,
                "-p",
                protocol,
                "--dport",
                &port.to_string(),
                "-j",
                "DROP",
            ],
        )?;
        Ok(())
    }

    /// Enable rate limiting on a port (DDoS protection)
    pub fn enable_rate_limit(interface: &str, protocol: &str, port: u16) -> Result<()> {
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-i",
                interface,
                "-p",
                protocol,
                "--dport",
                &port.to_string(),
                "-m",
                "limit",
                "--limit",
                "25/minute",
                "--limit-burst",
                "100",
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
                interface,
                "-p",
                protocol,
                "--dport",
                &port.to_string(),
                "-j",
                "DROP",
            ],
        )?;

        Ok(())
    }

    /// Set up port forwarding (e.g., external:8080 -> internal:80)
    pub fn port_forward(
        external_port: u16,
        internal_ip: &str,
        internal_port: u16,
        protocol: &str,
    ) -> Result<()> {
        // PREROUTING: redirect incoming traffic
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "PREROUTING",
                "-p",
                protocol,
                "--dport",
                &external_port.to_string(),
                "-j",
                "DNAT",
                "--to-destination",
                &format!("{}:{}", internal_ip, internal_port),
            ],
        )?;

        // FORWARD: allow the redirected traffic
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "FORWARD",
                "-p",
                protocol,
                "-d",
                internal_ip,
                "--dport",
                &internal_port.to_string(),
                "-m",
                "state",
                "--state",
                "NEW,RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )?;

        // Allow return traffic
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "FORWARD",
                "-p",
                protocol,
                "-s",
                internal_ip,
                "--sport",
                &internal_port.to_string(),
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
        )?;

        Ok(())
    }

    /// Log dropped packets
    pub fn enable_logging() -> Result<()> {
        SystemCommand::run_sudo(
            "iptables",
            &[
                "-A",
                "INPUT",
                "-j",
                "LOG",
                "--log-prefix",
                "FIREWALL_DROP: ",
                "--log-level",
                "7",
            ],
        )?;

        Ok(())
    }

    /// Show current firewall rules
    pub fn show_rules() -> Result<String> {
        SystemCommand::run_sudo("iptables", &["-L", "-n", "-v"])
    }

    /// Save firewall rules persistently
    pub fn save_rules() -> Result<()> {
        SystemCommand::run_sudo("sh", &[
            "-c",
            "mkdir -p /etc/iptables && iptables-save > /etc/iptables/rules.v4",
        ])?;
        Ok(())
    }

    /// Restore firewall rules from saved state
    pub fn restore_rules() -> Result<()> {
        SystemCommand::run_sudo("iptables-restore", &["<", "/etc/iptables/rules.v4"])?;
        Ok(())
    }
}
