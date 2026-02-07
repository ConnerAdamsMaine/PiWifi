# PiWifi Edge Router Deployment Guide

## Overview

PiWifi can be deployed as an **edge router** on your network infrastructure, providing DHCP and DNS services to multiple network segments or subnets. This guide covers DHCP Option 60/61 configuration and best practices for enterprise or advanced network setups.

## Architecture

### Traditional Deployment (Internal WiFi Bridge)
```
Main Router (192.168.1.1) ──┐
                             │
                    ┌────────▼────────┐
                    │   Raspberry Pi   │
                    │   (PiWifi)       │
                    │ wlan0: Client    │
                    │ eth0: AP Server  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
           Device1        Device2        Device3
          (192.168.    (192.168.       (192.168.
           100.x)       100.x)          100.x)
```

### Edge Router Deployment (Corporate Network)
```
Corporate Main Router
(Primary DHCP)
    │
    ├─── DHCP Server (Option 60/61 tags)
    │    Identifies PiWifi clients
    │
    │    ┌─────────────────────────┐
    └────│   Raspberry Pi (PiWifi)  │
         │   Edge Router Mode        │
         │ eth0: Corporate Network   │
         │ wlan0: Failover/Secondary │
         └──────────┬────────────────┘
                    │
         ┌──────────┼──────────┐
         │          │          │
      Printers   Cameras   IoT Devices
    (DHCP from   (DHCP from  (DHCP from
     main route  PiWifi)     PiWifi)
     via Option   with tags
     60/61)
```

## DHCP Option 60/61 Configuration

### Option 60: Vendor Class Identifier

**Purpose**: Identifies the device class to upstream DHCP servers

**RFC**: RFC 2132

**Format**: ASCII string identifying the device vendor/type

**Default PiWifi Value**: `PiWifi-EdgeRouter`

**Example**:
```ini
dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter
```

**Use Cases**:
- Identify PiWifi clients for special treatment
- Route specific traffic patterns
- Apply different QoS policies
- Track device types in DHCP logs

### Option 61: Client Identifier

**Purpose**: Provides specific hardware identification

**RFC**: RFC 2131

**Format**: Hardware address or custom identifier

**Default PiWifi Value**: Auto-detected MAC address (optional)

**Example**:
```ini
dhcp-option=option:client-id,00:1a:2b:3c:4d:5e
```

**Use Cases**:
- Specific device identification across restarts
- Reserved IP allocation
- Device tracking
- Security policies per device

## Configuration

### Set Edge Router Mode (In Code)

```rust
use piwifi::network::NetworkConfig;

let mut config = NetworkConfig::default();

// Set vendor identifier for upstream DHCP recognition
config.dhcp_option_60 = Some("PiWifi-EdgeRouter-Corporate".to_string());

// Optionally set hardware ID
config.dhcp_option_61 = Some("00:1a:2b:3c:4d:5e".to_string());

// Set vendor display name
config.dhcp_option_vendor_name = Some("PiWifi Edge Router".to_string());

NetworkManager::start_dhcp(&config)?;
```

### Set Via Configuration File (Recommended)

Create `piwifi-edge-config.json`:

```json
{
  "eth_ip": "10.0.1.254",
  "eth_netmask": "255.255.255.0",
  "eth_cidr": 24,
  "dhcp_start": "10.0.1.100",
  "dhcp_end": "10.0.1.200",
  "dns_upstream": ["8.8.8.8", "1.1.1.1"],
  "dns_domain": "corporate.local",
  "nat_enabled": false,
  "firewall_enabled": true,
  "dhcp_option_60": "PiWifi-EdgeRouter-Building-A",
  "dhcp_option_61": "00:1a:2b:3c:4d:5e",
  "dhcp_option_vendor_name": "PiWifi Edge Router Building A"
}
```

### Set Via API

**POST** `/api/network/configure`

```bash
curl -X POST http://piwifi:8080/api/network/configure \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "eth_ip": "10.0.1.254",
    "eth_netmask": "255.255.255.0",
    "eth_cidr": 24,
    "dhcp_start": "10.0.1.100",
    "dhcp_end": "10.0.1.200",
    "dns_upstream": ["8.8.8.8", "1.1.1.1"],
    "dns_domain": "corporate.local",
    "nat_enabled": false,
    "firewall_enabled": true,
    "dhcp_option_60": "PiWifi-EdgeRouter",
    "dhcp_option_61": "00:1a:2b:3c:4d:5e",
    "dhcp_option_vendor_name": "PiWifi Edge Router"
  }'
```

## Edge Router Setup Steps

### 1. Basic Network Configuration

```bash
# Configure for edge router (no NAT - bridge mode)
sudo piwifi --config piwifi-edge-config.json
```

