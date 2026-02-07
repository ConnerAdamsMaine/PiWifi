use crate::auth::{AuthManager, LoginRequest};
use crate::network::NetworkConfig;
use crate::{FirewallManager, NetworkManager, WifiManager};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

// ==================== Auth Endpoints ====================

pub async fn login(
    auth: web::Data<AuthManager>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    // Hardcoded for now - replace with actual user store
    if req.username != "admin" || req.password != "piwifi" {
        return HttpResponse::Unauthorized().json(ApiResponse::<String>::err(
            "Invalid credentials".to_string(),
        ));
    }

    match auth.create_token(&req.username, "admin") {
        Ok(token) => HttpResponse::Ok().json(ApiResponse::ok(token)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Token creation failed: {}", e))),
    }
}

pub async fn verify_token(req: HttpRequest, auth: web::Data<AuthManager>) -> impl Responder {
    match extract_token(&req) {
        Some(token) => match auth.verify_token(&token) {
            Ok(claims) => HttpResponse::Ok().json(ApiResponse::ok(claims)),
            Err(_) => HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::err("Invalid token".to_string())),
        },
        None => HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Missing token".to_string())),
    }
}

// ==================== WiFi Endpoints ====================

pub async fn scan_wifi() -> impl Responder {
    match WifiManager::scan() {
        Ok(networks) => HttpResponse::Ok().json(ApiResponse::ok(networks)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<Vec<String>>::err(format!("Scan failed: {}", e))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ConnectWiFiRequest {
    pub ssid: String,
    pub password: String,
}

pub async fn connect_wifi(
    req: web::Json<ConnectWiFiRequest>,
    _http_req: HttpRequest,
    wifi_creds: web::Data<std::sync::Arc<tokio::sync::Mutex<Option<(String, String)>>>>,
) -> impl Responder {
    // Verify token
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    match WifiManager::connect(&req.ssid, &req.password) {
        Ok(_) => {
            // Store credentials for auto-reconnect
            let creds = (req.ssid.clone(), req.password.clone());
            let mut credentials = wifi_creds.lock().await;
            *credentials = Some(creds);
            drop(credentials);
            tracing::info!("WiFi credentials stored for auto-reconnect: {}", req.ssid);
            
            HttpResponse::Ok().json(ApiResponse::ok(json!({"message": "Connecting..."})))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Connection failed: {}", e))),
    }
}

pub async fn get_wifi_status() -> impl Responder {
    match WifiManager::status() {
        Ok(status) => HttpResponse::Ok().json(ApiResponse::ok(status)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Status check failed: {}", e))),
    }
}

pub async fn disconnect_wifi(_http_req: HttpRequest) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    match WifiManager::disconnect() {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok(json!({"message": "Disconnected"}))),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Disconnect failed: {}", e))),
    }
}

// ==================== Network Endpoints ====================

pub async fn get_network_status() -> impl Responder {
    match NetworkManager::status() {
        Ok(status) => HttpResponse::Ok().json(ApiResponse::ok(status)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Network status failed: {}", e))),
    }
}

pub async fn configure_network(
    config: web::Json<NetworkConfig>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    match NetworkManager::configure_eth(&config) {
        Ok(_) => {
            // Log DHCP options if configured
            if let Some(opt60) = &config.dhcp_option_60 {
                tracing::info!("DHCP Option 60 (Vendor Class): {}", opt60);
            }
            if let Some(opt61) = &config.dhcp_option_61 {
                tracing::info!("DHCP Option 61 (Client ID): {}", opt61);
            }
            HttpResponse::Ok().json(ApiResponse::ok(json!({"message": "Configured"})))
        },
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Configuration failed: {}", e))),
    }
}

// ==================== Firewall Endpoints ====================

pub async fn get_firewall_rules() -> impl Responder {
    match FirewallManager::show_rules() {
        Ok(rules) => HttpResponse::Ok().json(ApiResponse::ok(rules)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Failed to get rules: {}", e))),
    }
}

#[derive(Debug, Deserialize)]
pub struct FirewallRuleRequest {
    pub action: String, // "allow", "block", "forward"
    pub interface: String,
    pub protocol: String,
    pub port: u16,
    pub target_ip: Option<String>,
    pub target_port: Option<u16>,
}

pub async fn apply_firewall_rule(
    rule: web::Json<FirewallRuleRequest>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    let result = match rule.action.as_str() {
        "allow" => FirewallManager::allow_port(&rule.interface, &rule.protocol, rule.port),
        "block" => FirewallManager::block_port(&rule.interface, &rule.protocol, rule.port),
        "forward" => {
            if let (Some(target_ip), Some(target_port)) = (&rule.target_ip, rule.target_port) {
                FirewallManager::port_forward(rule.port, target_ip, target_port, &rule.protocol)
            } else {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<String>::err("Missing target IP/port".to_string()));
            }
        }
        _ => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<String>::err("Unknown action".to_string()))
        }
    };

    match result {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok(json!({"message": "Rule applied"}))),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Rule application failed: {}", e))),
    }
}

pub async fn save_firewall_rules(_http_req: HttpRequest) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    match FirewallManager::save_rules() {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok(json!({"message": "Rules saved"}))),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::err(format!("Save failed: {}", e))),
    }
}

// ==================== System Endpoints ====================

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub uptime: String,
    pub cpu_temp: f32,
    pub ram_usage: u32,
    pub disk_usage: u32,
}

