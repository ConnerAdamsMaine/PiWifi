# PiWifi 🔌 WiFi Router for Raspberry Pi 4B

A complete, production-ready Rust application that transforms a Raspberry Pi into a seamless WiFi access point and router. Receive WiFi from your main router, bridge it through Ethernet to connected devices with automatic DHCP/DNS and a fully secured firewall.

## ⚡ Quick Start

```bash
# 1. Clone and navigate
git clone <repo> && cd PiWifi

# 2. Deploy (automatic setup)
sudo ./deploy.sh

# 3. Start the service
sudo systemctl start piwifi

# 4. Connect to your main WiFi
sudo nmcli device wifi connect "YOUR_SSID" password "YOUR_PASSWORD" ifname wlan0

# 5. Done! Connect devices to ethernet
```

## 🎯 Features

### ✅ WiFi Management
- Scan available networks
- Connect/disconnect from WiFi
- Real-time signal monitoring
- NetworkManager integration (modern, reliable)

### ✅ Network Services
- **DHCP Server**: Auto IP assignment (192.168.100.50-200)
- **DHCP Options**: RFC 2132 Option 60/61 for edge router deployments
- **DNS Server**: caching with upstream forwarding (8.8.8.8, 8.8.4.4)
- **Local domain**: `.piwifi.local` support
- **IP Forwarding**: Seamless WiFi ↔ Ethernet bridging

### ✅ Firewall (iptables)
- **Secure by default**: DROP policy on INPUT chain
- **Selective allow**: Only necessary services (DHCP, DNS, SSH, HTTP, HTTPS)
- **NAT**: WiFi traffic masqueraded through Ethernet (or bridge mode)
- **Connection tracking**: State-aware filtering
- **Persistent rules**: Survive reboots
- **Port forwarding**: Advanced routing capabilities
- **Rate limiting**: DDoS protection on critical ports

### ✅ Edge Router Mode
- **DHCP Option 60**: Vendor Class Identifier for upstream recognition
- **DHCP Option 61**: Client Identifier for device tracking
- **Bridge mode**: No NAT translation (transparent forwarding)
- **Multi-subnet support**: Multiple network segments
- **Enterprise integration**: Works with corporate DHCP/DNS

### ✅ Security
- No unauthorized access by default
- Isolated networks (eth0 ↔ wlan0)
- Firewall logging available
- Systemd hardening options
- WiFi password required
- JWT authentication for web UI

## 📊 Network Architecture

```
                    Raspberry Pi 4B
                  
    Main WiFi Router ──►  wlan0  ◄─────────────┐
                          (Client)              │
                            │                   │
                    [NAT Masquerade]            │
                            │                   │
                          eth0                  │
                        (Server)                │
                         │                      │
                    192.168.100.1               │
                    DHCP: 50-200                │
                    DNS: dnsmasq                │
                         │                      │
            ┌────────────┴────────────┐         │
            │                         │         │
        [Device 1]              [Device 2]  [Device N]
     192.168.100.50          192.168.100.51
```

## 📋 Configuration

Default `NetworkConfig`:
```rust
eth_ip: "192.168.100.1"
eth_netmask: "255.255.255.0"  (CIDR /24)
dhcp_start: "192.168.100.50"
dhcp_end: "192.168.100.200"
dns_upstream: ["8.8.8.8", "8.8.4.4"]  // Configurable
dns_domain: "piwifi.local"
nat_enabled: true
firewall_enabled: true
```

### Customize Config
Edit in code and rebuild:
```rust
// src/main.rs
let mut config = NetworkConfig::default();
config.eth_ip = "192.168.50.1".to_string();
config.eth_cidr = 24;
config.dns_upstream = vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()];
```

Or via JSON config file (future webUI):
```json
{
  "eth_ip": "192.168.100.1",
  "eth_cidr": 24,
  "dhcp_start": "192.168.100.50",
  "dhcp_end": "192.168.100.200",
  "dns_upstream": ["8.8.8.8", "8.8.4.4"],
  "dns_domain": "piwifi.local"
}
```

## 🔧 Installation

### Prerequisites
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  git \
  network-manager \
  dnsmasq \
  iptables \
  iptables-persistent \
  iproute2
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build & Deploy
```bash
# Automated (recommended)
sudo ./deploy.sh

# Manual
cargo build --release
sudo cp target/release/piwifi /usr/local/bin/
sudo cp piwifi.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable piwifi
sudo systemctl start piwifi
```

