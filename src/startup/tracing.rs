use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithTonicConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tonic::transport::Channel;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, reload};

use crate::startup::config::TelemetryConfig;

pub struct TracingHandle {
    reload_handle: reload::Handle<EnvFilter, tracing_subscriber::Registry>,
    provider: Option<SdkTracerProvider>,
}

impl TracingHandle {
    pub fn init(telemetry: &TelemetryConfig) -> anyhow::Result<Self> {
        let (filter, reload_handle) = reload::Layer::new(EnvFilter::new("info"));
        let fmt = tracing_subscriber::fmt::layer();

        if telemetry.enabled {
            let channel = Channel::from_shared(telemetry.otlp_endpoint.clone())?.connect_lazy();

            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_channel(channel)
                .build()?;

            let provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(
                    Resource::builder()
                        .with_attribute(KeyValue::new("service.name", telemetry.service_name.clone()))
                        .build(),
                )
                .build();

            let tracer = provider.tracer(telemetry.service_name.clone());
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt)
                .with(otel_layer)
                .try_init()
                .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

            Ok(Self {
                reload_handle,
                provider: Some(provider),
            })
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt)
                .try_init()
                .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

            Ok(Self {
                reload_handle,
                provider: None,
            })
        }
    }

    pub fn set_level(&self, level: &str) -> anyhow::Result<()> {
        self.reload_handle
            .modify(|f| *f = EnvFilter::new(level))
            .map_err(|e| anyhow::anyhow!("Failed to reload log level: {}", e))
    }
}

impl Drop for TracingHandle {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take() {
            let _ = provider.shutdown();
        }
    }
}
