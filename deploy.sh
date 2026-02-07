#!/bin/bash
set -e

# PiWifi Deployment Script for Raspberry Pi 4B
# Usage: sudo ./deploy.sh

if [ "$EUID" -ne 0 ]; then 
  echo "❌ This script must be run as root (use: sudo ./deploy.sh)"
  exit 1
fi

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║          PiWifi Deployment Script                        ║"
echo "║  Raspberry Pi WiFi Router Setup                          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Install dependencies
echo "▶ Step 1: Installing dependencies..."
apt-get update
apt-get install -y \
  network-manager \
  dnsmasq \
  iptables \
  iptables-persistent \
  iproute2 \
  iputils-ping \
  curl

echo "  ✓ Dependencies installed"
echo ""

# Step 2: Build the project
echo "▶ Step 2: Building PiWifi..."
if ! command -v cargo &> /dev/null; then
  echo "  ⚠ Rust not found, installing..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source $HOME/.cargo/env
fi

cd "$(dirname "$0")"
cargo build --release
echo "  ✓ Build complete"
echo ""

# Step 3: Install binary
echo "▶ Step 3: Installing binary..."
cp target/release/piwifi /usr/local/bin/
chmod +x /usr/local/bin/piwifi
echo "  ✓ Binary installed to /usr/local/bin/piwifi"
echo ""

# Step 4: Create required directories
echo "▶ Step 4: Creating directories..."
mkdir -p /etc/iptables
mkdir -p /var/log
chmod 755 /etc/iptables
echo "  ✓ Directories created"
echo ""

# Step 5: Install systemd service
echo "▶ Step 5: Installing systemd service..."
cp piwifi.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable piwifi
echo "  ✓ Service installed"
echo ""

# Step 6: Display next steps
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║          Installation Complete!                          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Next Steps:"
echo ""
echo "  1. Start the service:"
echo "     sudo systemctl start piwifi"
echo ""
echo "  2. Check status:"
echo "     sudo systemctl status piwifi"
echo ""
echo "  3. View logs:"
echo "     sudo journalctl -u piwifi -f"
echo ""
echo "  4. Connect to your main WiFi:"
echo "     sudo nmcli device wifi connect 'YOUR_SSID' password 'YOUR_PASSWORD' ifname wlan0"
echo ""
echo "  5. Test connectivity (from connected device):"
echo "     ping 192.168.100.1"
echo "     nslookup google.com"
echo ""
echo "🔗 Configuration Files:"
echo "  - DHCP/DNS:  /etc/dnsmasq.d/piwifi.conf"
echo "  - Firewall:  /etc/iptables/rules.v4"
echo "  - Service:   /etc/systemd/system/piwifi.service"
echo ""
echo "📚 Documentation:"
echo "  - Setup:     ./SETUP.md"
echo "  - Usage:     sudo /usr/local/bin/piwifi"
echo ""