#[derive(Debug, Serialize)]
pub struct PingResult {
    pub success: bool,
    pub output: String,
    pub rtt_ms: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct DnsResult {
    pub success: bool,
    pub records: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RouteResult {
    pub success: bool,
    pub hops: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub status: String,
    pub addresses: Vec<String>,
    pub mac: String,
}

#[derive(Debug, Serialize)]
pub struct InterfacesResult {
    pub success: bool,
    pub interfaces: HashMap<String, InterfaceInfo>,
}

fn get_real_system_status() -> Result<SystemStatus> {
    let uptime = get_uptime()?;
    let cpu_temp = get_cpu_temp()?;
    let ram_usage = get_ram_usage()?;
    let disk_usage = get_disk_usage()?;

    Ok(SystemStatus {
        uptime,
        cpu_temp,
        ram_usage,
        disk_usage,
    })
}

fn get_uptime() -> Result<String> {
    let uptime_str = std::fs::read_to_string("/proc/uptime")?;
    let seconds: u64 = uptime_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    Ok(format!("{}d {}h {}m", days, hours, minutes))
}

fn get_cpu_temp() -> Result<f32> {
    // Try reading from thermal zone
    if let Ok(content) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(millidegrees) = content.trim().parse::<f32>() {
            return Ok(millidegrees / 1000.0);
        }
    }

    // Fallback to sensors command
    let output = Command::new("sensors")
        .output()
        .map_err(|_| anyhow::anyhow!("sensors command failed"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("Core") || line.contains("Package") {
            if let Some(temp_str) = line.split('+').nth(1) {
                if let Some(temp_part) = temp_str.split('°').next() {
                    if let Ok(temp) = temp_part.trim().parse::<f32>() {
                        return Ok(temp);
                    }
                }
            }
        }
    }

    Ok(0.0)
}

fn get_ram_usage() -> Result<u32> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(val) = line.split_whitespace().nth(1) {
                mem_total = val.parse().unwrap_or(0);
            }
        }
        if line.starts_with("MemAvailable:") {
            if let Some(val) = line.split_whitespace().nth(1) {
                mem_available = val.parse().unwrap_or(0);
            }
        }
    }

    if mem_total == 0 {
        return Ok(0);
    }

    let usage_percent = ((mem_total - mem_available) * 100) / mem_total;
    Ok(usage_percent as u32)
}

fn get_disk_usage() -> Result<u32> {
    let output = Command::new("df")
        .args(&["-B1", "/"])
        .output()
        .map_err(|_| anyhow::anyhow!("df command failed"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return Ok(0);
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 5 {
        return Ok(0);
    }

    let total: u64 = parts[1].parse().unwrap_or(0);
    let used: u64 = parts[2].parse().unwrap_or(0);

    if total == 0 {
        return Ok(0);
    }

    let usage_percent = (used * 100) / total;
    Ok(usage_percent as u32)
}

pub async fn get_system_status() -> impl Responder {
    match get_real_system_status() {
        Ok(status) => HttpResponse::Ok().json(ApiResponse::ok(status)),
        Err(e) => {
            tracing::warn!("Failed to get system status: {}", e);
            // Fallback to default values if real monitoring fails
            let status = SystemStatus {
                uptime: "Unknown".to_string(),
                cpu_temp: 0.0,
                ram_usage: 0,
                disk_usage: 0,
            };
            HttpResponse::Ok().json(ApiResponse::ok(status))
        }
    }
}

fn sanitize_hostname(input: &str) -> Result<String> {
    if input
        .chars()
        .any(|c| matches!(c, ';' | '|' | '&' | '$' | '`' | '\'' | '"' | '<' | '>' | '\n'))
    {
        return Err(anyhow::anyhow!(
            "Invalid hostname: contains shell metacharacters"
        ));
    }
    if input.is_empty() || input.len() > 255 {
        return Err(anyhow::anyhow!("Invalid hostname: length out of range"));
    }
    Ok(input.to_string())
}

pub async fn diagnostic_ping(path: web::Path<String>) -> impl Responder {
    let host = path.into_inner();
    tracing::debug!("Ping diagnostic for host: {}", host);

    let host = match sanitize_hostname(&host) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::BadRequest().json(ApiResponse::<PingResult>::err(format!(
                "Invalid hostname: {}",
                e
            )))
        }
    };

    let result = match run_ping_command(&host).await {
        Ok((success, output, rtt)) => PingResult {
            success,
            output,
            rtt_ms: rtt,
        },
        Err(_e) => PingResult {
            success: false,
            output: String::new(),
            rtt_ms: None,
        },
    };

    HttpResponse::Ok().json(ApiResponse::ok(result))
}

