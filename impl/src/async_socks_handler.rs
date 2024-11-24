use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use dns_lookup;

pub async fn handle_client_stream(
    mut client_stream: TcpStream,
    auth_required: Arc<bool>,
    username: Arc<String>,
    password: Arc<String>,
) {
    // Accept greeting
    let mut greeting = vec![0; 256];
    match client_stream.read(&mut greeting).await {
        Ok(size) => {
            log(format!("Received {} bytes for greeting", size));
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

    if *auth_required {
        let response = b"\x05\x02"; // \x02 indicates username/password authentication
        if client_stream.write_all(response).await.is_err() {
            log("Error responding with auth requirement.".to_owned());
            return;
        }

        let auth_creds = format!("{}{}", *username, *password);
        let mut auth_data = vec![0; 512]; // Buffer for authentication data
        match client_stream.read(&mut auth_data).await {
            Ok(size) => {
                log(format!("Received {} bytes for authentication data", size));
            }
            Err(error) => {
                log(format!("Error reading authentication data: {}", error));
                return;
            }
        };

        let req_auth_data: String = String::from_utf8_lossy(&auth_data)
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();

        if req_auth_data == auth_creds {
            log("Auth successful, proceeding further.".to_owned());
            let auth_success_response = b"\x01\x00"; // \x00 indicates general success
            if client_stream.write_all(auth_success_response).await.is_err() {
                log("Error responding with auth success.".to_owned());
                return;
            }
        } else {
            log("Auth failed.".to_owned());
            let auth_fail_response = b"\x01\x01"; // \x01 indicates general failure
            if client_stream.write_all(auth_fail_response).await.is_err() {
                log("Error responding with auth failure.".to_owned());
                return;
            }
            return;
        }
    } else {
        // Send no authentication response
        let response = b"\x05\x00";
        if client_stream.write_all(response).await.is_err() {
            log("Error responding with no authentication.".to_owned());
            return;
        }
    }

    let mut connection_request = vec![0; 256];
    match client_stream.read(&mut connection_request).await {
        Ok(size) => {
            log(format!("Received {} bytes for connection request", size));
        }
        Err(error) => {
            log(format!("Error reading connection request: {}", error));
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
        let response = b"\x05\x07";
        if client_stream.write_all(response).await.is_err() {
            log("Error responding with unsupported command.".to_owned());
            return;
        }
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
            let response = b"\x05\x08";
            if client_stream.write_all(response).await.is_err() {
                log("Error responding with unsupported address type.".to_owned());
                return;
            }
            return;
        }
    }

    match TcpStream::connect((target_ip, target_port)).await {
        Ok(mut remote_socket) => {
            log("Connected to remote server".to_owned());

            // Send success reply to the client
            let mut reply = Vec::new();
            reply.extend_from_slice(b"\x05\x00\x00\x01");
            reply.extend_from_slice(&[0, 0, 0, 0]);
            reply.extend_from_slice(&(80 as u16).to_be_bytes());

            if client_stream.write_all(&reply).await.is_err() {
                log("Error sending success reply to client.".to_owned());
                return;
            }

            relay_data(client_stream, remote_socket).await;
        }
        Err(e) => {
            log(format!("Failed to connect to target: {}", e));
            return;
        }
    }
}

// Dummy relay_data function to demonstrate structure
async fn relay_data(mut client_stream: TcpStream, mut remote_socket: TcpStream) -> (u64, u64) {
    tokio::io::copy_bidirectional(&mut client_stream, &mut remote_socket)
        .await
        .unwrap()
}

// Dummy log function for logging messages
fn log(message: String) {
    println!("{}", message);
}

