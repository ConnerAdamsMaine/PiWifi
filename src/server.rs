use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::*;
use crate::AuthManager;
use crate::HistoryManager;
use crate::devices::DeviceManager;

pub async fn start_server(port: u16) -> std::io::Result<()> {
    info!("Starting PiWifi web server on 0.0.0.0:{}", port);

    let auth = web::Data::new(AuthManager::new("piwifi-secret-key-change-me".to_string()));
    let history = web::Data::new(HistoryManager::new());
    let devices = web::Data::new(DeviceManager::new());
    
    // Shared WiFi credentials for auto-reconnect
    let wifi_credentials: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let credentials_clone = wifi_credentials.clone();
    
    // Spawn WiFi auto-reconnect monitor in background
    tokio::spawn(async move {
        if let Err(e) = crate::wifi::start_monitor(credentials_clone).await {
            info!("WiFi monitor error: {}", e);
        }
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(auth.clone())
            .app_data(history.clone())
            .app_data(devices.clone())
            .app_data(web::Data::new(wifi_credentials.clone()))
            .wrap(cors)
            // Auth endpoints
            .route("/api/auth/login", web::post().to(login))
            .route("/api/auth/verify", web::get().to(verify_token))
            // WiFi endpoints
            .route("/api/wifi/scan", web::get().to(scan_wifi))
            .route("/api/wifi/status", web::get().to(get_wifi_status))
            .route("/api/wifi/connect", web::post().to(connect_wifi))
            .route("/api/wifi/disconnect", web::post().to(disconnect_wifi))
            .route("/api/wifi/history", web::get().to(get_connection_history))
            .route("/api/wifi/favorites", web::get().to(get_wifi_favorites))
            .route("/api/wifi/history/clear", web::post().to(clear_history))
            // Network endpoints
            .route("/api/network/status", web::get().to(get_network_status))
            .route("/api/network/configure", web::post().to(configure_network))
            .route("/api/network/clients", web::get().to(get_connected_clients))
            .route("/api/network/wake/{mac}", web::post().to(wake_device))
            .route("/api/network/bandwidth", web::get().to(get_bandwidth_stats))
            // Firewall endpoints
            .route("/api/firewall/rules", web::get().to(get_firewall_rules))
            .route("/api/firewall/apply", web::post().to(apply_firewall_rule))
            .route("/api/firewall/save", web::post().to(save_firewall_rules))
            // DHCP endpoints
            .route("/api/dhcp/config", web::get().to(get_dhcp_config))
            .route("/api/dhcp/config", web::post().to(set_dhcp_config))
            .route("/api/dhcp/restart", web::post().to(restart_dnsmasq))
            // System endpoints
             .route("/api/system/status", web::get().to(get_system_status))
             .route("/api/system/logs", web::get().to(get_system_logs))
             .route("/api/system/logs/dnsmasq", web::get().to(get_dnsmasq_logs))
             // Device endpoints
             .route("/api/devices", web::get().to(get_all_devices))
             .route("/api/devices/{mac}/alias", web::post().to(set_device_alias))
             .route("/api/dhcp/static", web::post().to(set_static_ip))
             // Speed test endpoints
             .route("/api/speedtest/run", web::post().to(speedtest_run))
             // Config endpoints
             .route("/api/config/backup", web::post().to(backup_config))
             .route("/api/config/restore", web::post().to(restore_config))
              // Diagnostics endpoints
             .route("/api/system/diagnostics/ping/{host}", web::post().to(diagnostic_ping))
             .route("/api/system/diagnostics/dns/{domain}", web::post().to(diagnostic_dns))
             .route("/api/system/diagnostics/route/{host}", web::post().to(diagnostic_route))
             .route("/api/system/diagnostics/interfaces", web::get().to(diagnostic_interfaces))
             .route("/api/health", web::get().to(health_check))
            // Serve static files (SvelteKit build output)
            .service(
                fs::Files::new("/", "./web/build")
                    .index_file("index.html"),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
