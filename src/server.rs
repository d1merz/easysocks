use tokio::io;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
struct ServerError {

}


struct User {

}

struct ServerParams {
    port: u16,
    ip: String,
    auth_methods: Vec<u8>,
    users: Vec<User>,
}

pub struct TcpServer {
    params: ServerParams,
    listener: TcpListener,
}

pub struct UdpServer {
    params: ServerParams,
    socket: UdpSocket,
}

impl TcpServer {
    pub async fn new(port: u16,
           ip: String) -> io::Result<Self> {
        let params = ServerParams {port: port.clone(), ip: ip.clone(), auth_methods: vec![], users: vec![]};
        match TcpListener::bind((ip, port)).await {
            Ok(listener) => {
                println!("TcpServer is successfully started!");
                Ok(TcpServer {params, listener})
            },
            Err(err) => {
                println!("Cannot start TcpServer because of the error, {}", err.to_string());
                Err(err)
            }
        }
    }
}

impl UdpServer {
    pub async fn new(port: u16,
                 ip: String) -> io::Result<Self> {
        let params = ServerParams {port: port.clone(), ip: ip.clone(), auth_methods: vec![], users: vec![]};
        match UdpSocket::bind((ip, port)).await {
            Ok(socket) => {
                println!("UdpServer is successfully started!");
                Ok(UdpServer {params, socket})
            },
            Err(err) => {
                println!("Cannot start TcpServer because of the error, {}", err.to_string());
                Err(err)
            }
        }
    }
}