## 🚀 Usage

### Run Manually
```bash
sudo piwifi
```

Output:
```
╔═══════════════════════════════════════╗
║  PiWifi - Raspberry Pi WiFi Router    ║
║  Full Network Setup with NAT/DNS      ║
╚═══════════════════════════════════════╝

📋 Configuration:
  ETH0 IP:     192.168.100.1
  DHCP Range:  192.168.100.50 - 192.168.100.200
  DNS Domain:  piwifi.local
  Upstream DNS: 8.8.8.8, 8.8.4.4

▶ Step 1: Configure Ethernet Interface (eth0)
  ✓ Ethernet configured with IP 192.168.100.1/24

▶ Step 2: Initialize Firewall
  ✓ Firewall initialized with secure defaults
  - Default policy: DROP (secure)
  - SSH (22), HTTP (80), HTTPS (443) allowed on eth0
  - DHCP (67-68) and DNS (53) enabled
  - WiFi → Ethernet forwarding enabled

▶ Step 3: Enable NAT & IP Forwarding
  ✓ NAT configured
  - WiFi traffic masqueraded over Ethernet
  - IP forwarding enabled

▶ Step 4: Start DHCP & DNS Server (dnsmasq)
  ✓ dnsmasq configured and started
  - DHCP range: 192.168.100.50 - 192.168.100.200
  - DNS forwarding to 8.8.8.8, 8.8.4.4
  - Local domain: piwifi.local

▶ Step 5: Persist Firewall Rules
  ✓ Rules saved to /etc/iptables/rules.v4

▶ Step 6: Scan Available WiFi Networks
  ✓ Found 7 networks:
    - MyMainRouter (Signal: -45)
    - Neighbor WiFi (Signal: -68)
    - ...

[Setup Complete!]
```

### Run as Service
```bash
# Start
sudo systemctl start piwifi

# Stop
sudo systemctl stop piwifi

# Status
sudo systemctl status piwifi

# Logs
sudo journalctl -u piwifi -f

# Auto-start on boot
sudo systemctl enable piwifi
```

## 🌐 Connect Devices

### Option 1: DHCP (Automatic)
Connect an Ethernet cable from Pi's eth0 to your device:
- Gets IP automatically: 192.168.100.x
- Gets DNS automatically: resolves via piwifi.local

### Option 2: Static IP
```bash
# On your device (manual)
sudo ip addr add 192.168.100.101/24 dev eth0
sudo ip route add default via 192.168.100.1
echo "nameserver 192.168.100.1" | sudo tee /etc/resolv.conf
```

### Test Connectivity
```bash
# From connected device
ping 192.168.100.1
ping 8.8.8.8
nslookup google.com
curl https://api.github.com
```

## 📡 Firewall Rules

### View Current Rules
```bash
sudo iptables -L -n
sudo iptables -L -n -t nat
```

### Allowed Services (eth0)
| Port | Protocol | Service |
|------|----------|---------|
| 22 | TCP | SSH |
| 53 | TCP/UDP | DNS |
| 67-68 | UDP | DHCP |
| 80 | TCP | HTTP |
| 443 | TCP | HTTPS |
| ICMP | - | Ping |

### Advanced Rules (Programmatic)

```rust
use piwifi::FirewallManager;

// Allow custom port
FirewallManager::allow_port("eth0", "tcp", 8080)?;

// Block port
FirewallManager::block_port("eth0", "tcp", 3306)?;

// Port forwarding (external:8080 → 192.168.100.50:80)
FirewallManager::port_forward(8080, "192.168.100.50", 80, "tcp")?;

// Rate limiting (DDoS protection)
FirewallManager::enable_rate_limit("eth0", "tcp", 22)?;

// Save changes persistently
FirewallManager::save_rules()?;
```

## 🔍 Monitoring & Diagnostics

### DHCP Leases
```bash
cat /var/lib/dnsmasq/dnsmasq.leases
```

### DNS Queries
```bash
tail -f /var/log/dnsmasq.log
```

### Network Interfaces
```bash
ip addr show
ip route show
```

### Firewall Drops
```bash
sudo iptables -L -n | grep DROP
```

### Active Connections
```bash
ss -tunap
netstat -tulpn
```

### Ping Main Router
```bash
ping 192.168.1.1  # Adjust to your router's IP
```