async fn run_ping_command(host: &str) -> Result<(bool, String, Option<f32>)> {
    let output = tokio::task::spawn_blocking({
        let host = host.to_string();
        move || {
            Command::new("ping")
                .args(&["-c", "4", "-W", "5", &host])
                .output()
        }
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let success = output.status.success();

    // Try to extract RTT from ping output
    let rtt_ms = extract_rtt_from_ping(&stdout);

    Ok((success, stdout, rtt_ms))
}

fn extract_rtt_from_ping(output: &str) -> Option<f32> {
    for line in output.lines() {
        if line.contains("min/avg/max") || line.contains("round-trip") {
            // Parse "min/avg/max/stddev = 1.234/5.678/9.012/1.234 ms"
            if let Some(avg_part) = line.split('/').nth(1) {
                if let Ok(avg) = avg_part.trim().parse::<f32>() {
                    return Some(avg);
                }
            }
        }
    }
    None
}

pub async fn diagnostic_dns(path: web::Path<String>) -> impl Responder {
    let domain = path.into_inner();
    tracing::debug!("DNS diagnostic for domain: {}", domain);

    let domain = match sanitize_hostname(&domain) {
        Ok(d) => d,
        Err(e) => {
            return HttpResponse::BadRequest().json(ApiResponse::<DnsResult>::err(format!(
                "Invalid domain: {}",
                e
            )))
        }
    };

    let result = match run_dns_command(&domain).await {
        Ok((success, records, error)) => DnsResult {
            success,
            records,
            error,
        },
        Err(e) => DnsResult {
            success: false,
            records: Vec::new(),
            error: Some(e.to_string()),
        },
    };

    HttpResponse::Ok().json(ApiResponse::ok(result))
}

async fn run_dns_command(domain: &str) -> Result<(bool, Vec<String>, Option<String>)> {
    let output = tokio::task::spawn_blocking({
        let domain = domain.to_string();
        move || {
            Command::new("dig")
                .args(&["+short", &domain, "@8.8.8.8"])
                .output()
        }
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    let records: Vec<String> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    let error = if !success || records.is_empty() {
        Some(stderr.trim().to_string())
    } else {
        None
    };

    Ok((success, records, error))
}

pub async fn diagnostic_route(path: web::Path<String>) -> impl Responder {
    let host = path.into_inner();
    tracing::debug!("Route diagnostic for host: {}", host);

    let host = match sanitize_hostname(&host) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::BadRequest().json(ApiResponse::<RouteResult>::err(format!(
                "Invalid hostname: {}",
                e
            )))
        }
    };

    let result = match run_traceroute_command(&host).await {
        Ok((success, hops, error)) => RouteResult {
            success,
            hops,
            error,
        },
        Err(e) => RouteResult {
            success: false,
            hops: Vec::new(),
            error: Some(e.to_string()),
        },
    };

    HttpResponse::Ok().json(ApiResponse::ok(result))
}

async fn run_traceroute_command(host: &str) -> Result<(bool, Vec<String>, Option<String>)> {
    let output = tokio::task::spawn_blocking({
        let host = host.to_string();
        move || {
            // Try traceroute first, fallback to tracert on Windows
            if let Ok(result) = Command::new("traceroute")
                .args(&["-m", "10", &host])
                .output()
            {
                Ok::<_, anyhow::Error>(result)
            } else {
                Command::new("tracert")
                    .args(&["-h", "10", &host])
                    .output()
                    .map_err(|e| anyhow::anyhow!(e))
            }
        }
    })
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    let hops: Vec<String> = stdout
        .lines()
        .filter(|l| {
            !l.trim().is_empty()
                && !l.contains("traceroute")
                && !l.contains("Traceroute")
                && !l.contains("to")
        })
        .map(|l| l.to_string())
        .collect();

    let error = if !success {
        Some(stderr.trim().to_string())
    } else {
        None
    };

    Ok((success, hops, error))
}

pub async fn diagnostic_interfaces() -> impl Responder {
    tracing::debug!("Interface diagnostic");

    match run_ip_addr_command().await {
        Ok(interfaces) => {
            let result = InterfacesResult {
                success: true,
                interfaces,
            };
            HttpResponse::Ok().json(ApiResponse::ok(result))
        }
        Err(e) => {
            tracing::error!("Failed to get interfaces: {}", e);
            let result = InterfacesResult {
                success: false,
                interfaces: HashMap::new(),
            };
            HttpResponse::Ok().json(ApiResponse::ok(result))
        }
    }
}

async fn run_ip_addr_command() -> Result<HashMap<String, InterfaceInfo>> {
    let output = tokio::task::spawn_blocking(|| Command::new("ip").arg("addr").arg("show").output())
        .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces: HashMap<String, InterfaceInfo> = HashMap::new();
    let mut current_interface: Option<String> = None;
    let mut current_info: Option<InterfaceInfo> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Parse interface lines (e.g., "1: lo: <LOOPBACK,UP,LOWER_UP>")
        if let Some(_colon_pos) = trimmed.find(':') {
            if trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                if let Some(info) = current_info.take() {
                    if let Some(name) = current_interface.take() {
                        interfaces.insert(name, info);
                    }
                }

                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 3 {
                    let name = parts[1].trim().to_string();
                    let flags = parts[2].trim();
                    let status = if flags.contains("UP") {
                        "UP".to_string()
                    } else {
                        "DOWN".to_string()
                    };

                    current_interface = Some(name);
                    current_info = Some(InterfaceInfo {
                        name: current_interface.clone().unwrap_or_default(),
                        status,
                        addresses: Vec::new(),
                        mac: String::new(),
                    });
                }
            }
        }

        // Parse address lines
        if trimmed.starts_with("inet ") || trimmed.starts_with("inet6 ") {
            if let Some(addr) = trimmed.split_whitespace().nth(1) {
                if let Some(info) = current_info.as_mut() {
                    info.addresses.push(addr.to_string());
                }
            }
        }

        // Parse MAC address lines
        if trimmed.starts_with("link/ether ") {
            if let Some(mac) = trimmed.split_whitespace().nth(1) {
                if let Some(info) = current_info.as_mut() {
                    info.mac = mac.to_string();
                }
            }
        }
    }

    // Don't forget the last interface
    if let Some(info) = current_info {
        if let Some(name) = current_interface {
            interfaces.insert(name, info);
        }
    }

    Ok(interfaces)
}

// ==================== DHCP Endpoints ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct DHCPConfig {
    pub dhcp_start: String,
    pub dhcp_end: String,
    pub lease_time: u32,  // in seconds
    pub dns_servers: Vec<String>,
    pub local_domain: String,
}

#[derive(Debug, Serialize)]
pub struct DHCPStatus {
    pub enabled: bool,
    pub config: DHCPConfig,
    pub active_leases: usize,
}

