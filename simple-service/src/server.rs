mod config;
mod service;
mod telemetry;

use clap::Parser;

use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

use http::Request;
use hyper::Body;
use opentelemetry::trace::Tracer;
use tracing::{Level, Span};

use futures_util::FutureExt;

/// The Collector is a service lets you control the Companion Module.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the config file to read
    #[clap(parse(from_os_str), short, long, default_value = "config.yaml")]
    path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cfg = config::Config::parse(&args.path)?;
    let _trace = telemetry::Trace::new(&cfg.tracer)?;

    run(&cfg).await
}

async fn run(cfg: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = tokio::sync::oneshot::channel::<()>();

    let name = cfg.project.name.clone();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.service.port));

    let handle = tokio::spawn(async move {
        let svc = service::SimpleServiceServerImpl::new();
        let svc = api::pb::simple_service_server::SimpleServiceServer::new(svc);

        println!("{} is listening on {}", name, addr);

        // Build our middleware stack
        let layer = ServiceBuilder::new()
            .timeout(std::time::Duration::from_secs(1))
            .layer(
                TraceLayer::new_for_grpc()
                    .make_span_with(|_request: &Request<Body>| tracing::info_span!("http-request"))
                    .on_request(|request: &Request<Body>, _span: &Span| {
                        println!("Middleware {}", request.uri().path());
                        opentelemetry::global::tracer("Open Telemetry Middleware")
                            .start("on_request");
                        tracing::info!("Middleware {}", request.uri().path())
                    })
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .into_inner();

        tonic::transport::Server::builder()
            .layer(layer)
            .add_service(svc)
            .serve_with_shutdown(addr, receiver.map(drop))
            .await
            .unwrap();
    });

    tokio::signal::ctrl_c().await?;
    sender.send(()).unwrap();

    handle.await?;

    Ok(())
}
