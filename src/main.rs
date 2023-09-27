mod server;
pub mod socks5;

use std::net::{AddrParseError, IpAddr};
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

fn ip_validator(ip: &str) -> Result<IpAddr, AddrParseError> {
    ip.parse()
}


#[derive(Parser)]
struct Cli {
    #[arg(value_enum, long)]
    proto: Proto,
    #[clap(value_parser = ip_validator, long)]
    ip: IpAddr,
    #[clap(value_parser = port_validator, long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.proto {
        Proto::TCP => {
            let server = server::TcpServer::new(cli.port, cli.ip).await.unwrap();
            server.listen().await;
        }
        Proto::UDP => {
            debug!("Not implemented yet");
            return;
        }
    }
}
