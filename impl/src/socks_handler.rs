use super::logger::log;
use super::relay::relay_data;

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpStream};
use std::thread;

pub fn handle_client_stream(mut client_stream: TcpStream) {
    // Accept greeting
    let mut greeting = [0; 256];
    match client_stream.read(&mut greeting) {
        Ok(size) => {
            log(format!("Recived {} bytes for greeting", size));
        }
        Err(error) => {
            log(format!("Error reading greeting {}", error));
            return;
        }
    };

    // Send no authentication response
    let response = "\x05\x00";
    match client_stream.write(response.as_bytes()) {
        Ok(_) => {}
        Err(error) => {
            log(format!("Got error while responding: {}", error));
            return;
        }
    };

    let mut connection_request = [0; 256];
    match client_stream.read(&mut connection_request) {
        Ok(size) => {
            log(format!("Recived {} bytes for connection request", size));
        }
        Err(error) => {
            log(format!("Error reading connection request {}", error));
            return;
        }
    };

    let version = connection_request[0];
    let cmd = connection_request[1];
    let address_type = connection_request[3];
    log(format!(
        "Parsed values: version={}, cmd={}, address_type={}",
        version, cmd, address_type
    ));

    // Command not supported.
    if cmd != 1 {
        let response = "\x05\x07";
        match client_stream.write(response.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                log(format!("Got error while responding: {}", error));
                return;
            }
        };
        return;
    }

    let target_ip: IpAddr;
    let target_port: u16;
    match address_type {
        // IPV4
        1 => {
            target_ip = IpAddr::V4(Ipv4Addr::new(
                connection_request[4],
                connection_request[5],
                connection_request[6],
                connection_request[7],
            ));
            target_port = u16::from_be_bytes([connection_request[8], connection_request[9]]);
        }
        // Domain
        3 => {
            let domain_length = connection_request[4] as usize;
            let domain_bytes = &connection_request[5..(5 + domain_length)];
            let domain = String::from_utf8_lossy(domain_bytes);
            let port_bytes = &connection_request[(5 + domain_length)..(5 + domain_length + 2)];
            target_port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]);

            match dns_lookup::lookup_host(&domain) {
                Ok(addrs) => {
                    target_ip = addrs[0];
                    println!("Resolved IP Address: {}", target_ip);
                }
                Err(e) => {
                    eprintln!("Failed to resolve domain: {}", e);
                    return;
                }
            }
        }
        _ => {
            let response = "\x05\x08";
            match client_stream.write(response.as_bytes()) {
                Ok(_) => {}
                Err(error) => {
                    log(format!("Got error while responding: {}", error));
                    return;
                }
            };
            return;
        }
    }

    match TcpStream::connect((target_ip, target_port)) {
        Ok(remote_socket) => {
            log("Connected to remote server".to_owned());
            // Send success reply to the client
            let mut reply = Vec::new();
            reply.extend_from_slice("\x05\x00\x00\x01".as_bytes());
            reply.extend_from_slice(&[0, 0, 0, 0]);
            reply.extend_from_slice(&(80 as u16).to_be_bytes());
            println!("Reply: {:?}", reply);

            client_stream.write_all(&reply).unwrap();
            relay_data(client_stream, remote_socket);
        }
        Err(e) => {
            log(format!("Failed to connect: {}", e));
            return;
        }
    }
}
