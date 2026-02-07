# Enhanced System Monitoring & Diagnostics API

## Overview

The PiWifi API has been enhanced with real system monitoring and comprehensive network diagnostics endpoints. All previous hardcoded values in the system status endpoint have been replaced with actual readings from system files and commands.

## System Monitoring

### GET /api/system/status

Returns real system metrics with actual values:

**Response:**
```json
{
  "success": true,
  "data": {
    "uptime": "2d 5h 3m",
    "cpu_temp": 45.5,
    "ram_usage": 62,
    "disk_usage": 45
  }
}
```

**Implementation Details:**

#### Uptime (`get_uptime()`)
- Source: `/proc/uptime` (first value)
- Format: Converts seconds to human-readable format (e.g., "2d 5h 3m")
- Calculation:
  - Days = seconds / 86400
  - Hours = (seconds % 86400) / 3600
  - Minutes = (seconds % 3600) / 60

#### CPU Temperature (`get_cpu_temp()`)
- Primary: `/sys/class/thermal/thermal_zone0/temp` 
  - Value in millidegrees, divided by 1000 to get Celsius
- Fallback: `sensors` command output parsing
  - Looks for "Core" or "Package" lines
  - Parses temperature after '+' sign
  - Extracts numeric value before '°' symbol

#### RAM Usage (`get_ram_usage()`)
- Source: `/proc/meminfo`
- Calculates: `(MemTotal - MemAvailable) / MemTotal * 100`
- Returns: Percentage (0-100)

#### Disk Usage (`get_disk_usage()`)
- Command: `df -B1 /` (bytes, root filesystem)
- Calculates: `(Used / Total) * 100`
- Returns: Percentage (0-100)
- Fallback: Returns 0 on error, doesn't fail the entire request

---

## Diagnostics Endpoints

All diagnostics endpoints include security validation:
- **Input Sanitization**: Rejects hostnames/domains containing shell metacharacters (`;`, `|`, `&`, `$`, `` ` ``, `'`, `"`, `<`, `>`, newlines)
- **Length Validation**: Accepts 1-255 character inputs
- **Logging**: All requests logged via `tracing::debug!`
- **Error Handling**: Gracefully handles timeouts and missing commands

### POST /api/system/diagnostics/ping/{host}

Tests connectivity to a host using ICMP ping.

**Parameters:**
- `host` (path): Hostname or IP address to ping

**Response:**
```json
{
  "success": true,
  "data": {
    "success": true,
    "output": "PING example.com (93.184.216.34) 56(84) bytes of data.\n64 bytes from 93.184.216.34: icmp_seq=1 ttl=56 time=23.4 ms\n...",
    "rtt_ms": 23.4
  }
}
```

**Implementation:**
- Command: `ping -c 4 -W 5 {host}` (4 packets, 5s timeout)
- Extracts average RTT from ping statistics line
- Returns full command output for debugging
- Tolerates missing `ping` command

---

### POST /api/system/diagnostics/dns/{domain}

Performs DNS lookup using Google's public DNS.

**Parameters:**
- `domain` (path): Domain name to resolve

**Response:**
```json
{
  "success": true,
  "data": {
    "success": true,
    "records": [
      "93.184.216.34",
      "2606:2800:220:1:248:1893:25c8:1946"
    ],
    "error": null
  }
}
```

**Implementation:**
- Command: `dig +short {domain} @8.8.8.8` (Google DNS)
- Filters empty lines from output
- Returns all DNS records as separate strings
- Includes error message if query fails or returns empty

---

### POST /api/system/diagnostics/route/{host}

Traces the network path to a host.

**Parameters:**
- `host` (path): Hostname or IP address to trace

**Response:**
```json
{
  "success": true,
  "data": {
    "success": true,
    "hops": [
      "1  192.168.1.1 (192.168.1.1)  2.134 ms  2.145 ms  2.156 ms",
      "2  router.isp.com (203.0.113.1)  12.345 ms  12.456 ms  12.567 ms",
      "..."
    ],
    "error": null
  }
}
```

**Implementation:**
- Primary Command: `traceroute -m 10 {host}` (max 10 hops)
- Fallback Command: `tracert -h 10 {host}` (Windows)
- Filters header/footer lines
- Returns cleaned hop information
- Cross-platform support

---

### GET /api/system/diagnostics/interfaces

Enumerates all network interfaces and their configuration.

**Response:**
```json
{
  "success": true,
  "data": {
    "success": true,
    "interfaces": {
      "eth0": {
        "name": "eth0",
        "status": "UP",
        "addresses": [
          "192.168.1.100/24",
          "fe80::1/64"
        ],
        "mac": "b8:27:eb:ab:cd:ef"
      },
      "lo": {
        "name": "lo",
        "status": "UP",
        "addresses": [
          "127.0.0.1/8",
          "::1/128"
        ],
        "mac": ""
      }
    }
  }
}
```

**Implementation:**
- Command: `ip addr show`
- Parses interface lines (numeric prefix with colons)
- Extracts UP/DOWN status from flags
- Collects IPv4 and IPv6 addresses
- Captures MAC address from `link/ether` lines
- Returns HashMap for easy lookup by interface name

---

## Security

### Input Validation Function: `sanitize_hostname()`

All user input (hostnames, domains) passes through this validation:

```rust
fn sanitize_hostname(input: &str) -> Result<String> {
    if input.chars().any(|c| matches!(c, ';' | '|' | '&' | '$' | '`' | '\'' | '"' | '<' | '>' | '\n')) {
        return Err(anyhow::anyhow!("Invalid hostname: contains shell metacharacters"));
    }
    if input.is_empty() || input.len() > 255 {
        return Err(anyhow::anyhow!("Invalid hostname: length out of range"));
    }
    Ok(input.to_string())
}
```

**Rejected characters:**
- `;` - Command separator
- `|` - Pipe
- `&` - Background/AND operator
- `$` - Variable expansion
- `` ` `` - Command substitution
- `'`, `"` - Quote injection
- `<`, `>` - Redirection
- Newline - Multi-line injection

