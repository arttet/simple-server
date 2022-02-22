use clap::{Parser, Subcommand};

use api::pb::{simple_service_client, HelloRequest};

use std::time::Duration;
use tonic::transport::Channel;
use tower::timeout::Timeout;

/// The client lets you control the simple service server.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sends a greeting
    Say {
        /// The gRPC server address
        url: String,

        /// The user's name
        #[clap(short, long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Say { url, name } => {
            let mut client = SimpleServiceClient::new(url.to_owned()).await?;
            client.say_hello(name.to_owned()).await
        }
    }
}

#[derive(Debug)]
struct SimpleServiceClient {
    client: simple_service_client::SimpleServiceClient<Timeout<Channel>>,
}

impl SimpleServiceClient {
    async fn new(url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(url.to_owned())?
            .rate_limit(32, Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;

        let timeout_channel = Timeout::new(channel, Duration::from_secs(1));
        let client = simple_service_client::SimpleServiceClient::new(timeout_channel);

        Ok(Self { client: client })
    }

    async fn say_hello(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(HelloRequest {
            name: name.to_owned(),
        });

        let response = self.client.say_hello(request).await?;
        dbg!(response);

        Ok(())
    }
}
