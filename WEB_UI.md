# PiWifi Web UI - Complete Guide

## Overview

A full-featured web dashboard for controlling your Raspberry Pi WiFi router with:
- ✅ **Authentication** - JWT-based login (default: admin/piwifi)
- ✅ **WiFi Management** - Scan, connect, disconnect
- ✅ **Network Control** - Configure eth0, DHCP, DNS
- ✅ **Firewall Rules** - Apply, block, forward ports
- ✅ **Terminal Emulator** - Read-only command interface (PTY ready)
- ✅ **System Monitoring** - CPU, RAM, disk usage

## Stack

**Backend:**
- Rust (actix-web)
- JWT authentication (jsonwebtoken)
- Password hashing (bcrypt)
- REST API on `:8080`

**Frontend:**
- Svelte 4.x
- TypeScript
- Vite
- Responsive dark theme

## Architecture

```
┌─────────────────────────────────────────────┐
│         Web Browser (HTTPS)                 │
│  Login → JWT Token → Dashboard              │
└─────────────────────────────────────────────┘
                    ↓
         Authorization: Bearer {token}
                    ↓
┌─────────────────────────────────────────────┐
│    Rust Backend (actix-web)                 │
│    ┌─────────────────────────────────────┐  │
│    │ /api/auth     - Login/Verify        │  │
│    │ /api/wifi/*   - WiFi operations     │  │
│    │ /api/network/*- Network config      │  │
│    │ /api/firewall/*- Firewall rules     │  │
│    │ /api/system/* - System status       │  │
│    └─────────────────────────────────────┘  │
│              ↓                              │
│    ┌─────────────────────────────────────┐  │
│    │  Core Libraries (Rust)              │  │
│    │  ├─ WifiManager (nmcli)             │  │
│    │  ├─ NetworkManager (ip/iptables)    │  │
│    │  ├─ FirewallManager (iptables)      │  │
│    │  └─ AuthManager (JWT/bcrypt)        │  │
│    └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
                    ↓
           System Commands (sudo)
```

## Getting Started

### Build the Frontend

```bash
cd web
npm install
npm run build
```

Output: `web/build/` directory with static files

### Run the Backend with Web UI

```bash
# Terminal mode
sudo piwifi

# Web UI mode (port 8080)
sudo piwifi --web
sudo piwifi --web --port 8080
```

### Access the Dashboard

Open browser: `http://raspberrypi.local:8080`

Default credentials:
- **Username**: `admin`
- **Password**: `piwifi`

## Components

### 1. **Login Page** (`src/components/Login.svelte`)

Features:
- Username/password authentication
- JWT token management
- Error handling
- Token storage (localStorage)

Default credentials (hardcoded, change in production):
```
admin / piwifi
```

### 2. **WiFi Panel** (`src/components/WiFiPanel.svelte`)

**Features:**
- 🔍 Scan available networks
- 📡 Connect to network with password
- ❌ Disconnect from current network
- 📊 View signal strength (-dBm)
- 🔐 Security type display

**API Endpoints:**
```
GET  /api/wifi/scan            - List networks
POST /api/wifi/connect         - Connect to SSID
GET  /api/wifi/status          - Current status
POST /api/wifi/disconnect      - Disconnect
```

### 3. **Network Panel** (`src/components/NetworkPanel.svelte`)

**Features:**
- 📱 eth0 configuration display
- 🌐 DHCP range (192.168.100.50-200)
- 🔗 DNS domain (piwifi.local)
- ✅ Upstream DNS servers (8.8.8.8, 8.8.4.4)
- 📊 Real-time status

**API Endpoints:**
```
GET  /api/network/status       - Network info
POST /api/network/configure    - Apply config
```

### 4. **Firewall Panel** (`src/components/FirewallPanel.svelte`)

**Features:**
- 🛡️ View current rules (iptables)
- ✅ Allow ports/protocols
- ❌ Block ports
- ➡️ Port forwarding (external → internal)
- 💾 Persistent rule saving

