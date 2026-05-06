use std::{env, process};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry};
use url::Url;

pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let is_dev = environment == "development";
    let filter_str = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    if is_dev {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_ansi(true)
            .with_filter(EnvFilter::new(&filter_str));
        Registry::default().with(fmt_layer).try_init()?;
    } else {
        let loki_endpoint = env::var("LOKI_ENDPOINT")
            .unwrap_or_else(|_| "http://logging-loki.logging.svc.cluster.local:3100".to_string());
        let url = Url::parse(&loki_endpoint)?;
        let (loki_layer, task) = tracing_loki::builder()
            .label("service", env!("CARGO_PKG_NAME"))?
            .label("env", &environment)?
            .extra_field("pid", format!("{}", process::id()))?
            .build_url(url)?;
        tokio::spawn(task);
        let filter = EnvFilter::new(&filter_str);
        Registry::default()
            .with(loki_layer.with_filter(filter))
            .try_init()?;
    }

    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("unknown panic");
        eprintln!("\x1b[31m[PANIC] at {}: {}\x1b[0m", location, msg);
        tracing::error!(location = %location, "[PANIC] {}", msg);
    }));

    Ok(())
}

pub use tracing::{debug, error, info, warn};
