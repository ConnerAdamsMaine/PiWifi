# PiWifi - Full Raspberry Pi WiFi Router Setup

## Overview
PiWifi is a complete Rust-based WiFi router application for Raspberry Pi 4B that:
- Connects to your main WiFi network (wlan0)
- Provides Ethernet connectivity to devices (eth0)
- Handles DHCP assignment
- Provides DNS resolution with forwarding
- Implements complete firewall rules with NAT

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Raspberry Pi 4B                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────┐       ┌──────────────────────┐  │
│  │   WiFi Interface     │       │  Ethernet Interface  │  │
│  │   (wlan0)            │◄─────►│  (eth0)              │  │
│  │  - Client mode       │  NAT  │  - Server mode       │  │
│  │  - Receives WiFi     │       │  - DHCP/DNS          │  │
│  │  - Routes via main   │       │  - Devices connect   │  │
│  │    router            │       │                      │  │
│  └──────────────────────┘       └──────────────────────┘  │
│          │                               │                  │
│          │ 192.168.1.x                   │ 192.168.100.x    │
│          │                               │                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Networking Services                                 │  │
│  │  ├─ Firewall (iptables)                              │  │
│  │  │  ├─ DROP policy by default (secure)               │  │
│  │  │  ├─ Allow DHCP/DNS/SSH/HTTP/HTTPS on eth0        │  │
│  │  │  └─ Forward WiFi ↔ Ethernet                       │  │
│  │  ├─ NAT (masquerade)                                 │  │
│  │  │  └─ Translate LAN IPs to WiFi IP                  │  │
│  │  ├─ DHCP Server (dnsmasq)                            │  │
│  │  │  └─ Lease: 192.168.100.50 - .200                 │  │
│  │  └─ DNS Server (dnsmasq)                             │  │
│  │     ├─ Local .piwifi.local domain                    │  │
│  │     └─ Forward to upstream (8.8.8.8, 8.8.4.4)        │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