**API Endpoints:**
```
GET  /api/firewall/rules       - Current rules
POST /api/firewall/apply       - Apply new rule
POST /api/firewall/save        - Save rules
```

**Example Rule Application:**
```json
{
  "action": "allow",
  "interface": "eth0",
  "protocol": "tcp",
  "port": 8080
}
```

### 5. **Terminal Panel** (`src/components/TerminalPanel.svelte`)

**Features:**
- ⌨️ Command emulator (read-only demo)
- 📖 Help system
- 🟢 Status display
- 📝 Command history

**Available Commands (demo):**
- `help` - Show this help
- `status` - System status
- `wifi-scan` - List networks
- `wifi-status` - Current WiFi
- `network-status` - Ethernet info
- `firewall-rules` - iptables rules
- `ifconfig` - Interface config
- `ping 8.8.8.8` - Test connectivity
- `nslookup google.com` - DNS test
- `clear` - Clear screen

**PTY Integration (Coming Soon):**
```rust
// WebSocket connection for real PTY
ws://localhost:8080/api/pty/{sessionId}
```

## API Reference

### Authentication

**Login**
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "piwifi"
}
```

Response:
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 86400
  }
}
```

**Verify Token**
```http
GET /api/auth/verify
Authorization: Bearer {token}
```

### WiFi Endpoints

**Scan Networks**
```http
GET /api/wifi/scan
Authorization: Bearer {token}
```

Response:
```json
{
  "success": true,
  "data": [
    {
      "ssid": "MyNetwork",
      "signal": -45,
      "security": "WPA2"
    }
  ]
}
```

**Connect to WiFi**
```http
POST /api/wifi/connect
Authorization: Bearer {token}
Content-Type: application/json

{
  "ssid": "MyNetwork",
  "password": "mypassword"
}
```

**Get WiFi Status**
```http
GET /api/wifi/status
Authorization: Bearer {token}
```

Response:
```json
{
  "success": true,
  "data": {
    "connected": true,
    "ssid": "MyNetwork",
    "ip": "192.168.1.100",
    "signal": -45
  }
}
```

### Firewall Endpoints

**Get Current Rules**
```http
GET /api/firewall/rules
Authorization: Bearer {token}
```

**Apply Rule**
```http
POST /api/firewall/apply
Authorization: Bearer {token}
Content-Type: application/json

{
  "action": "allow",      // "allow", "block", "forward"
  "interface": "eth0",    // "eth0", "wlan0"
  "protocol": "tcp",      // "tcp", "udp"
  "port": 8080,
  "target_ip": "192.168.100.50",    // For forward
  "target_port": 80                  // For forward
}
```

**Save Rules**
```http
POST /api/firewall/save
Authorization: Bearer {token}
```

### System Endpoints

**Get System Status**
```http
GET /api/system/status
Authorization: Bearer {token}
```

Response:
```json
{
  "success": true,
  "data": {
    "uptime": "2 days 5 hours",
    "cpu_temp": 45.5,
    "ram_usage": 62,
    "disk_usage": 45
  }
}
```

**Health Check**
```http
GET /api/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2024-02-07T08:30:00Z"
}
```

## Styling

Dark modern theme with:
- **Primary**: `#667eea` / `#764ba2` (purple gradient)
- **Accent**: `#60a5fa` (blue)
- **Background**: `#0f172a` (dark slate)
- **Cards**: `#1e293b` (slate)
- **Text**: `#e2e8f0` (light slate)

### Color Codes

| Element | Color |
|---------|-------|
| Primary Button | `#3b82f6` (blue) |
| Success Button | `#22c55e` (green) |
| Danger Button | `#ef4444` (red) |
| Input Focus | `#60a5fa` (light blue) |
| Error | `#7f1d1d` (dark red bg) |

## Development

### Dev Server

```bash
cd web
npm run dev
```

Runs on `http://localhost:5173` with proxy to `http://localhost:8080/api`

### Build for Production

```bash
npm run build
```

Creates optimized bundle in `web/build/`

### Environment Variables

