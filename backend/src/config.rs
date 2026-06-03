//! Runtime configuration loading.

use std::{env, net::IpAddr};

use thiserror::Error;

const DEFAULT_HOST: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_LOG_FILTER: &str = "promptops_arena_backend=info,tower_http=info";

/// Application configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// HTTP server configuration.
    pub server: ServerConfig,
    /// Logging and tracing configuration.
    pub log: LogConfig,
}

/// HTTP server configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Address to bind the HTTP server to.
    pub host: IpAddr,
    /// Port to bind the HTTP server to.
    pub port: u16,
}

/// Logging and tracing configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogConfig {
    /// Tracing subscriber filter directive.
    pub filter: String,
}

/// Configuration loading error.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// The configured host is not a valid IP address.
    #[error("invalid APP_HOST value: {value}")]
    InvalidHost {
        /// Invalid value.
        value: String,
    },

    /// The configured port is not a valid TCP port.
    #[error("invalid APP_PORT value: {value}")]
    InvalidPort {
        /// Invalid value.
        value: String,
    },
}

impl AppConfig {
    /// Loads configuration from the current process environment.
    ///
    /// # Errors
    ///
    /// Returns an error when an environment override cannot be parsed.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(env::vars())
    }

    /// Loads configuration from key-value pairs.
    ///
    /// This keeps parsing testable without mutating process-global environment.
    ///
    /// # Errors
    ///
    /// Returns an error when a supported override cannot be parsed.
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        for (key, value) in pairs {
            let key = key.into();
            let value = value.into();

            match key.as_str() {
                "APP_HOST" => {
                    config.server.host = value
                        .parse()
                        .map_err(|_| ConfigError::InvalidHost { value })?;
                }
                "APP_PORT" => {
                    config.server.port = value
                        .parse()
                        .map_err(|_| ConfigError::InvalidPort { value })?;
                }
                "RUST_LOG" => {
                    config.log.filter = value;
                }
                _ => {}
            }
        }

        Ok(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: DEFAULT_HOST,
                port: DEFAULT_PORT,
            },
            log: LogConfig {
                filter: DEFAULT_LOG_FILTER.to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_are_stable() {
        let config = AppConfig::from_pairs([] as [(String, String); 0]).expect("valid defaults");

        assert_eq!(config, AppConfig::default());
        assert_eq!(config.server.host, DEFAULT_HOST);
        assert_eq!(config.server.port, DEFAULT_PORT);
        assert_eq!(config.log.filter, DEFAULT_LOG_FILTER);
    }

    #[test]
    fn config_reads_supported_environment_overrides() {
        let config = AppConfig::from_pairs([
            ("APP_HOST", "0.0.0.0"),
            ("APP_PORT", "8080"),
            ("RUST_LOG", "debug"),
            ("IGNORED", "value"),
        ])
        .expect("valid overrides");

        assert_eq!(
            config.server.host,
            "0.0.0.0".parse::<IpAddr>().expect("valid ip")
        );
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log.filter, "debug");
    }

    #[test]
    fn config_rejects_invalid_host() {
        let error = AppConfig::from_pairs([("APP_HOST", "localhost")]).expect_err("invalid host");

        assert_eq!(
            error,
            ConfigError::InvalidHost {
                value: "localhost".to_owned()
            }
        );
    }

    #[test]
    fn config_rejects_invalid_port() {
        let error = AppConfig::from_pairs([("APP_PORT", "70000")]).expect_err("invalid port");

        assert_eq!(
            error,
            ConfigError::InvalidPort {
                value: "70000".to_owned()
            }
        );
    }
}