### Hardware
- Raspberry Pi 4B (8GB RAM recommended)
- Micro SD card (32GB+)
- Ethernet adapter (if not built-in)
- WiFi adapter (Pi's built-in WiFi)

### Software
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
  iproute2 \
  iputils-ping
```

### Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Installation

1. **Clone the repository**
   ```bash
   git clone <repo-url>
   cd PiWifi
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Install binary**
   ```bash
   sudo cp target/release/piwifi /usr/local/bin/
   ```

## Usage

### Run the Setup
```bash
sudo /usr/local/bin/piwifi
```

This will:
1. Configure eth0 with IP 192.168.100.1/24
2. Initialize firewall with secure defaults
3. Enable NAT and IP forwarding
4. Start dnsmasq (DHCP + DNS)
5. Scan available WiFi networks
6. Display network status
7. Save firewall rules persistently

### Connect to WiFi

After running the setup, connect to your main WiFi:

```bash
# Via CLI
sudo nmcli device wifi connect "YOUR_SSID" password "YOUR_PASSWORD" ifname wlan0

# Or interactively in a Rust program (see WebUI)
```

### Configure as Service

Create `/etc/systemd/system/piwifi.service`:

```ini
[Unit]
Description=PiWifi - WiFi Router
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/piwifi
Restart=on-failure
RestartSec=10
User=root
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable piwifi
sudo systemctl start piwifi
sudo systemctl status piwifi
```

## Firewall Rules

### Default Policies
| Chain | Policy | Reason |
|-------|--------|--------|
| INPUT | DROP | Default deny |
| OUTPUT | ACCEPT | Allow all outgoing |
| FORWARD | DROP | Block unknown flows |

### Allowed Incoming (eth0)
- **Port 22/tcp** - SSH (management)
- **Port 80/tcp** - HTTP (WebUI)
- **Port 443/tcp** - HTTPS (WebUI)
- **Port 53/tcp+udp** - DNS
- **Port 67-68/udp** - DHCP
- **ICMP** - Ping diagnostics

### Port Forwarding Example
```rust
// Forward external port 8080 to internal device at 192.168.100.50:80
FirewallManager::port_forward(8080, "192.168.100.50", 80, "tcp")?;
```

### Rate Limiting
```rust
// Limit SSH to 25 connections/minute
FirewallManager::enable_rate_limit("eth0", "tcp", 22)?;
```

## DNS Configuration

The DHCP server provides DNS via dnsmasq with:
- **Local domain**: `piwifi.local`
- **Upstream DNS**: 8.8.8.8, 8.8.4.4 (configurable)
- **Cache**: 1000 entries
- **Query logging**: `/var/log/dnsmasq.log`

### Configure Upstream DNS
Edit `/etc/dnsmasq.d/piwifi.conf` or modify `NetworkConfig::dns_upstream`:

```rust
let mut config = NetworkConfig::default();
config.dns_upstream = vec![
    "1.1.1.1".to_string(),  // Cloudflare
    "1.0.0.1".to_string(),
];
```

## Monitoring

### Check DHCP leases
```bash
cat /var/lib/dnsmasq/dnsmasq.leases
```

### View DNS queries
```bash
tail -f /var/log/dnsmasq.log | grep "query"
```

### Monitor firewall drops
```bash
journalctl -f -p 4 | grep FIREWALL_DROP
```

### Check active connections
```bash
ss -tunap
netstat -tulpn
```

### IP forwarding status
```bash
cat /proc/sys/net/ipv4/ip_forward  # Should be 1
```

## Troubleshooting

### Issue: No internet on connected devices
**Solution**: Check firewall rules
```bash
sudo iptables -L -n | grep FORWARD
sudo iptables -L -n -t nat | grep POSTROUTING
```

### Issue: DHCP clients not getting addresses
**Solution**: Restart dnsmasq
```bash
sudo systemctl restart dnsmasq
sudo systemctl status dnsmasq
```

### Issue: DNS not resolving
**Solution**: Check upstream DNS connectivity
```bash
nslookup 8.8.8.8
sudo tail -f /var/log/dnsmasq.log
```

### Issue: WiFi connection drops
**Solution**: Check network-manager logs
```bash
journalctl -u NetworkManager -f
```

### Reset everything
```bash
sudo systemctl stop piwifi
sudo systemctl stop dnsmasq
sudo iptables -F
sudo iptables -P INPUT ACCEPT
sudo iptables -P OUTPUT ACCEPT
sudo iptables -P FORWARD ACCEPT
sudo iptables -t nat -F
```

## WebUI (Future)

The application will include a React-based WebUI for:
- WiFi network scanning and connection
- DHCP client management
- DNS settings
- Firewall rule management
- Network statistics and monitoring

## Module Structure

### `system.rs` - System Command Execution
- Wrapper for sudo/sh commands
- Error handling and output capture

### `wifi.rs` - WiFi Management
- Scan available networks
- Connect/disconnect from networks
- Query connection status
- Signal strength monitoring

### `network.rs` - Network Configuration
- Configure ethernet interface
- Enable/disable NAT
- Start/stop DHCP server
- Persistent configuration

### `firewall.rs` - Firewall Management
- Initialize secure defaults
- Allow/block ports
- Port forwarding
- Rate limiting
- Rule persistence

## Security Considerations

1. **Default Deny**: All incoming traffic is denied by default
2. **Established Connections**: Only allow established/related return traffic
3. **DHCP Limiting**: Max 250 concurrent leases to prevent exhaustion
4. **DNS Filtering**: Can filter specific domains
5. **Logging**: All dropped packets can be logged for analysis
6. **Persistent Rules**: Firewall rules survive reboots

## Performance Notes

- WiFi speed limited by Pi's WiFi adapter (~150 Mbps theoretical)
- Ethernet backhaul provides full network performance
- dnsmasq caching improves DNS response times
- Connection tracking in kernel (conntrack) handles up to ~65k connections

## License
MIT

## Contributing
See CONTRIBUTING.md
