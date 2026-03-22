use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, reload};

pub struct TracingHandle {
    reload_handle: reload::Handle<EnvFilter, tracing_subscriber::Registry>,
}

impl TracingHandle {
    pub fn init() -> anyhow::Result<Self> {
        let (filter, reload_handle) = reload::Layer::new(EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

        Ok(Self { reload_handle })
    }

    pub fn set_level(&self, level: &str) -> anyhow::Result<()> {
        self.reload_handle
            .modify(|f| *f = EnvFilter::new(level))
            .map_err(|e| anyhow::anyhow!("Failed to reload log level: {}", e))
    }
}
