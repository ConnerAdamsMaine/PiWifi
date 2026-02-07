use piwifi::{WifiManager, NetworkManager, FirewallManager};
use piwifi::network::NetworkConfig;
use std::env;

#[tokio::main]
async fn main() {
    // Check if running web server or CLI setup
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--web".to_string()) || args.contains(&"-w".to_string()) {
        tracing_subscriber::fmt::init();
        let port = args
            .iter()
            .position(|x| x == "--port" || x == "-p")
            .and_then(|i| args.get(i + 1))
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        
        if let Err(e) = piwifi::server::start_server(port).await {
            eprintln!("Server error: {}", e);
        }
        return;
    }
    tracing_subscriber::fmt::init();

    println!("╔═══════════════════════════════════════╗");
    println!("║  PiWifi - Raspberry Pi WiFi Router    ║");
    println!("║  Full Network Setup with NAT/DNS      ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Load default config
    let config = NetworkConfig::default();
    
    // Example: Configure for edge router deployment
    // Uncomment to use edge router mode with DHCP Option 60/61
    /*
    config.dhcp_option_60 = Some("PiWifi-EdgeRouter-Corporate".to_string());
    config.dhcp_option_61 = Some("00:1a:2b:3c:4d:5e".to_string());
    config.nat_enabled = false; // Bridge mode (no NAT)
    */
    
    println!("📋 Configuration:");
    println!("  ETH0 IP:     {}", config.eth_ip);
    println!("  DHCP Range:  {} - {}", config.dhcp_start, config.dhcp_end);
    println!("  DNS Domain:  {}", config.dns_domain);
    println!("  Upstream DNS: {}\n", config.dns_upstream.join(", "));

    // Step 1: Configure Ethernet interface
    println!("▶ Step 1: Configure Ethernet Interface (eth0)");
    match NetworkManager::configure_eth(&config) {
        Ok(_) => println!("  ✓ Ethernet configured with IP {}/{}\n", config.eth_ip, config.eth_cidr),
        Err(e) => {
            println!("  ✗ Error: {}\n", e);
            return;
        }
    }

    // Step 2: Initialize firewall with secure defaults
    println!("▶ Step 2: Initialize Firewall");
    match FirewallManager::init() {
        Ok(_) => println!("  ✓ Firewall initialized with secure defaults"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!("  - Default policy: DROP (secure)");
    println!("  - SSH (22), HTTP (80), HTTPS (443) allowed on eth0");
    println!("  - DHCP (67-68) and DNS (53) enabled");
    println!("  - WiFi → Ethernet forwarding enabled\n");

    // Step 3: Enable NAT
    println!("▶ Step 3: Enable NAT & IP Forwarding");
    match NetworkManager::enable_nat(&config) {
        Ok(_) => println!("  ✓ NAT configured"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!("  - WiFi traffic masqueraded over Ethernet");
    println!("  - IP forwarding enabled\n");

    // Step 4: Start DHCP/DNS server
    println!("▶ Step 4: Start DHCP & DNS Server (dnsmasq)");
    match NetworkManager::start_dhcp(&config) {
        Ok(_) => println!("  ✓ dnsmasq configured and started"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!("  - DHCP range: {} - {}", config.dhcp_start, config.dhcp_end);
    println!("  - DNS forwarding to {}", config.dns_upstream.join(", "));
    println!("  - Local domain: {}\n", config.dns_domain);

    // Step 5: Save firewall rules
    println!("▶ Step 5: Persist Firewall Rules");
    match FirewallManager::save_rules() {
        Ok(_) => println!("  ✓ Rules saved to /etc/iptables/rules.v4\n"),
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    // Step 6: Scan WiFi networks
    println!("▶ Step 6: Scan Available WiFi Networks");
    match WifiManager::scan() {
        Ok(networks) => {
            if networks.is_empty() {
                println!("  ⚠ No networks found (check WiFi adapter)");
            } else {
                println!("  ✓ Found {} networks:", networks.len());
                for net in networks.iter().take(10) {
                    println!("    - {} (Signal: {})", net.ssid, net.signal);
                }
            }
        }
        Err(e) => println!("  ✗ Error scanning: {}", e),
    }
    println!();

    // Step 7: Network status
    println!("▶ Step 7: Network Status");
    match NetworkManager::status() {
        Ok(status) => println!("{}", status),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    // Display current firewall rules
    println!("▶ Current Firewall Rules (INPUT chain):");
    match FirewallManager::show_rules() {
        Ok(rules) => {
            for line in rules.lines().take(15) {
                println!("  {}", line);
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }
    println!();

    println!("╔═══════════════════════════════════════╗");
    println!("║  Setup Complete!                      ║");
    println!("║                                       ║");
    println!("║  Next steps:                          ║");
    println!("║  1. Connect to your main WiFi        ║");
    println!("║  2. Connect devices to Ethernet      ║");
    println!("║  3. Monitor with: tail -f /var/log/dnsmasq.log");
    println!("║  4. Restart with: sudo systemctl restart dnsmasq");
    println!("╚═══════════════════════════════════════╝");
}