## 🐛 Troubleshooting

### Issue: No internet on connected devices
```bash
# Check firewall rules
sudo iptables -L FORWARD -n
sudo iptables -L -t nat -n

# Check IP forwarding
cat /proc/sys/net/ipv4/ip_forward  # Should be 1

# Fix
sudo sysctl -w net.ipv4.ip_forward=1
```

### Issue: DHCP not working
```bash
# Restart dnsmasq
sudo systemctl restart dnsmasq

# Check status
sudo systemctl status dnsmasq

# Check logs
sudo journalctl -u dnsmasq -f
```

### Issue: DNS not resolving
```bash
# Test upstream DNS
nslookup google.com 8.8.8.8

# Check dnsmasq config
cat /etc/dnsmasq.d/piwifi.conf

# Test DNS resolution
dig @192.168.100.1 google.com
```

### Issue: WiFi connection drops
```bash
# Check NetworkManager
sudo systemctl status NetworkManager
sudo journalctl -u NetworkManager -f

# Re-scan networks
sudo nmcli device wifi rescan

# Reconnect
sudo nmcli device wifi connect "SSID" password "PASS" ifname wlan0
```

### Reset Everything
```bash
sudo systemctl stop piwifi dnsmasq
sudo iptables -F
sudo iptables -P INPUT ACCEPT
sudo iptables -P OUTPUT ACCEPT
sudo iptables -P FORWARD ACCEPT
```

## 📁 File Structure

```
PiWifi/
├── src/
│   ├── main.rs          # Entry point & initialization
│   ├── lib.rs           # Module exports
│   ├── system.rs        # System command execution
│   ├── wifi.rs          # WiFi management (nmcli)
│   ├── network.rs       # Network config (eth0, DHCP, DNS)
│   └── firewall.rs      # Firewall rules (iptables)
├── Cargo.toml           # Rust dependencies
├── piwifi.service       # Systemd service file
├── deploy.sh            # Automated deployment script
├── SETUP.md             # Detailed setup guide
└── README.md            # This file
```

## 🚧 Upcoming Features

- [ ] Real-time WebSocket PTY terminal
- [ ] Advanced port forwarding UI
- [ ] Network traffic graphs & bandwidth monitoring
- [ ] DHCP client lease management UI
- [ ] Custom DNS rules & filtering
- [ ] dnsmasq log viewer with filtering
- [ ] Configuration file (JSON/YAML) import/export
- [ ] Email alerts & notifications
- [ ] VPN integration (WireGuard/OpenVPN)
- [ ] Mobile app (React Native)
- [ ] Database backend for users/permissions
- [ ] Audit logging & system events
- [ ] REST API documentation (OpenAPI/Swagger)

## ⚙️ System Requirements

| Component | Requirement |
|-----------|-------------|
| Hardware | Raspberry Pi 4B |
| RAM | 2GB+ (4GB+ recommended) |
| Storage | 16GB SD card+ |
| OS | Raspberry Pi OS (Debian 11+) |
| Network | WiFi adapter + Ethernet |

## 📊 Performance

| Metric | Expected |
|--------|----------|
| WiFi → Ethernet throughput | ~150 Mbps (Pi WiFi limit) |
| Ethernet backhaul | Full gigabit (if available) |
| DNS query latency | <5ms (cached), ~50ms (upstream) |
| Max DHCP clients | 250 (configurable) |
| Max connections tracked | ~65k (kernel default) |

## 🔐 Security

- ✅ Default-deny firewall (DROP policy)
- ✅ No unnecessary services exposed
- ✅ WiFi password required
- ✅ State-aware connection tracking
- ✅ Isolated networks (no LAN ↔ Internet)
- ✅ DHCP limiting
- ✅ DNS filtering available
- ✅ Systemd security hardening
- ✅ Firewall rules persistent across reboots

## 📝 License

MIT License - See LICENSE file

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

## 📧 Support

- 📖 Documentation: See SETUP.md
- 🐛 Issues: GitHub Issues
- 💬 Discussions: GitHub Discussions

## 🙏 Acknowledgments

- Raspberry Pi Foundation
- NetworkManager community
- dnsmasq project
- iptables/netfilter project
- Rust async ecosystem (tokio, actix)

---

**Ready to set up your WiFi pineapple? Run `sudo ./deploy.sh` now!** 🍍
