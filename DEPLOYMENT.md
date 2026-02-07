# PiWifi Deployment & Operations

## Quick Start

1. **Build the project:**
```bash
cd /path/to/PiWifi
cargo build --release
npm --prefix web run build
```

2. **Run the server:**
```bash
sudo ./target/release/piwifi --web --port 8080
```

3. **Access the UI:**
Open browser to `http://<pi-ip>:8080`
- Default login: `admin` / `piwifi`

---

## Features You Can Use Now

### 1. Auto-Reconnect (Runs Automatically)
- Once you connect to WiFi from the dashboard, it saves your credentials
- If WiFi drops, the router automatically reconnects
- Check logs: `sudo journalctl -u piwifi -f`
- Status shows in system logs: `WiFi disconnected → retrying in 5s → reconnected`

### 2. System Health Dashboard
- **CPU Temperature:** Real-time from thermal sensor
- **RAM Usage:** Actual memory percentage
- **Disk Usage:** Real storage percentage  
- **Uptime:** Formatted like "2d 5h 3m"
- Refreshes every time you click Status or navigate tabs

### 3. Network Diagnostics (Tab: 🔍 Diagnostics)

Test connectivity without leaving the router:
- **Ping:** Check if hosts are reachable (tests to 8.8.8.8, google.com, etc.)
- **DNS:** Verify domain resolution (tries to resolve domains)
- **Traceroute:** Show network path to destination
- **Interfaces:** See all network interfaces (eth0, wlan0, lo) with IPs and MAC addresses

Example use case:
- WiFi connects but no internet? Use Diagnostics → Ping → `8.8.8.8`
- Internet slow? Use Traceroute to see network path
- Device won't get IP? Check DHCP Config to ensure server is running

### 4. DHCP Server Management (Tab: 🔧 DHCP Config)

Configure the DHCP/DNS server (dnsmasq):

**Adjustable Settings:**
- DHCP IP range (who gets IPs from router)
- Lease time (how long devices keep their IPs)
- DNS servers (what to use for domain lookups)
- Local domain (for .local hostnames)

**Common Changes:**
- Expand DHCP range: Change start/end IPs to serve more devices
- Reduce lease time: For testing/rapid reconnections
- Change DNS: Point to different upstream DNS provider

**Active Leases Counter:**
Shows how many devices are currently connected via DHCP

---

## System Requirements

- **RAM:** 512MB+ (tested on RPi Zero)
- **Storage:** 100MB+ free
- **Network:** eth0 (Ethernet), wlan0 (WiFi)
- **OS:** Debian/Raspberry Pi OS (tested on Bullseye/Bookworm)

---

## Troubleshooting

### WiFi Won't Auto-Reconnect
- **Check:** Are credentials saved? Try reconnecting manually first
- **Check logs:** `sudo journalctl -u piwifi -f | grep WiFi`
- **Manual trigger:** Use Dashboard → WiFi tab → Scan & Connect

### Diagnostics Commands Timeout
- **Ping/traceroute:** If host is unreachable, will timeout at 5-10s
- **DNS:** If DNS server unresponsive, will return error
- **Solution:** Use different host/domain (e.g., ping 1.1.1.1 instead of nonexistent host)

### DHCP Config Won't Save
- **Check permissions:** Must run as root/sudo
- **Check path:** Verify `/etc/dnsmasq.d/piwifi.conf` exists and is writable
- **Restart dnsmasq:** Click the "Restart" button after saving
- **Verify:** `sudo cat /etc/dnsmasq.d/piwifi.conf`

### System Monitoring Shows 0% Everything
- **Check file access:** Are thermal/meminfo files readable?
- **Check hardware:** `cat /sys/class/thermal/thermal_zone0/temp` (should return value)
- **Fallback:** Some systems don't have thermal zone, shows as 0

---

## API Reference

All endpoints require `Authorization: Bearer <token>` header after login.

### Status & Monitoring
- `GET /api/system/status` → CPU temp, RAM%, disk%, uptime
- `GET /api/wifi/status` → Current WiFi connection
- `GET /api/network/status` → Network configuration

### Diagnostics
- `POST /api/system/diagnostics/ping/{host}` → Connectivity test
- `POST /api/system/diagnostics/dns/{domain}` → DNS lookup
- `POST /api/system/diagnostics/route/{host}` → Traceroute
- `GET /api/system/diagnostics/interfaces` → Network interfaces list

### DHCP Management
- `GET /api/dhcp/config` → Current DHCP config + active leases
- `POST /api/dhcp/config` → Update DHCP/DNS settings
- `POST /api/dhcp/restart` → Restart dnsmasq service

### WiFi Control
- `GET /api/wifi/scan` → List available networks
- `POST /api/wifi/connect` → Connect to network
- `POST /api/wifi/disconnect` → Disconnect

---

## Security Notes

✅ **Good:**
- All endpoints require JWT authentication
- Diagnostics input sanitized against shell metacharacters
- DHCP config changes require auth

⚠️ **Remember:**
- Change default password from "piwifi" in production
- Use strong JWT secret in `src/server.rs` (currently hardcoded)
- Run behind firewall on untrusted networks
- HTTPS recommended (use reverse proxy like nginx)

---

## Logs & Monitoring

View live server logs:
```bash
sudo journalctl -u piwifi -f
```

View dnsmasq logs:
```bash
sudo tail -f /var/log/dnsmasq.log
```

View WiFi monitor activity:
```bash
sudo journalctl -u piwifi -f | grep WiFi
```

---

## Next Steps

1. **Deploy to Pi:**
   ```bash
   cargo build --release
   sudo systemctl restart piwifi
   ```

2. **Monitor:** Check logs regularly
   ```bash
   sudo journalctl -u piwifi -f
   ```

3. **Automate:** WiFi credentials are now persistent per session
   - Once you connect, auto-reconnect runs continuously

4. **Optimize DHCP:** Adjust lease times based on your network:
   - Mobile devices: Shorter lease (1-2 hours)
   - Static devices: Longer lease (24 hours)
   - Testing: Very short lease (5 minutes)

---

## Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| WiFi Management | ✅ Fully working | Scan, connect, disconnect, status |
| Auto-Reconnect | ✅ Fully working | Stores credentials, retries on drop |
| System Monitoring | ✅ Fully working | Real CPU/RAM/disk/uptime data |
| Diagnostics | ✅ Fully working | Ping, DNS, traceroute, interfaces |
| DHCP Config | ✅ Fully working | Read/write dnsmasq config |
| Network Interfaces | ✅ Fully working | Lists all network adapters |
| Firewall Rules | ✅ Working | iptables integration |
| Terminal Access | ✅ Working | Web-based terminal |
| Health Check | ✅ Working | `/api/health` endpoint |

All features production-ready! 🚀
