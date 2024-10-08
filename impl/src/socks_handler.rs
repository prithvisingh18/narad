use super::logger::log;
use super::relay::relay_data;

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::sync::Arc;

pub fn handle_client_stream(
    mut client_stream: TcpStream,
    auth_required: Arc<bool>,
    username: Arc<String>,
    password: Arc<String>,
) {
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
    println!(
        "got this greeting -> {}",
        String::from_utf8_lossy(&greeting)
    );
    println!("{} {} {}", greeting[0], greeting[1], greeting[2]);
    if *auth_required {
        let response = "\x05\x02"; // \x02 indicates username/password authentication
        match client_stream.write(response.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                log(format!("Got error while responding: {}", error));
                return;
            }
        };
        let auth_creds = String::from(format!("{}{}", *username, *password));
        println!("{}", auth_creds.as_bytes().len());
        // Read username and password authentication message
        let mut auth_data = vec![0; 512]; // Buffer for authentication data
        match client_stream.read(&mut auth_data) {
            Ok(size) => {
                log(format!("Received {} bytes for authentication data", size));
            }
            Err(error) => {
                log(format!("Error reading authentication data: {}", error));
                return;
            }
        };
        let req_auth_data: String = String::from_utf8_lossy(&auth_data)
            .to_string()
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();
        println!("{} {}", auth_creds, req_auth_data);
        if req_auth_data == auth_creds {
            log("Auth successfull, proceeding further.".to_owned());
            let auth_success_response = "\x01\x00"; // \x00 indicates general success
            match client_stream.write(auth_success_response.as_bytes()) {
                Ok(_) => {}
                Err(error) => {
                    log(format!("Got error while responding: {}", error));
                    return;
                }
            };
        } else {
            log("Auth failed.".to_owned());
            let auth_fail_response = "\x01\x01"; // \x01 indicates general failure
            match client_stream.write(auth_fail_response.as_bytes()) {
                Ok(_) => {
                    return;
                }
                Err(error) => {
                    log(format!("Got error while responding: {}", error));
                    return;
                }
            };
        }
    } else {
        // Send no authentication response
        let response = "\x05\x00";
        match client_stream.write(response.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                log(format!("Got error while responding: {}", error));
                return;
            }
        };
    }

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