pub async fn get_dhcp_config() -> impl Responder {
    match read_dhcp_config() {
        Ok(status) => {
            tracing::debug!("DHCP config retrieved successfully");
            HttpResponse::Ok().json(ApiResponse::ok(status))
        }
        Err(e) => {
            tracing::error!("Failed to read DHCP config: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<DHCPStatus>::err(format!("Failed to read config: {}", e)))
        }
    }
}

pub async fn set_dhcp_config(
    config: web::Json<DHCPConfig>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    // Validate input
    if config.dhcp_start >= config.dhcp_end {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("DHCP start must be less than end".to_string()));
    }

    if config.lease_time == 0 {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("Lease time must be greater than 0".to_string()));
    }

    if config.dns_servers.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("At least one DNS server is required".to_string()));
    }

    match write_dhcp_config(&config) {
        Ok(_) => {
            tracing::info!(
                "DHCP config updated: range {}-{}, lease {}s, DNS {:?}",
                config.dhcp_start,
                config.dhcp_end,
                config.lease_time,
                config.dns_servers
            );
            HttpResponse::Ok().json(ApiResponse::ok("Configuration saved and applied".to_string()))
        }
        Err(e) => {
            tracing::error!("Failed to set DHCP config: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::err(format!("Failed to set config: {}", e)))
        }
    }
}

pub async fn restart_dnsmasq(_http_req: HttpRequest) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    match restart_dnsmasq_service() {
        Ok(_) => {
            tracing::info!("dnsmasq service restarted successfully");
            HttpResponse::Ok().json(ApiResponse::ok("dnsmasq restarted".to_string()))
        }
        Err(e) => {
            tracing::error!("Failed to restart dnsmasq: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::err(format!("Failed to restart dnsmasq: {}", e)))
        }
    }
}

// ==================== Log Viewer Endpoints ====================

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    #[serde(default = "default_lines")]
    pub lines: usize,
    pub filter: Option<String>,
}

fn default_lines() -> usize {
    100
}

fn parse_log_line(line: &str, filter: Option<&str>) -> Option<LogEntry> {
    // Parse journalctl format: "timestamp HOSTNAME process[pid]: message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 4 {
        return None;
    }

    let timestamp = parts[0].to_string();
    let message = parts[3].to_string();

    // Extract log level from message keywords
    let level = if message.contains("ERROR") || message.contains("error") {
        "error".to_string()
    } else if message.contains("WARN") || message.contains("warning") {
        "warn".to_string()
    } else if message.contains("DEBUG") {
        "debug".to_string()
    } else if message.contains("INFO") || message.contains("info") {
        "info".to_string()
    } else {
        "info".to_string()
    };

    // Apply filter if provided
    if let Some(filter_text) = filter {
        if !message.to_lowercase().contains(&filter_text.to_lowercase()) {
            return None;
        }
    }

    Some(LogEntry {
        timestamp,
        level,
        message,
    })
}

pub async fn get_system_logs(query: web::Query<LogQuery>) -> impl Responder {
    tracing::debug!("Fetching system logs: lines={}, filter={:?}", query.lines, query.filter);

    // Clamp lines to reasonable range (default 100, max 500)
    let lines = std::cmp::min(std::cmp::max(query.lines, 1), 500);

    match Command::new("journalctl")
        .arg("-n")
        .arg(lines.to_string())
        .arg("--no-pager")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                tracing::warn!("journalctl command failed");
                return HttpResponse::Ok().json(ApiResponse::ok(Vec::<LogEntry>::new()));
            }

            let logs = String::from_utf8_lossy(&output.stdout);
            let mut entries: Vec<LogEntry> = logs
                .lines()
                .filter_map(|line| parse_log_line(line, query.filter.as_deref()))
                .collect();

            // Reverse to show newest first
            entries.reverse();

            tracing::debug!("Retrieved {} log entries", entries.len());
            HttpResponse::Ok().json(ApiResponse::ok(entries))
        }
        Err(e) => {
            tracing::error!("Failed to execute journalctl: {}", e);
            HttpResponse::Ok().json(ApiResponse::ok(Vec::<LogEntry>::new()))
        }
    }
}

pub async fn get_dnsmasq_logs(query: web::Query<LogQuery>) -> impl Responder {
    tracing::debug!("Fetching dnsmasq logs: lines={}", query.lines);

    // Clamp lines to reasonable range
    let lines = std::cmp::min(std::cmp::max(query.lines, 1), 500);

    match std::fs::read_to_string("/var/log/dnsmasq.log") {
        Ok(content) => {
            let mut entries: Vec<LogEntry> = content
                .lines()
                .rev() // Get newest first
                .take(lines)
                .filter_map(|line| {
                    // Parse dnsmasq log format
                    let parts: Vec<&str> = line.splitn(4, ' ').collect();
                    if parts.len() < 4 {
                        return None;
                    }

                    let timestamp = parts[0].to_string();
                    let message = parts[3..].join(" ");

                    // Extract log level from dnsmasq message patterns
                    let level = if message.contains("DHCP") {
                        "info".to_string()
                    } else if message.contains("error") || message.contains("ERROR") {
                        "error".to_string()
                    } else if message.contains("query") || message.contains("reply") {
                        "debug".to_string()
                    } else {
                        "info".to_string()
                    };

                    Some(LogEntry {
                        timestamp,
                        level,
                        message,
                    })
                })
                .collect();

            // Already in reverse order, just reverse again to get newest first consistently
            entries.reverse();
            tracing::debug!("Retrieved {} dnsmasq log entries", entries.len());
            HttpResponse::Ok().json(ApiResponse::ok(entries))
        }
        Err(e) => {
            tracing::debug!("Failed to read dnsmasq log: {}", e);
            // Return empty logs on file not found or read error
            HttpResponse::Ok().json(ApiResponse::ok(Vec::<LogEntry>::new()))
        }
    }
}

// ==================== Connection History Endpoints ====================

#[derive(Debug, Serialize)]
pub struct FavoriteNetwork {
    pub ssid: String,
    pub connection_count: usize,
    pub success_rate: f64,
}

pub async fn get_connection_history(
    history: web::Data<crate::history::HistoryManager>,
) -> impl Responder {
    let entries = history.get_all().await;
    tracing::debug!("Retrieved {} connection history entries", entries.len());
    HttpResponse::Ok().json(ApiResponse::ok(entries))
}

