mod server;
mod client;
use clap::{Parser, ValueEnum};
use clap_num::number_range;

// Режимы, в которых будет работать приложение
#[derive(ValueEnum, Clone)]
enum Mode {
    Client,
    Server
}

// Выбор доступных протоколов
#[derive(ValueEnum, Clone)]
enum Proto {
    TCP,
    UDP
}

// Первые 1024 порта зарезервированы операционной системой, поэтому сразу напишем обработчик, валидирующий номер порта
fn port_validator(port: &str) -> Result<u16, String> {
    number_range(port, 1024, u16::MAX)
}


// Структура для аргументов командной строки
#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
    #[arg(value_enum)]
    proto: Proto,
    #[clap(value_parser=port_validator)]
    port: u16
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Client => {
            println!("Client mode");
            todo!()
        }
        Mode::Server => {
            println!("Server mode");
            match cli.proto {
                Proto::TCP => {
                    let server = server::TcpServer::new(cli.port, "127.0.0.1".to_string()).await.unwrap();
                },
                Proto::UDP => {
                    let server = server::UdpServer::new(cli.port, "127.0.0.1".to_string()).await.unwrap();
                }
            }
        }
    }
}
