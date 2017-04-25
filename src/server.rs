#[warn(unused_must_use)]

// copied mostly from public domain https://github.com/zmack/rust-socks/blob/master/src/server.rs
extern crate byteorder;
extern crate webextension_protocol as protocol;

use std::thread;
use std::time::Duration;
use std::io::Write;
use std::net::{TcpStream, Shutdown};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::net::lookup_host;
use std::io::{Error,ErrorKind};
use std::io;
use std::sync::mpsc::{Sender};
use std::convert::AsRef;
use helpers;

use std::io::Read;

use server::byteorder::{BigEndian, ReadBytesExt};


pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<u64>
    where R: Read, W: Write
{
    let mut buf = [0; 8*1024];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };

        println_stderr!("READ {}", buf.iter().fold(String::new(), |acc, &num| acc + (num as char).to_string().as_str() ));
        writer.write_all(&buf[..len])?;
        written += len as u64;
    }
}

#[allow(dead_code)]
enum RocksError {
    Io(Error),
    Generic(String)
}

pub struct SocksServer {
    tcp_stream: TcpStream,
    sender: Sender<(Sender<String>, String)>,
}

impl SocksServer {

    #[allow(unused_must_use)]
    pub fn new(tcp_stream: TcpStream, sender: Sender<(Sender<String>, String)>) {
        let mut server = SocksServer {
            tcp_stream: tcp_stream,
            sender: sender,
        };
        server.handle_client();
    }

    #[allow(unused_must_use)]
    fn handle_client(&mut self) -> Result<(), RocksError> {
        loop {
            let version = match self.tcp_stream.read_u8() {
                Ok(v) => v ,
                _ => break,
            };
            if version == 5 {
                println_stderr!("wrong protocol version");
                let num_methods = self.tcp_stream.read_u8().unwrap();
                let mut methods = Vec::with_capacity(num_methods as usize);
                unsafe { methods.set_len(num_methods as usize) };
                self.tcp_stream.read_exact(&mut methods).unwrap();
                println_stderr!("num_methods is {:?}, methods is {:?}", num_methods, methods);
                if methods.contains(&2) {
                    // Authenticated
                    self.tcp_stream.write(&[5, 2]);
                    //self.authenticate().unwrap()
                } else {
                    // Unauthenticated
                    self.tcp_stream.write(&[5, 0]);
                }
                //
            } else {
                drop(&self.tcp_stream);
                println_stderr!("wrong protocol version");
                break
            }

            let v1 = self.tcp_stream.read_u8().unwrap();
            let c = self.tcp_stream.read_u8().unwrap();
            let res = self.tcp_stream.read_u8().unwrap();
            let addr_type = self.tcp_stream.read_u8().unwrap();

            println_stderr!("v1 is {:?}", v1);
            println_stderr!("c is {:?}", c);
            println_stderr!("res is {:?}", res);
            println_stderr!("Address type is {:?}", addr_type);
            let addr = self.get_remote_addr(addr_type).unwrap();

            println_stderr!("Address is {:?}", addr);

            let msg = helpers::js_message("getBlock".to_string(), "\"".to_string()+addr.to_string().as_str()+"\"");
            let response = helpers::send_sync(self.sender.clone(), msg);

            println_stderr!("jsengine response {}", response);


            match response.as_ref() {
                "false" => {
                },
                "true" => {
                    drop(&self.tcp_stream);
                    break;
                },
                _ => {},
            }

            println_stderr!("got a connection");
            let outbound = TcpStream::connect(addr).unwrap();
            outbound.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

            self.tcp_stream.write(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
            println_stderr!("Wrote things");

            let mut client_reader = self.tcp_stream.try_clone().unwrap();
            let mut client_writer = self.tcp_stream.try_clone().unwrap();
            let mut socket_reader = outbound.try_clone().unwrap();
            let mut socket_writer = outbound.try_clone().unwrap();

            thread::spawn(move || {
                copy(&mut client_reader, &mut socket_writer);
                client_reader.shutdown(Shutdown::Read);
                socket_writer.shutdown(Shutdown::Write);
            });

            copy(&mut socket_reader, &mut client_writer);
            socket_reader.shutdown(Shutdown::Read);
            client_writer.shutdown(Shutdown::Write);
        }

        return Ok(())
    }

    #[allow(unused_must_use)]
    fn get_remote_addr(&mut self, addr_type: u8) -> Result<SocketAddr, String> {
        match addr_type {
            1 => {
                let mut ip_bytes = [0u8; 4];
                self.tcp_stream.read_exact(&mut ip_bytes);
                let ip = Ipv4Addr::from(ip_bytes);
                let port = self.tcp_stream.read_u16::<BigEndian>().unwrap();

                return Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)));
            },
            3 => {
                let num_str = self.tcp_stream.read_u8().unwrap();
                let mut hostname_vec = Vec::with_capacity(num_str as usize);
                unsafe { hostname_vec.set_len(num_str as usize) };
                self.tcp_stream.read_exact(&mut hostname_vec).unwrap();
                let port = self.tcp_stream.read_u16::<BigEndian>().unwrap();

                let hostname = match String::from_utf8(hostname_vec) { Ok(s) => s, _ => "".to_string() };
                let address = self.resolve_addr_with_cache(&hostname);

                if address.is_none() {
                    return Err(From::from("Empty Address".to_string()))
                } else {
                    // println!("Resolution succeeded for {?} - {?}", hostname, addresses);
                    let mut address = address.unwrap();
                    address.set_port(port);
                    return Ok(address);
                }
            },
            _ => return Err(From::from("Invalid Address Type".to_string()))
        }
    }

    fn resolve_addr_with_cache(&self, hostname: &str) -> Option<SocketAddr> {
        match lookup_host(&hostname) {
            Ok(mut a) => { a.nth(0) },
            _ => { None }
        }
    }
}
