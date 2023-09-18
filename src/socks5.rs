use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{Error, ErrorKind};
use std::str::FromStr;

pub const VERSION: u8 = 0x05;

#[derive(Debug, PartialEq, Clone)]
pub enum AuthMethods {
    NoAuth = 0x00,
    UserPass = 0x02,
    // GSSAPI = 0x01, UNSUPPORTED
}

impl AuthMethods {
    pub fn from(method: &u8) -> Option<Self> {
        match method {
            0x00 => Some(AuthMethods::NoAuth),
            0x02 => Some(AuthMethods::UserPass),
            _ => {None}
        }
    }
}

pub enum AuthResponseCode {
    Success = 0x00,
    Failure = 0x01
}

#[derive(Debug)]
pub enum Cmd {
    Connect = 0x01,
    /*
    UNSUPPORTED
    Bind = 0x02,
    UdpAssosiate = 0x3,
    */
}

impl Cmd {
    pub fn from(cmd: &u8) -> Option<Self> {
        match cmd {
            0x01 => Some(Cmd::Connect),
            /*
            UNSUPPORTED
            0x02 => Some(Cmd::Bind),
            0x03 => Some(Cmd::UdpAssosiate),
             */
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum Atyp {
    IpV4 = 0x01,
    Domain = 0x03,
    IpV6 = 0x04
}

impl Atyp {
    pub fn from(atyp: &u8) -> Option<Self> {
        match atyp {
            0x01 => Some(Atyp::IpV4),
            0x03 => Some(Atyp::Domain),
            0x04 => Some(Atyp::IpV6),
            _ => None
        }
    }
}

pub enum Reply {
    Success = 0x00,
    ServerFailure = 0x01,
    ConnectionFailure = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TTLExpired = 0x06,
    InvalidCommand = 0x07,
    InvalidAddress = 0x08,
    Other = 0x09
}