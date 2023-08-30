use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::socks5;

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
        let mut client_hello_header = [0u8; 2];
        stream.read_exact(&mut client_hello_header).await?;
        let nmethods = socks5::parse_client_methods_header(client_hello_header)?;
        let mut method = 0u8;
        let mut auth_methods = vec![];
        for _ in 0..nmethods {
            stream.read_exact(std::slice::from_mut(&mut method)).await?;
            if let (auth_method) = socks5::parse_client_method(&method) {
                auth_methods.push(auth_method);
            }
        }
        println!("Client auth methods: {:?}", auth_methods);

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
}