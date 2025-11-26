use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use narad::logger::log;
use narad::relay::relay_data;

async fn client_handler(
    mut client_stream: TcpStream,
    node_main_stream: Arc<Mutex<TcpStream>>,
    node_socket_listener: Arc<TcpListener>,
) {
    // Accept greeting
    let mut greeting = [0; 256];
    match client_stream.read(&mut greeting).await {
        Ok(_size) => {
            // log(format!("Recived {} bytes for greeting", size));
        }
        Err(error) => {
            log(format!("Error reading greeting {}", error));
            return;
        }
    };

    // Send no authentication response
    let response = "\x05\x00";
    match client_stream.write(response.as_bytes()).await {
        Ok(_) => {}
        Err(error) => {
            log(format!("Got error while responding: {}", error));
            return;
        }
    };

    let mut connection_request = [0; 256];
    match client_stream.read(&mut connection_request).await {
        Ok(_size) => {
            // log(format!("Recived {} bytes for connection request", size));
        }
        Err(error) => {
            log(format!("Error reading connection request {}", error));
            return;
        }
    };

    let _version = connection_request[0];
    let cmd = connection_request[1];
    let address_type = connection_request[3];
    // log(format!(
    //     "Parsed values: version={}, cmd={}, address_type={}",
    //     version, cmd, address_type
    // ));

    // Command not supported.
    if cmd != 1 {
        let response = "\x05\x07";
        match client_stream.write(response.as_bytes()).await {
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

            let domain_owned = domain.to_string();
            match tokio::task::spawn_blocking(move || dns_lookup::lookup_host(&domain_owned))
                .await
            {
                Ok(Ok(addrs)) => {
                    target_ip = addrs[0];
                    println!("Resolved IP Address: {}", target_ip);
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to resolve domain: {}", e);
                    return;
                }
                Err(e) => {
                    eprintln!("Task failed: {}", e);
                    return;
                }
            }
        }
        _ => {
            let response = "\x05\x08";
            match client_stream.write(response.as_bytes()).await {
                Ok(_) => {}
                Err(error) => {
                    log(format!("Got error while responding: {}", error));
                    return;
                }
            };
            return;
        }
    }

    // connect to remote server through node
    // log("Connecting to remote server".to_owned());
    // Send node a connect message
    {
        let mut node_stream = node_main_stream.lock().await;
        if let Err(e) = node_stream.write_all("node_tmp_connect".as_bytes()).await {
            log(format!("Error sending to node: {}", e));
            return;
        }
    }

    let (mut tmp_node_socket, addr) = match node_socket_listener.accept().await {
        Ok((socket, addr)) => (socket, addr),
        Err(e) => {
            log(format!("Error accepting node connection: {}", e));
            return;
        }
    };
    log(format!("Node connected from {}, temporarily.", addr));

    // Send node target IP
    if let Err(e) = tmp_node_socket
        .write_all(target_ip.to_string().as_bytes())
        .await
    {
        log(format!("Error sending target IP: {}", e));
        return;
    }

    let mut response: [u8; 2] = [0; 2];
    match tmp_node_socket.read(&mut response).await {
        Ok(_size) => {
            // log(format!("Recived {} bytes from temp node connection", size));
        }
        Err(error) => {
            log(format!("Error reading from tmp node connection {}", error));
            return;
        }
    };

    let response = String::from_utf8_lossy(&response);
    if response.to_string() != String::from("ok") {
        log("ERROR: Sending target ip to node failed.".to_owned());
        return;
    }

    // Send node target port
    if let Err(e) = tmp_node_socket
        .write_all(target_port.to_string().as_bytes())
        .await
    {
        log(format!("Error sending target port: {}", e));
        return;
    }

    let mut response = [0; 2];
    match tmp_node_socket.read(&mut response).await {
        Ok(size) => {
            log(format!("Recived {} bytes from temp node connection", size));
        }
        Err(error) => {
            log(format!("Error reading from tmp node connection {}", error));
            return;
        }
    };
    let response = String::from_utf8_lossy(&response);
    if response.into_owned() != String::from("ok") {
        log("Sending target port to node failed.".to_owned());
        return;
    }

    // Send success reply to the client
    let mut reply = Vec::new();
    reply.extend_from_slice("\x05\x00\x00\x01".as_bytes());
    reply.extend_from_slice(&[0, 0, 0, 0]);
    reply.extend_from_slice(&(80 as u16).to_be_bytes());
    if let Err(e) = client_stream.write_all(&reply).await {
        log(format!("Error sending reply to client: {}", e));
        return;
    }

    relay_data(client_stream, tmp_node_socket).await;
}

async fn node_greetings_handler(mut client_stream: TcpStream) -> Option<(TcpStream, bool)> {
    let mut greeting = [0; 256];
    let bytes_read = match client_stream.read(&mut greeting).await {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                println!("Received no greeting from client");
                return None;
            }
            bytes_read
        }
        Err(e) => {
            println!("Got error -> {}", e);
            return None;
        }
    };
    println!("Received greeting from client:");
    let greeting = String::from_utf8_lossy(&greeting[..bytes_read]);
    println!("{}", greeting);
    if greeting == "node_greeting" {
        Some((client_stream, true))
    } else {
        Some((client_stream, false))
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let binding_ip = "127.0.0.1";
    let binding_ip = "0.0.0.0";
    let listener = TcpListener::bind(format!("{}:{}", binding_ip, "9000")).await?;
    println!("Controller listening for clients on port 9000");
    let node_listener = TcpListener::bind(format!("{}:{}", binding_ip, "1111")).await?;
    println!("Controller listening for nodes on port 1111");

    let node_listener = Arc::new(node_listener);
    let node_stream: Option<Arc<Mutex<TcpStream>>>;

    log("Waiting for node to connect.".to_owned());
    loop {
        let (client_socket, client_addr) = node_listener.accept().await?;
        log(format!(
            "Client connected from {}, verifying if it is a node",
            client_addr
        ));
        // Verify it as a node or client
        match node_greetings_handler(client_socket).await {
            Some((client_socket, is_node)) => {
                if is_node {
                    log("Node's main connection successfull.".to_owned());
                    node_stream = Some(Arc::new(Mutex::new(client_socket)));
                    break;
                } else {
                    log("Node verification failed, waiting for node again.".to_owned());
                    continue;
                }
            }
            None => {
                log("Node verification failed, waiting for node again.".to_owned());
                continue;
            }
        };
    }

    let node_stream = node_stream.expect("Node should be connected");

    log("Node connected, now accepting client connections.".to_owned());
    loop {
        let (client_socket, client_addr) = listener.accept().await?;
        println!("Client connected from {}", client_addr);

        let node_main_stream = Arc::clone(&node_stream);
        let node_socket_listener = Arc::clone(&node_listener);

        tokio::spawn(async move {
            client_handler(client_socket, node_main_stream, node_socket_listener).await;
        });
    }
}