---

## Error Handling

All endpoints gracefully handle errors:

1. **Validation Errors** → 400 Bad Request
   ```json
   {
     "success": false,
     "data": null,
     "error": "Invalid hostname: contains shell metacharacters"
   }
   ```

2. **Command Timeouts/Missing Tools** → 200 OK with error flag
   ```json
   {
     "success": false,
     "data": {
       "success": false,
       "records": [],
       "error": "connection timed out"
     }
   }
   ```

3. **System Monitoring Fallback** → Returns zeros instead of failing
   - If `/proc/uptime` missing → "Unknown"
   - If sensors unavailable → 0.0°C
   - If df fails → 0% disk usage

---

## Asynchronous Execution

All system commands run on blocking task thread pool via `tokio::task::spawn_blocking`:
- Prevents blocking the async HTTP handler thread
- Handles potential hangs gracefully
- Supports concurrent requests

---

## Logging

Diagnostic requests are logged at DEBUG level for monitoring:
```
DEBUG api: Ping diagnostic for host: 8.8.8.8
DEBUG api: DNS diagnostic for domain: example.com
DEBUG api: Route diagnostic for host: 1.1.1.1
DEBUG api: Interface diagnostic
```

Errors logged at appropriate levels:
- Errors: `tracing::error!()`
- Warnings: `tracing::warn!()`
- Debug info: `tracing::debug!()`

---

## New API Response Structures

```rust
// System status from real monitoring
#[derive(Serialize)]
pub struct SystemStatus {
    pub uptime: String,      // "2d 5h 3m"
    pub cpu_temp: f32,       // 45.5
    pub ram_usage: u32,      // 62 (%)
    pub disk_usage: u32,     // 45 (%)
}

// Ping diagnostics result
#[derive(Serialize)]
pub struct PingResult {
    pub success: bool,
    pub output: String,      // Full command output
    pub rtt_ms: Option<f32>, // Average round-trip time
}

// DNS lookup results
#[derive(Serialize)]
pub struct DnsResult {
    pub success: bool,
    pub records: Vec<String>,    // A, AAAA, CNAME, etc.
    pub error: Option<String>,
}

// Traceroute results
#[derive(Serialize)]
pub struct RouteResult {
    pub success: bool,
    pub hops: Vec<String>,   // Hop information lines
    pub error: Option<String>,
}

// Interface information
#[derive(Serialize)]
pub struct InterfaceInfo {
    pub name: String,              // eth0, wlan0, etc.
    pub status: String,            // UP or DOWN
    pub addresses: Vec<String>,    // IPv4 and IPv6
    pub mac: String,               // MAC address
}

#[derive(Serialize)]
pub struct InterfacesResult {
    pub success: bool,
    pub interfaces: HashMap<String, InterfaceInfo>,
}
```

---

## Testing Examples

### Test real system status
```bash
curl http://localhost:8000/api/system/status
```

### Ping Google DNS
```bash
curl -X POST http://localhost:8000/api/system/diagnostics/ping/8.8.8.8
```

### DNS lookup
```bash
curl -X POST http://localhost:8000/api/system/diagnostics/dns/google.com
```

### Trace route to 1.1.1.1
```bash
curl -X POST http://localhost:8000/api/system/diagnostics/route/1.1.1.1
```

### List all network interfaces
```bash
curl http://localhost:8000/api/system/diagnostics/interfaces
```

### Test input validation
```bash
# Should be rejected
curl -X POST http://localhost:8000/api/system/diagnostics/ping/127.0.0.1\;cat\ /etc/passwd
```

---

## Build & Deployment

✅ Compiles without errors
✅ All tests pass
✅ Release build optimized

Build with: `cargo build --release`
Binary: `target/release/piwifi`

---

## Files Modified

1. **src/api.rs** - Complete rewrite of system monitoring + 4 new diagnostic endpoints
2. **src/server.rs** - Added 4 new routes for diagnostics endpoints

## Backward Compatibility

✅ All existing endpoints unchanged
✅ /api/system/status still returns same structure but with real values
✅ No breaking changes to API contracts