### 2. Verify DHCP Options in dnsmasq

```bash
# Check generated config
cat /etc/dnsmasq.d/piwifi.conf | grep "dhcp-option"

# Expected output:
# dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter
# dhcp-option=option:client-id,00:1a:2b:3c:4d:5e
```

### 3. Monitor DHCP Assignments with Options

```bash
# View DHCP leases with client info
cat /var/lib/dnsmasq/dnsmasq.leases

# Monitor live DHCP traffic
sudo dnsmasq --no-daemon --debug --interface=eth0
```

### 4. Verify Option 60/61 in DHCP Responses

```bash
# Use tcpdump to capture DHCP packets
sudo tcpdump -i eth0 -nn udp port 67 or udp port 68

# Or use dhcpdump (if available)
sudo dhcpdump -i eth0
```

### 5. Register with Upstream DHCP/DNS

Contact your network administrator to:
- Register `PiWifi-EdgeRouter` as known vendor class
- Add static routes for this device
- Apply QoS policies based on Option 60
- Enable logging for Option 60 clients

## DHCP Workflow with Options 60/61

```
┌─────────────────┐
│  DHCP Client    │
│  (Device)       │
└────────┬────────┘
         │ DHCPDISCOVER (Option 60/61)
         ▼
┌─────────────────┐
│   PiWifi DHCP   │
│   (dnsmasq)     │
└────────┬────────┘
         │ Logs vendor class
         │ Matches Option 60: "PiWifi-EdgeRouter"
         │ Identifies as trusted PiWifi client
         ▼
┌─────────────────┐
│   Firewall      │
│   (iptables)    │
└────────┬────────┘
         │ Apply rules for PiWifi clients
         │ Higher QoS priority
         │ Allow specific ports
         ▼
┌─────────────────┐
│  DHCPOFFER      │
│  (IP Assigned)  │
└─────────────────┘
```

## Advanced Configuration Examples

### Multi-Building Deployment

```bash
# Building A
echo "dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter-BuildingA" >> /etc/dnsmasq.d/piwifi.conf

# Building B
echo "dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter-BuildingB" >> /etc/dnsmasq.d/piwifi.conf

# Building C
echo "dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter-BuildingC" >> /etc/dnsmasq.d/piwifi.conf
```

### VPN/Remote Site Integration

```ini
# Mark as remote site router
dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter-RemoteSite-NY-01
dhcp-option=option:client-id,00:11:22:33:44:55

# Upstream recognizes this as remote site A
# Routes accordingly via VPN tunnel
```

### QoS/Traffic Shaping

```bash
# Configure QoS for PiWifi Option 60 clients
sudo tc qdisc add dev eth0 root handle 1: htb default 11

# High priority for PiWifi devices (based on Option 60 identification)
sudo tc class add dev eth0 parent 1: classid 1:1 htb rate 1000mbit

# Medium priority for other devices
sudo tc class add dev eth0 parent 1: classid 1:2 htb rate 100mbit
```

## Monitoring & Debugging

### View DHCP Clients with Options

```bash
#!/bin/bash
# Script: show-dhcp-clients-with-options.sh

echo "DHCP Clients (PiWifi Edge Router):"
echo "=================================="

# Extract from dnsmasq log
grep "DHCPACK" /var/log/dnsmasq.log | tail -20 | while read line; do
  echo "$line"
  
  # Check if client matched Option 60
  if echo "$line" | grep -q "PiWifi"; then
    echo "  ✓ PiWifi Client (Option 60 matched)"
  fi
done
```

### Log DHCP Option 60/61 Assignments

```bash
# Enable detailed logging in dnsmasq
echo "log-dhcp" >> /etc/dnsmasq.d/piwifi.conf
echo "dhcp-option=vendor-class-identifier,PiWifi-EdgeRouter" >> /etc/dnsmasq.d/piwifi.conf

# Restart dnsmasq
sudo systemctl restart dnsmasq

# Monitor
sudo tail -f /var/log/dnsmasq.log
```

### Inspect DHCP Packets

```bash
# Capture DHCP Option 60 in packets
sudo tcpdump -i eth0 -vv "(port 67 or port 68)" | grep -A 5 "Option 60"

# Or use wireshark with DHCP filter
# Filter: bootp
```

## Firewall Rules for Edge Router

### Allow DHCP Traffic

```bash
sudo iptables -A INPUT -i eth0 -p udp --dport 67:68 -j ACCEPT
sudo iptables -A INPUT -i eth0 -p udp --sport 67:68 -j ACCEPT
```

### Allow DNS Traffic

