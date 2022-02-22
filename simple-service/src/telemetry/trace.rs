use crate::config;

use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::Registry;

pub struct Trace {
    #[allow(dead_code)]
    subscriber: Layered<OpenTelemetryLayer<Registry, opentelemetry::sdk::trace::Tracer>, Registry>,
}

impl Trace {
    pub fn new(cfg: &config::Tracer) -> Result<Self, Box<dyn std::error::Error>> {
        opentelemetry::global::set_text_map_propagator(
            opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        );

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name(&cfg.name)
            .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("exporter", "jaeger"),
                    opentelemetry::KeyValue::new("service.name", cfg.name.clone()),
                    opentelemetry::KeyValue::new("service.namespace", cfg.namespace.clone()),
                ]),
            ))
            .install_batch(opentelemetry::runtime::Tokio)?;

        let subscriber =
            Registry::default().with(tracing_opentelemetry::layer().with_tracer(tracer));

        println!("The tracer is running");
        Ok(Self {
            subscriber: subscriber,
        })
    }
}

impl Drop for Trace {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
        println!("The tracer shut down");
    }
}
