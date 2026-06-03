use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Once,
};

use anyhow::{Context, Result};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static TRACING: Once = Once::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub rust_log: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST")
            .ok()
            .map(|value| value.parse().context("HOST must be a valid IP address"))
            .transpose()?
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let port = env::var("PORT")
            .ok()
            .map(|value| value.parse().context("PORT must be a valid u16"))
            .transpose()?
            .unwrap_or(3000);

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            host,
            port,
            rust_log,
        })
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }

    pub fn init_tracing(&self) {
        let rust_log = self.rust_log.clone();

        TRACING.call_once(move || {
            tracing_subscriber::registry()
                .with(EnvFilter::new(rust_log))
                .with(fmt::layer())
                .init();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_addr_uses_host_and_port() {
        let config = AppConfig {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8080,
            rust_log: "debug".to_string(),
        };

        assert_eq!(config.bind_addr(), SocketAddr::from(([127, 0, 0, 1], 8080)));
    }
}