pub async fn get_wifi_favorites(
    history: web::Data<crate::history::HistoryManager>,
) -> impl Responder {
    let favorites = history.get_favorites(10).await;
    
    let mut result = Vec::new();
    for (ssid, count) in favorites {
        let success_rate = history
            .get_success_rate(&ssid)
            .await
            .unwrap_or(0.0);
        
        result.push(FavoriteNetwork {
            ssid,
            connection_count: count,
            success_rate,
        });
    }
    
    tracing::debug!("Retrieved {} favorite networks", result.len());
    HttpResponse::Ok().json(ApiResponse::ok(result))
}

pub async fn clear_history(
    _http_req: HttpRequest,
    history: web::Data<crate::history::HistoryManager>,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }
    
    history.clear().await;
    tracing::info!("Connection history cleared by user");
    HttpResponse::Ok().json(ApiResponse::ok("History cleared".to_string()))
}

// ==================== Connected Clients Endpoints ====================

#[derive(Debug, Serialize)]
pub struct Client {
    pub hostname: String,
    pub mac: String,
    pub ip: String,
    pub lease_expires: String,
}

fn parse_dnsmasq_leases() -> Result<Vec<(String, String, String)>> {
    match std::fs::read_to_string("/var/lib/dnsmasq/dnsmasq.leases") {
        Ok(content) => {
            let mut clients = Vec::new();
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    // Format: timestamp MAC IP hostname duration
                    let _timestamp = parts[0];
                    let mac = parts[1].to_string();
                    let ip = parts[2].to_string();
                    let hostname = parts[3].to_string();
                    clients.push((ip, mac, hostname));
                }
            }
            Ok(clients)
        }
        Err(e) => {
            tracing::warn!("Failed to read dnsmasq leases: {}", e);
            Ok(Vec::new())
        }
    }
}

pub async fn get_connected_clients() -> impl Responder {
    match parse_dnsmasq_leases() {
        Ok(leases) => {
            let mut clients = Vec::new();
            for (ip, mac, hostname) in leases {
                let lease_expires = format!("Active");
                
                clients.push(Client {
                    hostname,
                    mac,
                    ip,
                    lease_expires,
                });
            }
            
            tracing::debug!("Retrieved {} connected clients", clients.len());
            HttpResponse::Ok().json(ApiResponse::ok(clients))
        }
        Err(e) => {
            tracing::error!("Failed to get connected clients: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Vec<Client>>::err(
                    "Failed to retrieve client list".to_string(),
                ))
        }
    }
}

// ==================== Config Backup/Restore Endpoints ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigBackup {
    pub wifi_history: Vec<crate::history::ConnectionEntry>,
    pub dhcp_config: DHCPConfig,
    pub network_config: NetworkConfig,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

pub async fn backup_config(
    _http_req: HttpRequest,
    history: web::Data<crate::history::HistoryManager>,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }
    
    // Get WiFi history
    let wifi_history = history.get_all().await;
    
    // Get DHCP config
    let dhcp_config = match read_dhcp_config() {
        Ok(status) => status.config,
        Err(e) => {
            tracing::error!("Failed to read DHCP config for backup: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::err("Failed to read DHCP config".to_string()));
        }
    };
    
    // Get network config (use defaults for now)
    let network_config = NetworkConfig::default();
    
    let backup = ConfigBackup {
        wifi_history,
        dhcp_config,
        network_config,
        timestamp: chrono::Utc::now(),
        version: "1.0".to_string(),
    };
    
    match serde_json::to_string(&backup) {
        Ok(json_str) => {
            tracing::info!("Configuration backup created");
            HttpResponse::Ok().json(ApiResponse::ok(json_str))
        }
        Err(e) => {
            tracing::error!("Failed to serialize backup: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::err("Failed to create backup".to_string()))
        }
    }
}

pub async fn restore_config(
    backup: web::Json<serde_json::Value>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }
    
    // Validate backup has required fields
    if !backup.is_object() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("Invalid backup format".to_string()));
    }
    
    let obj = backup.as_object().unwrap();
    if !obj.contains_key("wifi_history") 
        || !obj.contains_key("dhcp_config") 
        || !obj.contains_key("network_config") {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err(
                "Missing required backup fields".to_string(),
            ));
    }
    
    // Extract and restore DHCP config
    if let Some(dhcp_value) = obj.get("dhcp_config") {
        match serde_json::from_value::<DHCPConfig>(dhcp_value.clone()) {
            Ok(dhcp_config) => {
                if let Err(e) = write_dhcp_config(&dhcp_config) {
                    tracing::warn!("Failed to restore DHCP config: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<String>::err(
                            "Failed to restore DHCP config".to_string(),
                        ));
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse DHCP config from backup: {}", e);
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<String>::err("Invalid DHCP config in backup".to_string()));
            }
        }
    }
    
    tracing::warn!("Configuration restored from backup");
    HttpResponse::Ok().json(ApiResponse::ok("Configuration restored".to_string()))
}

// ==================== Speed Test Endpoint ====================

#[derive(Debug, Serialize)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

pub async fn speedtest_run() -> impl Responder {
    tracing::info!("Starting speed test...");
    
    // Try speedtest-cli first, then fallback to speedtest
    let output = match run_speedtest_cli() {
        Ok(out) => out,
        Err(_) => {
            tracing::warn!("speedtest-cli not available, trying speedtest");
            match run_speedtest() {
                Ok(out) => out,
                Err(e) => {
                    tracing::error!("Speed test failed: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<SpeedTestResult>::err(
                            "speedtest-cli not installed or execution failed".to_string(),
                        ));
                }
            }
        }
    };

    match parse_speedtest_output(&output) {
        Ok((download, upload, ping)) => {
            let result = SpeedTestResult {
                download_mbps: download,
                upload_mbps: upload,
                ping_ms: ping,
                timestamp: chrono::Utc::now(),
                status: "success".to_string(),
            };
            tracing::info!(
                "Speed test complete: {} Mbps down, {} Mbps up, {} ms ping",
                download,
                upload,
                ping
            );
            HttpResponse::Ok().json(ApiResponse::ok(result))
        }
        Err(e) => {
            tracing::error!("Failed to parse speedtest output: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<SpeedTestResult>::err(
                    "Failed to parse speed test results".to_string(),
                ))
        }
    }
}

