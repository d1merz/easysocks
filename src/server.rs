use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::str::FromStr;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
struct ServerError {

}

const SOCKS_VERSION: u8 = 0x05;

pub enum AuthMethods {
    NO_AUTH = 0x00,
    GSSAPI = 0x01,
    USER_PASS = 0x02,
}

impl FromStr for AuthMethods {

    type Err = ();

    fn from_str(input: &str) -> Result<AuthMethods, Self::Err> {
        match input {
            "0x00"  => Ok(Self::NO_AUTH),
            "0x01"  => Ok(Self::GSSAPI),
            "0x02"  => Ok(Self::USER_PASS),
            _      => Err(()),
        }
    }
}

impl Display for AuthMethods {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::GSSAPI => write!(f, "GSSAPI"),
            Self::NO_AUTH => write!(f, "No authentication"),
            Self::USER_PASS => write!(f, "User and Password")
        }
    }
}

struct Client {
    auth_methods: Vec<u8>,
}

struct ServerParams {
    port: u16,
    ip: String,
    clients: Vec<Client>,
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
        let params = ServerParams {port: port.clone(), ip: ip.clone(), clients: vec![]};
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

async fn parse_client_auth(mut stream: TcpStream) -> Result<Vec<AuthMethods>, std::io::Result<()>> {
    println!("Auth Methods negotiation starts...");
    let mut client_methods = vec![];
    let mut version_header = 0u8;
    match stream.read_exact(std::slice::from_mut(&mut version_header)).await {
        Ok(_) => {
            if version_header != SOCKS_VERSION {
                println!("Unsupported SOCKS version: {}", version_header.to_string());
                return Err(stream.shutdown().await)
            }
        },
        Err(err) => {
            println!("Error occurred while parsing client SOCKS version: {}", err.to_string());
            return Err(stream.shutdown().await)
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
                        if let Ok(auth_method) = AuthMethods::from_str(method.to_string().as_str()) {
                            println!("Client supports method {}", auth_method);
                            client_methods.push(auth_method);
                        }
                    },
                    Err(err) => {
                        println!("Cannot parse AUTH methods: {}", err.to_string());
                        return Err(stream.shutdown().await)
                    }
                }
            }
        },
        Err(err) => {
            println!("Error occurred while parsing client NMETHODS header: {}", err.to_string());
            return Err(stream.shutdown().await)
        }
    }
    Ok(client_methods)
}

impl UdpServer {
    pub async fn new(port: u16,
                 ip: String) -> io::Result<Self> {
        let params = ServerParams {port: port.clone(), ip: ip.clone(), clients: vec![]};
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