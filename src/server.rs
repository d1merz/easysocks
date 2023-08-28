use std::collections::HashMap;
use std::fmt;
use std::fmt::write;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
struct ServerError {

}

const SOCKS_VERSION: u8 = 0x05;

static AUTH_LABELS: HashMap<u8, AUTH_METHODS> = HashMap::from([(0x00, AUTH_METHODS::NO_AUTH),
    (0x02, AUTH_METHODS::GSSAPI),
    (0xFF, AUTH_METHODS::USER_PASS)]);

enum AUTH_METHODS {
    NO_AUTH,
    GSSAPI,
    USER_PASS
}

impl fmt::Display for AUTH_METHODS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AUTH_METHODS::NO_AUTH => write!(f, "NO_AUTH"),
            AUTH_METHODS::GSSAPI => write!(f, "GSSAPI"),
            AUTH_METHODS::USER_PASS => write!(f, "USER_PASS")
        }
    }
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
    pub async fn listen(&self) {
        println!("Listening to connections...");
        while let Ok((stream, addr)) = self.listener.accept().await {
            tokio::spawn(async move {parse_client_auth(stream)}).await.unwrap();
        }
    }
}

async fn parse_client_auth(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Auth Methods negotiation starts...");
    let mut version_header = 0u8;
    match stream.read_exact(std::slice::from_mut(&mut version_header)).await {
        Ok(_) => {
            if version_header != SOCKS_VERSION {
                println!("Unsupported SOCKS version: {}", version_header.to_string());
                return stream.shutdown().await
            }
        },
        Err(err) => {
            println!("Error occurred while parsing client SOCKS version: {}", err.to_string());
            return stream.shutdown().await
        }
    }
    let mut nmethods_header = 0u8;
    match stream.read_exact(std::slice::from_mut(&mut nmethods_header)).await {
        Ok(_) => {
            println!("Client supports {} number of auth methods", nmethods_header.to_string());
            for _ in 0..nmethods_header {
                let mut method = 0u8;
                match stream.read_exact(std::slice::from_mut(&mut method)).await {
                    Ok(_) => {
                        if let Some(auth) = AUTH_LABELS.get(&method) {
                            println!("Client supports method {}", auth);
                            //self.params.auth_methods.push(method);
                        }
                    },
                    Err(err) => {
                        println!("Cannot parse AUTH methods: {}", err.to_string());
                        return stream.shutdown().await
                    }
                }
            }
        },
        Err(err) => {
            println!("Error occurred while parsing client NMETHODS header: {}", err.to_string());
            return stream.shutdown().await
        }
    }
    Ok(())
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