// ==================== Wake-on-LAN Endpoint ====================

pub async fn wake_device(
    mac: web::Path<String>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    let mac_addr = mac.into_inner();
    
    // Format and validate MAC
    match format_mac(&mac_addr) {
        Ok(formatted_mac) => {
            match create_magic_packet(&formatted_mac) {
                Ok(packet) => {
                    match send_magic_packet(&packet) {
                        Ok(_) => {
                            tracing::info!("Magic packet sent to {}", formatted_mac);
                            HttpResponse::Ok().json(ApiResponse::ok(
                                format!("Magic packet sent to {}", formatted_mac),
                            ))
                        }
                        Err(e) => {
                            tracing::error!("Failed to send magic packet: {}", e);
                            HttpResponse::InternalServerError()
                                .json(ApiResponse::<String>::err("Failed to send packet".to_string()))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create magic packet: {}", e);
                    HttpResponse::BadRequest()
                        .json(ApiResponse::<String>::err(format!("Invalid MAC address: {}", e)))
                }
            }
        }
        Err(e) => {
            tracing::warn!("Invalid MAC format: {}", e);
            HttpResponse::BadRequest()
                .json(ApiResponse::<String>::err(format!("Invalid MAC format: {}", e)))
        }
    }
}

// ==================== Device Management Endpoints ====================

#[derive(Debug, Serialize)]
pub struct Device {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub alias: Option<String>,
    pub vendor: Option<String>,
    pub is_static: bool,
}

pub async fn get_all_devices() -> impl Responder {
    tracing::info!("Fetching all connected devices");
    
    // Parse ARP table to get devices
    match get_arp_devices() {
        Ok(devices) => {
            tracing::debug!("Found {} devices", devices.len());
            HttpResponse::Ok().json(ApiResponse::ok(devices))
        }
        Err(e) => {
            tracing::error!("Failed to get devices: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Vec<Device>>::err(
                    "Failed to retrieve devices".to_string(),
                ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AliasRequest {
    pub alias: String,
}

pub async fn set_device_alias(
    mac: web::Path<String>,
    body: web::Json<AliasRequest>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    let mac_addr = mac.into_inner();
    let alias = &body.alias;

    // Validate alias length and characters
    if alias.is_empty() || alias.len() > 64 {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err(
                "Alias must be 1-64 characters".to_string(),
            ));
    }

    if !alias.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err(
                "Alias can only contain alphanumeric, space, dash, and underscore".to_string(),
            ));
    }

    // TODO: Store alias in device manager persistence
    tracing::info!("Device alias updated: {} => {}", mac_addr, alias);
    
    HttpResponse::Ok().json(ApiResponse::ok("Alias updated".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct StaticIpRequest {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
}

pub async fn set_static_ip(
    body: web::Json<StaticIpRequest>,
    _http_req: HttpRequest,
) -> impl Responder {
    if !verify_auth(&_http_req) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::err("Unauthorized".to_string()));
    }

    // Validate MAC format
    if let Err(e) = format_mac(&body.mac) {
        tracing::warn!("Invalid MAC format: {}", e);
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("Invalid MAC format".to_string()));
    }

    // Validate IP format
    if !validate_ip_format(&body.ip) {
        tracing::warn!("Invalid IP format: {}", body.ip);
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err("Invalid IP format".to_string()));
    }

    // Validate hostname
    if body.hostname.is_empty() || body.hostname.len() > 63 {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::err(
                "Hostname must be 1-63 characters".to_string(),
            ));
    }

    // Write to dnsmasq static hosts config
    match write_static_ip_config(&body.mac, &body.ip, &body.hostname) {
        Ok(_) => {
            // Restart dnsmasq
            if let Err(e) = restart_dnsmasq_service() {
                tracing::error!("Failed to restart dnsmasq: {}", e);
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::err("Failed to restart DHCP service".to_string()));
            }

            tracing::info!(
                "Static IP assigned: {} -> {} ({})",
                body.mac,
                body.ip,
                body.hostname
            );
            HttpResponse::Ok().json(ApiResponse::ok("Static IP assigned".to_string()))
        }
        Err(e) => {
            tracing::error!("Failed to write static IP config: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::err("Failed to assign static IP".to_string()))
        }
    }
}

// ==================== Bandwidth Tracking Endpoint ====================

#[derive(Debug, Serialize)]
pub struct BandwidthStat {
    pub device_mac: String,
    pub device_name: String,
    pub ip: String,
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

pub async fn get_bandwidth_stats() -> impl Responder {
    tracing::info!("Retrieving bandwidth statistics");

    // Get ARP mapping (IP -> MAC)
    let arp_map = get_arp_mapping();

    // Try to get stats from /proc/net/nf_conntrack
    let mut stats_by_ip: HashMap<String, (u64, u64)> = HashMap::new();

    if let Ok(conntrack_data) = parse_nf_conntrack() {
        for (ip, sent, recv) in conntrack_data {
            stats_by_ip
                .entry(ip)
                .and_modify(|(s, r)| {
                    *s += sent;
                    *r += recv;
                })
                .or_insert((sent, recv));
        }
    } else {
        tracing::warn!("Unable to parse /proc/net/nf_conntrack");
    }

    // Get device aliases from ARP
    let devices_by_ip = get_arp_devices_by_ip();

    // Build bandwidth stats
    let mut bandwidth_stats: Vec<BandwidthStat> = stats_by_ip
        .into_iter()
        .map(|(ip, (sent, recv))| {
            let (mac, name) = if let Some((m, n)) = devices_by_ip.get(&ip) {
                (m.clone(), n.clone())
            } else if let Some(mac) = arp_map.get(&ip) {
                (mac.clone(), ip.clone())
            } else {
                (String::from("unknown"), ip.clone())
            };

            BandwidthStat {
                device_mac: mac,
                device_name: name,
                ip: ip.clone(),
                packets_sent: 0,  // Would need additional parsing
                packets_recv: 0,  // Would need additional parsing
                bytes_sent: sent,
                bytes_recv: recv,
            }
        })
        .collect();

    // Sort by total bytes descending
    bandwidth_stats.sort_by(|a, b| {
        (b.bytes_sent + b.bytes_recv).cmp(&(a.bytes_sent + a.bytes_recv))
    });

    // Limit to top 20
    bandwidth_stats.truncate(20);

    tracing::debug!("Retrieved bandwidth stats for {} devices", bandwidth_stats.len());
    HttpResponse::Ok().json(ApiResponse::ok(bandwidth_stats))
}

// ==================== Health Check ====================

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ==================== DHCP Helpers ====================

fn format_lease_time(seconds: u32) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

fn count_dhcp_leases() -> usize {
    match std::fs::read_to_string("/var/lib/dnsmasq/dnsmasq.leases") {
        Ok(content) => content.lines().count(),
        Err(_) => 0,
    }
}

fn parse_lease_time(lease_str: &str) -> Option<u32> {
    let lease_str = lease_str.trim();
    if let Some(last_char) = lease_str.chars().last() {
        let num_part = &lease_str[..lease_str.len() - 1];
        if let Ok(num) = num_part.parse::<u32>() {
            return Some(match last_char {
                's' => num,
                'm' => num * 60,
                'h' => num * 3600,
                'd' => num * 86400,
                _ => return None,
            });
        }
    }
    None
}

fn read_dhcp_config() -> Result<DHCPStatus> {
    let config_path = "/etc/dnsmasq.d/piwifi.conf";
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

    let mut dhcp_start = String::new();
    let mut dhcp_end = String::new();
    let mut lease_time = 3600u32; // default 1 hour
    let mut dns_servers = Vec::new();
    let mut local_domain = String::from("piwifi.local");
    let mut enabled = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(range_part) = line.strip_prefix("dhcp-range=") {
            enabled = true;
            let parts: Vec<&str> = range_part.split(',').collect();
            if parts.len() >= 3 {
                dhcp_start = parts[0].to_string();
                dhcp_end = parts[1].to_string();
                if let Some(lease) = parse_lease_time(parts[2]) {
                    lease_time = lease;
                }
            }
        } else if let Some(server_part) = line.strip_prefix("server=") {
            dns_servers.push(server_part.to_string());
        } else if let Some(domain_part) = line.strip_prefix("local-domain=") {
            local_domain = domain_part.to_string();
        }
    }

    tracing::debug!(
        "Parsed DHCP config: range {}-{}, lease {}s, DNS {:?}",
        dhcp_start,
        dhcp_end,
        lease_time,
        dns_servers
    );

    Ok(DHCPStatus {
        enabled,
        config: DHCPConfig {
            dhcp_start,
            dhcp_end,
            lease_time,
            dns_servers,
            local_domain,
        },
        active_leases: count_dhcp_leases(),
    })
}

fn write_dhcp_config(config: &DHCPConfig) -> Result<()> {
    let lease_str = format_lease_time(config.lease_time);
    let mut content = String::from("# PiWifi DHCP Configuration\n");
    content.push_str(&format!(
        "dhcp-range={},{},{}\n",
        config.dhcp_start, config.dhcp_end, lease_str
    ));

    for dns in &config.dns_servers {
        content.push_str(&format!("server={}\n", dns));
    }

    content.push_str(&format!("local-domain={}\n", config.local_domain));
    content.push_str("# End PiWifi DHCP Configuration\n");

    let config_path = "/etc/dnsmasq.d/piwifi.conf";
    std::fs::write(config_path, &content)
        .map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

    tracing::info!("DHCP config written to {}", config_path);

    // Restart dnsmasq service
    restart_dnsmasq_service()?;

    Ok(())
}

fn restart_dnsmasq_service() -> Result<()> {
    let output = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("dnsmasq")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("dnsmasq restart failed: {}", error_msg));
    }

    tracing::info!("dnsmasq service restarted");
    Ok(())
}

