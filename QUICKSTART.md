# PiWifi - Quick Start Guide

## 🚀 30-Second Setup

```bash
# 1. Clone & navigate
git clone <repo> && cd PiWifi

# 2. Deploy (automatic install)
sudo ./deploy.sh

# 3. Connect to WiFi
sudo nmcli device wifi connect "YOUR_SSID" password "YOUR_PASSWORD" ifname wlan0

# 4. Open browser
# http://raspberrypi.local:8080
# Login: admin / piwifi
```

Done! 🎉

---

## Manual Setup (If Deploy Script Fails)

### Prerequisites
```bash
sudo apt update && sudo apt install -y \
  build-essential curl git network-manager \
  dnsmasq iptables iptables-persistent iproute2
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build & Install

```bash
# Build backend
cargo build --release

# Build frontend
cd web && npm install && npm run build && cd ..

# Install binary
sudo cp target/release/piwifi /usr/local/bin/

# Install service
sudo cp piwifi.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable piwifi
```

### Run

```bash
# Initialize network
sudo piwifi

# Start web server
sudo systemctl start piwifi

# Or run directly
sudo piwifi --web --port 8080
```

### Access
- **URL**: `http://raspberrypi.local:8080`
- **Login**: `admin` / `piwifi`

---

## What It Does

```
Your Main WiFi ──────────────────────┐
                                     │
                                     ▼
                         ┌──────────────────────┐
                         │  Raspberry Pi 4B     │
                         │  (PiWifi Running)    │
                         │                      │
                         │  wlan0: Client mode  │
                         │  eth0: Server mode   │
                         └──────────────────────┘
                             ▲        ▲
                             │        │
                    ┌────────┘        └────────┐
                    │                         │
                    │                         │
            Device 1                    Device 2
         (gets DHCP IP)             (gets DHCP IP)
        192.168.100.x               192.168.100.y
```

### Features

✅ **WiFi**: Connect your Pi to your main router  
✅ **DHCP**: Automatically assign IPs to connected devices  
✅ **DNS**: Resolve domain names (.piwifi.local)  
✅ **NAT**: Route traffic WiFi ↔ Ethernet  
✅ **Firewall**: Secure-by-default rules  
✅ **Web UI**: Control everything from browser  

---

## Common Commands

### View Logs
```bash
sudo journalctl -u piwifi -f
```

### Check Status
```bash
sudo systemctl status piwifi
ip addr show eth0
ps aux | grep piwifi
```

### Stop/Start Service
```bash
sudo systemctl stop piwifi
sudo systemctl start piwifi
sudo systemctl restart piwifi
```

### Manually Configure WiFi
```bash
# Scan networks
sudo nmcli dev wifi list

# Connect
sudo nmcli dev wifi connect "SSID" password "PASS" ifname wlan0

# Disconnect
sudo nmcli dev disconnect wlan0
```

### View Firewall Rules
```bash
sudo iptables -L -n -v
```

### Reset Everything
```bash
sudo systemctl stop piwifi
sudo iptables -F
sudo iptables -P INPUT ACCEPT
sudo iptables -P OUTPUT ACCEPT
sudo iptables -P FORWARD ACCEPT
```

---

## Web UI Quick Tour

### 📶 WiFi Panel
- Scan for networks
- Connect with password
- View current connection
- See signal strength
- Disconnect option

### 🌐 Network Panel
- View eth0 configuration
- DHCP range (192.168.100.50-200)
- DNS domain (piwifi.local)
- Upstream DNS servers

### 🛡️ Firewall Panel
- View iptables rules
- Allow ports
- Block ports
- Forward ports
- Save rules persistently

### ⌨️ Terminal Panel
- Run commands (demo)
- View status
- Test connectivity
- Network diagnostics

---

## Test It Works

### From a Device on Ethernet

```bash
# Get IP (should be 192.168.100.x)
ip addr

# Ping router
ping 192.168.100.1

# Resolve DNS
nslookup google.com

# Test internet
curl https://api.github.com

# Check DHCP lease
cat /var/lib/dnsmasq/dnsmasq.leases
```

### From Raspberry Pi

```bash
# Check WiFi
nmcli dev status

# Check network
ip addr show

# Check DNS
cat /etc/dnsmasq.d/piwifi.conf

# Test forwarding
cat /proc/sys/net/ipv4/ip_forward  # Should be 1

# View firewall
sudo iptables -L
```

---

## Troubleshooting

### Device not getting IP
```bash
# Restart DHCP
sudo systemctl restart dnsmasq

# Check dnsmasq logs
sudo tail -f /var/log/dnsmasq.log
```

### No internet on device
```bash
# Check routing
ip route

# Check NAT rules
sudo iptables -L -t nat

# Enable IP forwarding
sudo sysctl -w net.ipv4.ip_forward=1
```

### WiFi disconnects
```bash
# Check network manager
sudo systemctl status NetworkManager

# Reconnect
sudo nmcli dev wifi connect "SSID" password "PASS" ifname wlan0
```

### Web UI not accessible
```bash
# Check if service running
sudo systemctl status piwifi

# Check port 8080
sudo ss -tulpn | grep 8080

# Restart service
sudo systemctl restart piwifi
```

### Reset password
Edit `src/auth.rs` (hardcoded for now):
```rust
// Change this:
if req.username != "admin" || req.password != "piwifi" {
```

---

## Default Configuration

| Setting | Value |
|---------|-------|
| **Ethernet IP** | 192.168.100.1/24 |
| **DHCP Range** | 192.168.100.50-200 |
| **DNS Domain** | piwifi.local |
| **Upstream DNS** | 8.8.8.8, 8.8.4.4 |
| **Web Port** | 8080 |
| **Default User** | admin / piwifi |
| **Token Expiry** | 24 hours |

---

## Next Steps

1. **Change Default Password**
   - Edit `src/auth.rs` line ~95
   - Rebuild: `cargo build --release`

2. **Use HTTPS**
   - Install SSL certificate
   - Configure actix-web with TLS
   - Update browser bookmarks

3. **Add More Users**
   - Implement database (SQLite/PostgreSQL)
   - Replace hardcoded credentials
   - Add user management UI

4. **Enable Real Terminal**
   - Uncomment PTY code
   - Implement WebSocket streaming
   - Add to TerminalPanel.svelte

5. **Monitor Traffic**
   - Add bandwidth graphs
   - Log connections
   - Create dashboard widgets

---

## Files to Know

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point |
| `src/server.rs` | Web server |
| `src/api.rs` | REST endpoints |
| `web/src/App.svelte` | Frontend root |
| `web/src/components/*` | UI panels |
| `piwifi.service` | Systemd config |
| `deploy.sh` | Installation script |

---

## Getting Help

📖 Full docs: `README.md`  
🏗️ Architecture: `SETUP.md`  
💻 Web UI guide: `WEB_UI.md`  
📝 Implementation: `IMPLEMENTATION_SUMMARY.md`  

---

## One-Liner Deployment

```bash
git clone <repo> && cd PiWifi && sudo bash -c './deploy.sh && systemctl start piwifi' && sleep 2 && echo "✅ Access http://raspberrypi.local:8080 (admin/piwifi)"
```

---

**That's it! Enjoy your WiFi pineapple! 🍍**
