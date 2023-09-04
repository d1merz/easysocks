use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::socks5;
use crate::socks5::AuthMethods;

struct ServerParams {
    port: u16,
    ip: String,
}

pub struct TcpServer {
    params: ServerParams,
    listener: TcpListener,
}

pub struct UdpServer {
    params: ServerParams,
    socket: UdpSocket,
}

struct ClientParams {
    auth_methods: Vec<socks5::AuthMethods>,
    connection_status: socks5::ConnectionStatus
}

impl TcpServer {
    pub async fn new(port: u16,
           ip: String) -> io::Result<Self> {
        let params = ServerParams {port: port.clone(), ip: ip.clone()};
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
        while let Ok((mut stream, addr)) = self.listener.accept().await {
            tokio::spawn(async move {
                match Self::process_connection(&mut stream).await {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Failed to serve a connection {}\n{}", addr, err);
                        if let Err(err) = stream.shutdown().await {
                            println!("Failed to shutdown client {}\n{}", addr, err);
                        }
                    }
                }
            });
        }
    }

    async fn process_connection(stream: &mut TcpStream) -> std::io::Result<()> {
        let mut client_auth_header = [0u8; 2];
        stream.read_exact(&mut client_auth_header).await?;
        let nmethods = socks5::parse_client_auth_header(client_auth_header)?;
        let mut method = 0u8;
        let mut auth_methods = vec![];
        for _ in 0..nmethods {
            stream.read_exact(std::slice::from_mut(&mut method)).await?;
            if let Some(auth_method) = socks5::parse_client_method(&method) {
                auth_methods.push(auth_method);
            }
        }
        println!("Client auth methods: {:?}", auth_methods);
        let mut auth_method = AuthMethods::NO_AUTH;
        if auth_methods.contains(&AuthMethods::USER_PASS) {
            auth_method = AuthMethods::USER_PASS;
        } else if auth_methods.contains(&AuthMethods::GSSAPI) {
            auth_method = AuthMethods::GSSAPI;
        }
        let server_auth = [socks5::SOCKS_VERSION, auth_method as u8];
        stream.write_all(server_auth.as_slice()).await?;
        let mut username_len_buf = [0u8, 2];
        stream.read_exact(&mut username_len_buf).await?;
        let username_len = socks5::parse_username_len(username_len_buf)?;
        let mut username_buf = vec![0u8; username_len];
        stream.read_exact(&mut username_buf).await?;
        let username = String::from_utf8_lossy(&username_buf).to_string();
        Ok(())
    }
}



impl UdpServer {
    pub async fn new(port: u16,
                 ip: String) -> io::Result<Self> {
        let params = ServerParams {port: port.clone(), ip: ip.clone()};
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
    pub async fn run(&self) -> std::io::Result<()> {
        let mut clients: HashMap<SocketAddr, ClientParams> = HashMap::new();
        loop {
            // max size of client hello is 255 + 255 + 6*255 = 2040 = 255 bytes = 8 u8 units
            let mut buf = [0u8; 2048]; // standard buffer size for UDP datagram > MTU = 1500
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            if let Some(client) = clients.get(&addr) {
                match client.connection_status {
                    socks5::ConnectionStatus::HELLO => todo!(),
                    socks5::ConnectionStatus::AUTH => todo!()
                }
            } else {
                let nmethods = socks5::parse_client_auth_header(buf[0..2].try_into().unwrap())?;
                let mut auth_methods = vec![];
                for i in 0..nmethods {
                    let method = buf.get(i as usize + 2).unwrap();
                    if let Some(auth_method) = socks5::parse_client_method(&method) {
                        auth_methods.push(auth_method);
                    }
                }
                clients.insert(addr, ClientParams{auth_methods, connection_status: socks5::ConnectionStatus::HELLO});
            }
            let len = self.socket.send_to(&buf[..len], addr).await?;
            println!("{:?} bytes sent", len);
        }
    }
}