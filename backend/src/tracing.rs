//! Tracing subscriber setup.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes process-wide tracing if it has not already been initialized.
pub fn init(filter: &str) {
    let env_filter = EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer());

    let _ = subscriber.try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_init_is_safe_to_call_more_than_once() {
        init("info");
        init("debug");
    }
}