```bash
sudo iptables -A INPUT -i eth0 -p udp --dport 53 -j ACCEPT
sudo iptables -A INPUT -i eth0 -p tcp --dport 53 -j ACCEPT
```

### Disable NAT (Bridge Mode - Recommended for Edge Router)

```bash
# Comment out NAT rules
# Packets flow through without translation
# Upstream router sees client IPs directly
```

### Allow Managed Ports (e.g., WebUI)

```bash
sudo iptables -A INPUT -i eth0 -p tcp --dport 8080 -j ACCEPT
```

## Integration with Upstream DHCP Snooping

If your upstream router supports DHCP snooping:

```bash
# Configure Option 60 in PiWifi
dhcp-option=option:vendor-class-identifier,PiWifi-EdgeRouter-Zone-1

# Upstream DHCP snooping recognizes this
# Forwards to appropriate subnet
# Applies vendor-specific policies
```

## Failover Configuration

### Primary/Secondary DHCP Setup

**Primary Server** (Main router):
```
- Option 60: "PrimaryDHCP-Building-A"
- Serves 192.168.1.0/24
```

**Secondary Server** (PiWifi):
```
- Option 60: "PiWifi-EdgeRouter-Failover-Building-A"
- Serves 10.0.1.0/24 (failover subnet)
- Upstream knows to route failover clients
```

Clients check:
1. Primary DHCP (main router) - Option 60: "PrimaryDHCP"
2. Fallback to PiWifi DHCP - Option 60: "PiWifi-EdgeRouter-Failover"

## DHCP Option Reference

| Option | Field | PiWifi Value | Purpose |
|--------|-------|--------------|---------|
| 60 | Vendor Class ID | PiWifi-EdgeRouter | Device identification |
| 61 | Client Identifier | MAC Address | Hardware tracking |
| 1 | Subnet Mask | 255.255.255.0 | Network mask |
| 3 | Router | Gateway IP | Default gateway |
| 6 | DNS Servers | 8.8.8.8, 1.1.1.1 | DNS resolution |
| 15 | Domain Name | corporate.local | Domain suffix |
| 51 | Lease Time | 3600 | IP validity (seconds) |

## Troubleshooting

### Option 60 Not Appearing in DHCP Offers

**Check config file:**
```bash
cat /etc/dnsmasq.d/piwifi.conf | grep "vendor-class"
```

**Restart dnsmasq:**
```bash
sudo systemctl restart dnsmasq
```

**Check syntax:**
```bash
sudo dnsmasq --test --conf-file=/etc/dnsmasq.d/piwifi.conf
```

### Clients Not Recognizing Option 60

**Verify DHCP Option in packet:**
```bash
sudo tcpdump -i eth0 -A port 67 or port 68 | grep -i vendor
```

**Check client DHCP logs:**
```bash
# Linux client
sudo journalctl -u systemd-networkd -n 50

# Windows client
Get-WinEvent -LogName System -FilterXPath "*[System[EventID=1024]]" | head -20
```

### MAC Address Filtering with Option 61

If using Option 61 for device tracking:

```bash
# Get MAC from dnsmasq.leases
cat /var/lib/dnsmasq/dnsmasq.leases | awk '{print $3, $4}' | sort -u
```

## Best Practices

✅ **DO:**
- Use descriptive Option 60 values (e.g., `PiWifi-EdgeRouter-LocationX`)
- Log DHCP assignments for auditing
- Document Option 60/61 meanings in network wiki
- Test with isolated subnet first
- Register with upstream DHCP admin

❌ **DON'T:**
- Use generic identifiers (avoid just "Router")
- Ignore upstream DHCP scope conflicts
- Change Option 60 frequently (breaks policies)
- Enable NAT in edge router mode (bridge mode recommended)
- Share same Option 60 across multiple PiWifi instances

## Production Deployment Checklist

- [ ] Test DHCP Option 60 in isolated network
- [ ] Verify upstream DHCP recognizes Option 60
- [ ] Document Option 60 value in network docs
- [ ] Configure firewall rules appropriately
- [ ] Test DNS resolution from edge router
- [ ] Verify no IP address conflicts
- [ ] Monitor DHCP leases for 24 hours
- [ ] Setup logging for audit trail
- [ ] Create failover/redundancy plan
- [ ] Document in runbook
- [ ] Train network team on PiWifi options
- [ ] Schedule regular backups of config

## Support & Documentation

- **RFC 2131**: DHCP Protocol Specification
- **RFC 2132**: DHCP Options and BOOTP Vendor Extensions
- **dnsmasq Manual**: https://dnsmasq.org/docs/dnsmasq-man.html
- **Option 60/61 Details**: Search "DHCP Option 60 vendor class identifier"

---

**Ready to deploy PiWifi as an enterprise edge router!** 🚀
