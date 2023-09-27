mod server;
pub mod socks5;

use clap::{Parser, ValueEnum};
use clap_num::number_range;
use tracing::debug;
use tracing_subscriber;

#[derive(ValueEnum, Clone)]
enum Proto {
    TCP,
    UDP,
}

fn port_validator(port: &str) -> Result<u16, String> {
    number_range(port, 1024, u16::MAX)
}


#[derive(Parser)]
struct Cli {
    #[arg(value_enum, long)]
    proto: Proto,
    #[clap(value_parser = port_validator, long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.proto {
        Proto::TCP => {
            let server = server::TcpServer::new(cli.port, "0.0.0.0".to_string()).await.unwrap();
            server.listen().await;
        }
        Proto::UDP => {
            debug!("Not implemendet yet");
            return;
        }
    }
}
