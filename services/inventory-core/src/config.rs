use std::{env, net::SocketAddr};

pub struct AppConfig {
    pub addr: SocketAddr,
    pub db_url: String,
    pub tenant_id: String,
}

pub fn init_tracing() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info,inventory_core=debug".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

pub fn load_config() -> AppConfig {
    AppConfig {
        addr: configured_addr(),
        db_url: configured_database_url(),
        tenant_id: configured_tenant_id(),
    }
}

fn configured_addr() -> SocketAddr {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    format!("{}:{}", host, port)
        .parse()
        .expect("invalid HOST/PORT configuration")
}

fn configured_database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://inventory:inventory@localhost:5432/inventory".to_string())
}

fn configured_tenant_id() -> String {
    env::var("TENANT_ID").unwrap_or_else(|_| "tenant-local".to_string())
}