// ==================== Helpers ====================

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?
        .to_string()
        .into()
}

fn verify_auth(req: &HttpRequest) -> bool {
    extract_token(req).is_some()
    // TODO: Actually verify the token with AuthManager
}

// ==================== Speed Test Helpers ====================

fn run_speedtest_cli() -> Result<String> {
    let output = Command::new("timeout")
        .arg("30")
        .arg("speedtest-cli")
        .arg("--simple")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run speedtest-cli: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("speedtest-cli failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_speedtest() -> Result<String> {
    let output = Command::new("timeout")
        .arg("30")
        .arg("speedtest")
        .arg("--simple")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run speedtest: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("speedtest failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_speedtest_output(output: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<&str> = output.trim().split('\n').collect();
    if parts.len() < 1 {
        return Err(anyhow::anyhow!("Invalid speedtest output"));
    }

    let line = parts[0].trim();
    let values: Vec<&str> = line.split_whitespace().collect();

    if values.len() < 3 {
        return Err(anyhow::anyhow!("Expected 3 values in output"));
    }

    let download = values[0]
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Failed to parse download speed"))?;
    let upload = values[1]
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Failed to parse upload speed"))?;
    let ping = values[2]
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Failed to parse ping"))?;

    Ok((download, upload, ping))
}

// ==================== Wake-on-LAN Helpers ====================

fn format_mac(mac: &str) -> Result<String> {
    let mac = mac.to_uppercase();
    
    // Try different formats
    let normalized = if mac.contains(':') {
        // AA:BB:CC:DD:EE:FF format
        mac.replace(':', "")
    } else if mac.contains('-') {
        // AA-BB-CC-DD-EE-FF format
        mac.replace('-', "")
    } else {
        // AABBCCDDEEFF format
        mac.clone()
    };

    // Validate hex and length
    if normalized.len() != 12 {
        return Err(anyhow::anyhow!("MAC address must have 12 hex characters"));
    }

    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow::anyhow!("MAC address must contain only hex characters"));
    }

    // Return as AA:BB:CC:DD:EE:FF format
    let result = format!(
        "{}:{}:{}:{}:{}:{}",
        &normalized[0..2],
        &normalized[2..4],
        &normalized[4..6],
        &normalized[6..8],
        &normalized[8..10],
        &normalized[10..12]
    );

    Ok(result)
}

