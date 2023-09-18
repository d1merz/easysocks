use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::io::{Error, ErrorKind};
use std::io::ErrorKind::Other;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::os::macos::raw::stat;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use clap::builder::TypedValueParser;
use tinydb::Database;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use serde::{Serialize, Deserialize};
use tinydb::error::DatabaseError;

use crate::socks5::{*};
use crate::socks5::Atyp::IpV4;

struct ServerParams {
    port: u16,
    ip: String,
}

pub struct TcpServer {
    params: ServerParams,
    listener: TcpListener,
}

#[derive(PartialEq, Eq, Hash, Deserialize, Serialize)]
struct Client {
    user: String,
    pass: String,
}


impl TcpServer {
    pub async fn new(port: u16,
                     ip: String) -> io::Result<Self> {
        let params = ServerParams { port: port.clone(), ip: ip.clone() };
        match TcpListener::bind((ip, port)).await {
            Ok(listener) => {
                println!("TcpServer is successfully started!");
                Ok(TcpServer { params, listener })
            }
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
                    Ok(_) => {}
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

    async fn process_connection(mut stream: &mut TcpStream) -> std::io::Result<()> {
        println!("{} connected", stream.peer_addr().unwrap());
        let client_auth_methods = Self::parse_client_auth_methods(&mut stream).await?;
        println!("Client auth methods: {:?}", client_auth_methods);
        let auth_method = Self::define_auth_method(&client_auth_methods);
        println!("Server wants to use {:?}", auth_method.clone());
        Self::auth_client(&mut stream, auth_method).await?;
        println!("Client authorized, listening to requests...");
        let (addr, port) = Self::process_request(&mut stream).await?;
        println!("Client destination address is {:?}, port is {}", addr, port);
        Ok(())
    }
    async fn parse_client_auth_methods(stream: &mut TcpStream) -> Result<Vec<AuthMethods>, std::io::Error> {
        let mut version = 0u8;
        stream.read_exact(std::slice::from_mut(&mut version)).await?;
        if version != VERSION {
            return Err(Error::new(ErrorKind::Other, format!("Unsupported protocol version {}", version)));
        }
        let mut nmethods = 0u8;
        stream.read_exact(std::slice::from_mut(&mut nmethods)).await?;
        let mut methods = vec![];
        for _ in 0..nmethods {
            let mut method = 0u8;
            stream.read_exact(std::slice::from_mut(&mut method)).await?;
            if let Some(method) = AuthMethods::from(&method) {
                methods.push(method);
            }
        }
        Ok(methods)
    }

    fn define_auth_method(methods: &Vec<AuthMethods>) -> AuthMethods {
        let mut method = AuthMethods::NoAuth;
        if methods.contains(&AuthMethods::UserPass) {
            method = AuthMethods::UserPass;
        }
        method
    }

    async fn parse_user_pass(stream: &mut TcpStream) -> Result<(String, String), std::io::Error> {
        let mut version = 0u8;
        stream.read_exact(std::slice::from_mut(&mut version)).await?;
        let mut ulen = 0u8;
        stream.read_exact(std::slice::from_mut(&mut ulen)).await?;
        let mut username = vec![0u8; ulen as usize];
        stream.read_exact(&mut username).await?;
        let mut plen = 0u8;
        stream.read_exact(std::slice::from_mut(&mut plen)).await?;
        let mut password = vec![0u8; plen as usize];
        stream.read_exact(&mut password).await?;
        let username = String::from_utf8_lossy(&username).to_string();
        let password = String::from_utf8_lossy(&password).to_string();
        Ok((username, password))
    }

    async fn auth_client(stream: &mut TcpStream, auth_method: AuthMethods) -> Result<(), std::io::Error> {
        match auth_method {
            AuthMethods::NoAuth => {
                stream.write_all(&[VERSION, AuthMethods::NoAuth as u8]).await
            }
            AuthMethods::UserPass => {
                stream.write_all(&[VERSION, AuthMethods::UserPass as u8]).await?;
                let (user, pass) = Self::parse_user_pass(stream).await?;
                let db: Database<Client> = Database::from(PathBuf::from("clients.tinydb")).unwrap();
                if db.contains(&Client { user, pass }) {
                    stream.write_all(&[1, AuthResponseCode::Success as u8]).await?;
                    Ok(())
                } else {
                    stream.write_all(&[1, AuthResponseCode::Failure as u8]).await?;
                    Err(std::io::Error::new(Other, "No such user in clients database"))
                }
            }
        }
    }

    async fn reply(stream: &mut TcpStream, status: Reply, server_port: u16) -> Result<(), std::io::Error> {
        let buf = [VERSION, status as u8, 0x00, Atyp::IpV4 as u8, 0x00, 0x00, 0x00, 0x00, (server_port.clone() >> 8) as u8, ((server_port << 8) >> 8) as u8];
        stream.write_all(&buf).await?;
        Ok(())
    }

    async fn process_request(stream: &mut TcpStream) -> Result<(SocketAddr, u16), std::io::Error> {
        let mut version = 0u8;
        stream.read_exact(std::slice::from_mut(&mut version)).await?;
        if version != VERSION {
            return Err(Error::new(ErrorKind::Other, format!("Unsupported protocol version {}", version)));
        }
        let mut cmd = 0u8;
        stream.read_exact(std::slice::from_mut(&mut cmd)).await?;
        let _ = match Cmd::from(&cmd) {
            Some(command) => command,
            None => {
                Self::reply(stream, Reply::InvalidCommand, 0x00).await?;
                return Err(std::io::Error::new(Other, "Cannot parse client packet header: Command not found"))
            }
        };
        let mut rsv = 0u8;
        stream.read_exact(std::slice::from_mut(&mut rsv)).await?;
        let mut atyp = 0u8;
        stream.read_exact(std::slice::from_mut(&mut atyp)).await?;
        let addr_type = match Atyp::from(&atyp) {
            Some(addr_type) => addr_type,
            None => {
                Self::reply(stream, Reply::InvalidAddress, 0x00).await;
                return Err(std::io::Error::new(Other, "Invalid address type"))
            }
        };
        println!("Address type {:?}", addr_type);
        let addr = match addr_type {
            Atyp::IpV4 => {
                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).await?;
                buf.to_vec()
            }
            Atyp::IpV6 => {
                let mut buf = [0u8; 16];
                stream.read_exact(&mut buf).await?;
                buf.to_vec()
            }
            Atyp::Domain => {
                let mut len = 0u8;
                stream.read_exact(std::slice::from_mut(&mut len)).await?;
                let mut domain: Vec<u8> = vec![0u8; len as usize];
                stream.read_exact(&mut domain).await?;
                let name = String::from_utf8_lossy(&domain);
                println!("Domain {}", name);
                domain
            }
        };
        let mut port = [0u8; 2];
        stream.read_exact(&mut port).await?;
        let port = Self::toU16(&port[0], &port[1]);
        let mut target_connection = match addr_type {
            Atyp::IpV4 => {
                let target_host = SocketAddr::new(IpAddr::from(Ipv4Addr::new(addr[0].clone(),
                                                                             addr[1].clone(),
                                                                             addr[2].clone(),
                                                                             addr[3].clone())), port);
                TcpStream::connect(target_host).await
            }
            Atyp::IpV6 => {
                let target_host = SocketAddr::new(IpAddr::from(Ipv6Addr::new(Self::toU16(&addr[0], &addr[1]),
                                                                             Self::toU16(&addr[2], &addr[3]),
                                                                             Self::toU16(&addr[4], &addr[5]),
                                                                             Self::toU16(&addr[6], &addr[7]),
                                                                             Self::toU16(&addr[8], &addr[9]),
                                                                             Self::toU16(&addr[10], &addr[11]),
                                                                             Self::toU16(&addr[12], &addr[13]),
                                                                             Self::toU16(&addr[14], &addr[15]),
                )), port);
                TcpStream::connect(target_host).await
            }
            Atyp::Domain => {
                TcpStream::connect(format!("{}:{}", String::from_utf8_lossy(&addr), port)).await
            }
        };

       let target_address = match target_connection {
            Err(err) =>{
                let status= match err.kind() {
                    // ErrorKind::NetworkUnreachable => Reply::NetworkUnreachable, Unstable in current tokio version
                    ErrorKind::ConnectionRefused => Reply::ConnectionRefused,
                    ErrorKind::ConnectionReset => Reply::ConnectionFailure,
                    // ErrorKind::HostUnreachable => Reply::HostUnreachable, Unstable in current tokio version
                    ErrorKind::TimedOut => Reply::TTLExpired,
                    _ => Reply::Other
                };
                Self::reply(stream, status, port.clone()).await?;
                return Err(std::io::Error::new(Other, "Target connection error"))
            }
            Ok(mut target_stream) => {
                Self::reply(stream, Reply::Success, target_stream.local_addr().unwrap().port()).await?;
                tokio::io::copy_bidirectional(stream, &mut target_stream).await?;
                target_stream.peer_addr().unwrap()
            }
        };
       Ok((target_address, port.clone()))
    }
    fn toU16(a: &u8, b: &u8) -> u16 {
        (u16::from(a.to_owned()) << 8) | u16::from(b.to_owned())
    }
}