```bash
# .env (optional)
VITE_API_URL=http://localhost:8080
```

## Security

### Current Implementation

✅ JWT authentication  
✅ Password hashing (bcrypt)  
✅ CORS enabled  
✅ Token expiration (24 hours)  
✅ Authorization headers checked  

### Improvements Needed for Production

⚠️ Change default password  
⚠️ Use HTTPS only  
⚠️ Implement user database (not hardcoded)  
⚠️ Add rate limiting  
⚠️ Implement refresh tokens  
⚠️ Add CSRF protection  
⚠️ Validate all inputs  
⚠️ Add audit logging  

## PTY Terminal (WebSocket)

**Coming Soon** - Full implementation for real-time terminal access:

```typescript
// WebSocket connection
const ws = new WebSocket(`ws://raspberrypi.local:8080/api/pty/connect`);

ws.onmessage = (event) => {
  const { output, sessionId } = JSON.parse(event.data);
  displayTerminal(output);
};

// Send command
ws.send(JSON.stringify({
  command: 'whoami',
  sessionId: 'session-123'
}));
```

## Troubleshooting

### "401 Unauthorized"
- Check token in localStorage
- Re-login if token expired
- Verify backend is running

### "Cannot connect to API"
- Check backend is running: `sudo piwifi --web`
- Verify port 8080 is open
- Check firewall rules allow 8080

### Build fails
```bash
# Clear node_modules
rm -rf node_modules package-lock.json
npm install

# Rebuild
npm run build
```

### Frontend shows blank page
- Check browser console for errors (F12)
- Verify `web/build/` exists
- Check backend serves static files

## Deployment

### On Raspberry Pi

1. **Build frontend:**
   ```bash
   cd web && npm run build
   ```

2. **Update systemd service:**
   ```ini
   [Service]
   ExecStart=/usr/local/bin/piwifi --web --port 8080
   ```

3. **Start service:**
   ```bash
   sudo systemctl start piwifi
   sudo systemctl status piwifi
   ```

4. **Access dashboard:**
   ```
   http://raspberrypi.local:8080
   ```

### Docker (Future)

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM node:latest as web-builder
WORKDIR /app/web
COPY web .
RUN npm install && npm run build

FROM debian:bookworm
COPY --from=builder /app/target/release/piwifi /usr/local/bin/
COPY --from=web-builder /app/web/build /var/www/piwifi
EXPOSE 8080
CMD ["piwifi", "--web", "--port", "8080"]
```

## File Structure

```
web/
├── src/
│   ├── main.ts              # Svelte entry point
│   ├── App.svelte           # Root component
│   └── components/
│       ├── Login.svelte     # Authentication
│       ├── Dashboard.svelte # Main layout
│       ├── WiFiPanel.svelte
│       ├── NetworkPanel.svelte
│       ├── FirewallPanel.svelte
│       └── TerminalPanel.svelte
├── index.html               # HTML template
├── vite.config.ts           # Vite configuration
├── tsconfig.json            # TypeScript config
└── package.json             # Dependencies
```

## Performance

### Frontend Optimization
- Code splitting via Vite
- CSS tree-shaking
- Minified production build (~36 KB gzipped)
- Lazy loading of components

### Backend Optimization
- Actix-web (high-performance async)
- Connection pooling
- Request buffering
- Response compression

### Browser Caching
- Static files cached (1 year)
- API responses cached (60 seconds for status)
- JWT tokens cached (localStorage)

## Future Features

- [ ] Real-time WebSocket PTY terminal
- [ ] DHCP client management UI
- [ ] Network traffic monitoring/graphs
- [ ] Advanced port forwarding
- [ ] VPN integration
- [ ] Mobile app (React Native)
- [ ] Dark/light theme toggle
- [ ] Export configuration
- [ ] System logs viewer
- [ ] Email alerts

## Support

- 📖 See [README.md](README.md) for full docs
- 🐛 Report issues on GitHub
- 💬 Ask questions in Discussions
- 📧 Email: support@piwifi.local

---

**Built with ❤️ for Raspberry Pi**
