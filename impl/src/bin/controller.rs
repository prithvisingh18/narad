use std::io::{Read, Result, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};
use std::thread;

use narad::logger::log;
use narad::relay::relay_data;

fn client_handler(
    mut client_stream: TcpStream,
    mut node_main_stream: TcpStream,
    node_socket_listener: TcpListener,
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

    // connect to remote server through node
    log("Connecting to remote server".to_owned());
    // Send node a connect message
    node_main_stream
        .write_all("node_tmp_connect".as_bytes())
        .unwrap();

    let (mut tmp_node_socket, addr) = node_socket_listener.accept().unwrap();
    log(format!("Node connected from {}, temporarily.", addr));

    // Send node target IP
    tmp_node_socket
        .write_all(target_ip.to_string().as_bytes())
        .unwrap();

    let mut response: [u8; 2] = [0; 2];
    match tmp_node_socket.read(&mut response) {
        Ok(size) => {
            log(format!("Recived {} bytes from temp node connection", size));
        }
        Err(error) => {
            log(format!("Error reading from tmp node connection {}", error));
            return;
        }
    };

    let response = String::from_utf8_lossy(&response);
    println!("{} {}", response.to_string().len(), "ok".to_string().len());
    if response.to_string() != String::from("ok") {
        log("Sending target ip failed.".to_owned());
        return;
    }

    // Send node target port
    tmp_node_socket
        .write_all(target_port.to_string().as_bytes())
        .unwrap();

    let mut response = [0; 2];
    match tmp_node_socket.read(&mut response) {
        Ok(size) => {
            log(format!("Recived {} bytes from temp node connection", size));
        }
        Err(error) => {
            log(format!("Error reading from tmp node connection {}", error));
            return;
        }
    };
    let response = String::from_utf8_lossy(&response);
    println!("{} {}", response, "ok");
    if response.into_owned() != String::from("ok") {
        log("Sending target port failed.".to_owned());
        return;
    }

    // Send success reply to the client
    let mut reply = Vec::new();
    reply.extend_from_slice("\x05\x00\x00\x01".as_bytes());
    reply.extend_from_slice(&[0, 0, 0, 0]);
    reply.extend_from_slice(&(80 as u16).to_be_bytes());
    client_stream.write_all(&reply).unwrap();

    relay_data(client_stream, tmp_node_socket);
}

fn node_greetings_handler(mut client_stream: TcpStream) -> Option<(TcpStream, bool)> {
    let mut greeting = [0; 256];
    let bytes_read = match client_stream.read(&mut greeting) {
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

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000")?;
    println!("Controller listening for clients on port 9000");
    let node_listener = TcpListener::bind("0.0.0.0:1111")?;
    println!("Controller listening for nodes on port 1111");

    /*
    let thread_1_listener = listener.try_clone()?;
    let h1 = thread::spawn(move || {
        for stream in thread_1_listener.incoming() {
            let mut stream = stream.unwrap();
            thread::spawn(move || {
                println!("Accepted connection from thread 1");
                loop {
                    let mut msg = [0; 256];
                    let bytes_read = match stream.read(&mut msg) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                continue;
                            }
                            bytes_read
                        }
                        Err(e) => {
                            println!("Got error -> {}", e);
                            continue;
                        }
                    };
                    println!("Received msg from client:");
                    let msg = String::from_utf8_lossy(&msg[..bytes_read]);
                    println!("thread 1 -> {}", msg);
                }
            });
            // thread::sleep(Duration::from_secs(2));
        }
    });

    let thread_2_listener = listener.try_clone()?;
    let h2 = thread::spawn(move || {
        for stream in thread_2_listener.incoming() {
            let mut stream = stream.unwrap();
            thread::spawn(move || {
                println!("Accepted connection from thread 2");
                loop {
                    let mut msg = [0; 256];
                    let bytes_read = match stream.read(&mut msg) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                continue;
                            }
                            bytes_read
                        }
                        Err(e) => {
                            println!("Got error -> {}", e);
                            continue;
                        }
                    };
                    println!("Received msg from client:");
                    let msg = String::from_utf8_lossy(&msg[..bytes_read]);
                    println!("thread 2 -> {}", msg);
                }
            });
        }
    });
    h2.join().unwrap();
    h1.join().unwrap();

    // let mut thread_1_socket = client_socket.try_clone()?;
        // thread::spawn(move || loop {
        //     let mut msg = [0; 256];
        //     let bytes_read = match thread_1_socket.read(&mut msg) {
        //         Ok(bytes_read) => {
        //             if bytes_read == 0 {
        //                 continue;
        //             }
        //             bytes_read
        //         }
        //         Err(e) => {
        //             println!("Got error -> {}", e);
        //             continue;
        //         }
        //     };
        //     println!("Received msg from client:");
        //     let msg = String::from_utf8_lossy(&msg[..bytes_read]);
        //     println!("thread 1 -> {}", msg);
        // });

        // let mut thread_2_socket = client_socket.try_clone()?;
        // thread::spawn(move || loop {
        //     let mut msg = [0; 256];
        //     let bytes_read = match thread_2_socket.read(&mut msg) {
        //         Ok(bytes_read) => {
        //             if bytes_read == 0 {
        //                 println!("Received no msg from client");
        //                 continue;
        //             }
        //             bytes_read
        //         }
        //         Err(e) => {
        //             println!("Got error -> {}", e);
        //             continue;
        //         }
        //     };
        //     println!("Received msg from client:");
        //     let msg = String::from_utf8_lossy(&msg[..bytes_read]);
        //     println!("thread 2 -> {}", msg);
        // });

    */

    let mut nodes: Vec<TcpStream> = Vec::new();
    log("Waiting for node to connect.".to_owned());
    for stream in node_listener.incoming() {
        let client_socket = stream?;
        let client_addr = client_socket.peer_addr()?;
        log(format!(
            "Client connected from {}, verifying if it is a node",
            client_addr
        ));
        // Verfy it as a node or client
        match node_greetings_handler(client_socket) {
            Some((client_socket, is_node)) => {
                if is_node == true {
                    log("Node's main connection successfull.".to_owned());
                    nodes.push(client_socket);
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

    for stream in listener.incoming() {
        let client_socket = stream?;
        let client_addr = client_socket.peer_addr()?;
        println!("Client connected from {}", client_addr);
        let node_main_stream = nodes[0].try_clone()?;
        let node_socket_listener = node_listener.try_clone()?;
        thread::spawn(move || {
            client_handler(client_socket, node_main_stream, node_socket_listener);
        });
    }

    Ok(())
}
