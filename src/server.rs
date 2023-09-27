use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use serde::{Deserialize};
use tracing::{debug, error, info, instrument, warn};
use fs4::tokio::AsyncFileExt;
use tokio::fs::File;
use csv;

use crate::socks5::{*};

#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener
}

#[derive(PartialEq, Eq, Hash, Deserialize, Debug)]
pub struct Client {
    pub name: String,
    pub pass: String,
}

impl TcpServer {
    #[instrument]
    pub async fn new(port: u16,
                     ip: IpAddr) -> io::Result<Self> {
        match TcpListener::bind((ip, port)).await {
            Ok(listener) => {
                info!("TcpServer is successfully started!");
                Ok(TcpServer { listener })
            }
            Err(err) => {
                error!("Cannot start TcpServer because of the error, {}", err.to_string());
                Err(err)
            }
        }
    }
    #[instrument]
    pub async fn listen(&self) {
        info!("Listening to connections...");
        while let Ok((mut stream, addr)) = self.listener.accept().await {
            tokio::spawn(async move {
                match Self::process_connection(&mut stream).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to serve a connection {}\n{}", addr, err);
                        if let Err(err) = stream.shutdown().await {
                            error!("Failed to shutdown client {}\n{}", addr, err);
                        }
                    }
                }
            });
        }
    }
    #[instrument]
    async fn process_connection(mut stream: &mut TcpStream) -> std::io::Result<()> {
        info!("{} connected", stream.peer_addr().unwrap());
        let client_auth_methods = Self::parse_client_auth_methods(&mut stream).await?;
        info!("Client auth methods: {:?}", client_auth_methods);
        let auth_method = Self::define_auth_method(&client_auth_methods);
        info!("Server wants to use {:?}", auth_method.clone());
        Self::auth_client(&mut stream, auth_method).await?;
        info!("Client authorized, listening to requests...");
        Self::process_request(&mut stream).await?;
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
                let (name, pass) = Self::parse_user_pass(stream).await?;
                let mut file = File::open("users.csv").await?;
                let mut buf : Vec<u8> = Vec::new();
                file.lock_shared()?;
                file.read_to_end(&mut buf).await?;
                file.unlock()?;
                let mut reader = csv::Reader::from_reader(buf.as_slice());
                let clients: Vec<Result<Client, csv::Error>> = reader.deserialize().collect();
                if clients.into_iter().any(|client| {
                    if client.is_ok()  {
                        let c = client.unwrap();
                        Client {name: name.clone(), pass: pass.clone()} == c
                    } else { false }
                }) {
                    stream.write_all(&[1, AuthResponseCode::Success as u8]).await?;
                    Ok(())
                } else {
                    stream.write_all(&[1, AuthResponseCode::Failure as u8]).await?;
                    Err(std::io::Error::new(ErrorKind::Other, "No such user in clients database"))
                }
            }
        }
    }

    async fn reply(stream: &mut TcpStream, status: Reply, server_port: u16) -> Result<(), std::io::Error> {
        let buf = [VERSION, status as u8, 0x00, Atyp::IpV4 as u8, 0x00, 0x00, 0x00, 0x00, (server_port.clone() >> 8) as u8, ((server_port << 8) >> 8) as u8];
        stream.write_all(&buf).await?;
        Ok(())
    }

    #[instrument]
    async fn process_request(stream: &mut TcpStream) -> Result<(), std::io::Error> {
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
                return Err(std::io::Error::new(ErrorKind::Other, "Cannot parse client packet header: Command not found"))
            }
        };
        let mut rsv = 0u8;
        stream.read_exact(std::slice::from_mut(&mut rsv)).await?;
        let mut atyp = 0u8;
        stream.read_exact(std::slice::from_mut(&mut atyp)).await?;
        let address_type = match Atyp::from(&atyp) {
            Some(address_type) => address_type,
            None => {
                if let Err(err) = Self::reply(stream, Reply::InvalidAddress, 0x00).await {
                    warn!("Cannot send reply to client {}", err);
                }
                return Err(std::io::Error::new(ErrorKind::Other, "Invalid address type"))
            }
        };
        debug!("Address type {:?}", address_type);
        let addr = match address_type {
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
                debug!("Domain {}", name);
                domain
            }
        };
        let mut port = [0u8; 2];
        stream.read_exact(&mut port).await?;
        let port = Self::to_u16(&port[0], &port[1]);
        let target_connection = match address_type {
            Atyp::IpV4 => {
                let target_host = SocketAddr::new(IpAddr::from(Ipv4Addr::new(addr[0].clone(),
                                                                             addr[1].clone(),
                                                                             addr[2].clone(),
                                                                             addr[3].clone())), port);
                TcpStream::connect(target_host).await
            }
            Atyp::IpV6 => {
                let target_host = SocketAddr::new(IpAddr::from(Ipv6Addr::new(Self::to_u16(&addr[0], &addr[1]),
                                                                             Self::to_u16(&addr[2], &addr[3]),
                                                                             Self::to_u16(&addr[4], &addr[5]),
                                                                             Self::to_u16(&addr[6], &addr[7]),
                                                                             Self::to_u16(&addr[8], &addr[9]),
                                                                             Self::to_u16(&addr[10], &addr[11]),
                                                                             Self::to_u16(&addr[12], &addr[13]),
                                                                             Self::to_u16(&addr[14], &addr[15]),
                )), port);
                TcpStream::connect(target_host).await
            }
            Atyp::Domain => {
                TcpStream::connect(format!("{}:{}", String::from_utf8_lossy(&addr), port)).await
            }
        };

       match target_connection {
            Err(err) =>{
                let status= match err.kind() {
                    // ErrorKind::NetworkUnreachable => Reply::NetworkUnreachable, Unstable in current tokio version
                    ErrorKind::ConnectionRefused => Reply::ConnectionRefused,
                    ErrorKind::ConnectionReset => Reply::ConnectionFailure,
                    // ErrorKind::HostUnreachable => Reply::HostUnreachable, Unstable in current tokio version
                    ErrorKind::TimedOut => Reply::HostUnreachable,
                    _ => Reply::Other
                };
                Self::reply(stream, status, port.clone()).await?;
                return Err(std::io::Error::new(ErrorKind::Other, "Target connection error"))
            }
            Ok(mut target_stream) => {
                Self::reply(stream, Reply::Success, target_stream.local_addr().unwrap().port()).await?;
                tokio::io::copy_bidirectional(stream, &mut target_stream).await?;
            }
        };
       Ok(())
    }
    fn to_u16(a: &u8, b: &u8) -> u16 {
        (u16::from(a.to_owned()) << 8) | u16::from(b.to_owned())
    }
}