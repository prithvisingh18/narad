use super::logger::log;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::thread;

// Function to forward data from source to destination
fn forward_data(mut source: TcpStream, mut destination: TcpStream) -> Result<(), std::io::Error> {
    let mut buf = [0; 4096];
    loop {
        let bytes_read = source.read(&mut buf)?;
        if bytes_read == 0 {
            println!("Received no data; closing the relay.");
            break;
        }
        destination.write_all(&buf[..bytes_read])?;
    }
    Ok(())
}

// Function to relay data between client and server sockets
pub fn relay_data(client_socket: TcpStream, server_socket: TcpStream) {
    // Clone sockets for ownership transfer to threads
    let client_socket_copy = client_socket
        .try_clone()
        .expect("Failed to clone client socket");
    let server_socker_copy = server_socket
        .try_clone()
        .expect("Failed to clone server socket");

    let client_socket_copy_main = client_socket
        .try_clone()
        .expect("Failed to clone client socket");
    let server_socker_copy_main = server_socket
        .try_clone()
        .expect("Failed to clone server socket");

    // Spawn threads for bidirectional communication
    let handle1 = thread::spawn(move || {
        if let Err(e) = forward_data(client_socket, server_socket) {
            log(format!("Relay error: {}", e));
        }
    });

    let handle2 = thread::spawn(move || {
        if let Err(e) = forward_data(server_socker_copy, client_socket_copy) {
            log(format!("Relay error: {}", e));
        }
    });

    // Wait for both threads to complete
    if let Err(e) = handle1.join() {
        eprintln!("Thread 1 panicked: {:?}", e);
    }
    if let Err(e) = handle2.join() {
        eprintln!("Thread 2 panicked: {:?}", e);
    }

    // Shutdown sockets
    if let Err(e) = client_socket_copy_main.shutdown(Shutdown::Both) {
        eprintln!("Failed to shutdown client socket: {}", e);
    }
    if let Err(e) = server_socker_copy_main.shutdown(Shutdown::Both) {
        eprintln!("Failed to shutdown server socket: {}", e);
    }
}