fn create_magic_packet(mac: &str) -> Result<Vec<u8>> {
    let bytes_str = mac.replace(':', "");
    let mac_bytes: Vec<u8> = (0..bytes_str.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&bytes_str[i..i + 2], 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex in MAC"))
        })
        .collect::<Result<Vec<u8>>>()?;

    if mac_bytes.len() != 6 {
        return Err(anyhow::anyhow!("MAC must be 6 bytes"));
    }

    let mut packet = Vec::with_capacity(102);

    // 6 bytes of 0xFF
    for _ in 0..6 {
        packet.push(0xFF);
    }

    // 16 repeats of MAC address
    for _ in 0..16 {
        packet.extend_from_slice(&mac_bytes);
    }

    Ok(packet)
}

fn send_magic_packet(packet: &[u8]) -> Result<()> {
    use std::net::{UdpSocket, SocketAddr};

    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| anyhow::anyhow!("Failed to bind UDP socket: {}", e))?;
    socket.set_broadcast(true)
        .map_err(|e| anyhow::anyhow!("Failed to set broadcast: {}", e))?;
    
    let broadcast_addr: SocketAddr = "255.255.255.255:9".parse()?;
    socket.send_to(packet, &broadcast_addr)
        .map_err(|e| anyhow::anyhow!("Failed to send magic packet: {}", e))?;

    Ok(())
}

// ==================== Device Management Helpers ====================

fn get_arp_devices() -> Result<Vec<Device>> {
    let output = Command::new("arp")
        .arg("-an")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run arp: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("arp command failed"));
    }

    let mut devices = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        // Parse lines like: ? (192.168.1.100) at aa:bb:cc:dd:ee:ff [ether] on eth0
        if let Some((ip, mac)) = parse_arp_line(line) {
            devices.push(Device {
                mac,
                ip,
                hostname: String::new(),
                alias: None,
                vendor: None,
                is_static: false,
            });
        }
    }

    Ok(devices)
}

fn parse_arp_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    // Extract IP (remove parentheses)
    let ip = parts[1]
        .trim_start_matches('(')
        .trim_end_matches(')')
        .to_string();

    // Extract MAC (6th element, index 3)
    let mac = parts[3].to_string();

    if is_valid_ip(&ip) && is_valid_mac(&mac) {
        Some((ip, mac))
    } else {
        None
    }
}

fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

fn is_valid_mac(mac: &str) -> bool {
    let parts: Vec<&str> = mac.split(':').collect();
    parts.len() == 6 && parts.iter().all(|p| u8::from_str_radix(p, 16).is_ok())
}

fn get_arp_mapping() -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Ok(output) = Command::new("arp").arg("-an").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some((ip, mac)) = parse_arp_line(line) {
                map.insert(ip, mac);
            }
        }
    }

    map
}

fn get_arp_devices_by_ip() -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();

    if let Ok(devices) = get_arp_devices() {
        for device in devices {
            map.insert(device.ip.clone(), (device.mac.clone(), device.hostname.clone()));
        }
    }

    map
}

fn validate_ip_format(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

fn write_static_ip_config(mac: &str, ip: &str, hostname: &str) -> Result<()> {
    let config_path = "/etc/dnsmasq.d/static_hosts.conf";
    
    // Read existing config
    let existing = std::fs::read_to_string(config_path).unwrap_or_default();
    
    // Remove any existing entry for this MAC
    let lines: Vec<&str> = existing.lines().filter(|l| !l.contains(mac)).collect();
    
    // Write updated config
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    
    content.push_str(&format!("dhcp-host={},{},{}\n", mac, ip, hostname));
    
    std::fs::write(config_path, &content)
        .map_err(|e| anyhow::anyhow!("Failed to write static IP config: {}", e))?;

    tracing::debug!(
        "Static IP config written: {} -> {} ({})",
        mac,
        ip,
        hostname
    );

    Ok(())
}

// ==================== Bandwidth Tracking Helpers ====================

fn parse_nf_conntrack() -> Result<Vec<(String, u64, u64)>> {
    let content = std::fs::read_to_string("/proc/net/nf_conntrack")
        .map_err(|e| anyhow::anyhow!("Failed to read /proc/net/nf_conntrack: {}", e))?;

    let mut results = Vec::new();

    for line in content.lines() {
        // Parse lines like:
        // ipv4     2 tcp      6 431999 ESTABLISHED src=192.168.1.100 dst=8.8.8.8 sport=54321 dport=443 ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let mut src_ip = String::new();
        let mut bytes_sent = 0u64;

        for part in &parts {
            if let Some(ip) = part.strip_prefix("src=") {
                src_ip = ip.to_string();
            } else if let Some(bytes) = part.strip_prefix("bytes=") {
                if let Ok(b) = bytes.parse::<u64>() {
                    bytes_sent = b;
                }
            }
        }

        if !src_ip.is_empty() && bytes_sent > 0 {
            results.push((src_ip, bytes_sent, 0));
        }
    }

    Ok(results)
}
