use std::env;
use std::io::{Error, Read, Result, Write};
use std::net::{IpAddr, TcpStream};
use std::thread;

use narad::logger::log;
use narad::relay::relay_data;

// Function to handle tmp_connect in a separate thread
fn handle_tmp_connection(mut tmp_connection: TcpStream) -> Result<()> {
    // Read target ip
    let mut buffer = [0; 1024];
    let bytes_read = tmp_connection.read(&mut buffer)?;
    let target_ip_string = String::from_utf8_lossy(&buffer[..bytes_read]);
    let target_ip = match target_ip_string.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            println!("Failed to parse IP address: {}", e);
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "target ip parsing failed",
            ));
        }
    };
    tmp_connection.write_all(b"ok")?;

    // Read target port
    let bytes_read = tmp_connection.read(&mut buffer)?;
    let target_port_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let target_port = match target_port_str.trim().parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            println!("Failed to parse port: {}", e);
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "target port parsing failed",
            ));
        }
    };

    // Create another connection to the target ip and target port
    println!(
        "Creating port connection failed -> {}",
        format!("{}:{}", target_ip, target_port)
    );
    let remote_connection = TcpStream::connect((target_ip, target_port))?;

    // Respond 'ok' to both messages
    tmp_connection.write_all(b"ok")?;

    // Relay data between node and controller using `relay_data` function
    relay_data(tmp_connection, remote_connection);

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let controller_ip = &args[1];
    println!("Controller IP: {}", controller_ip);

    // Create the main connection to the controller
    let mut main_connection = TcpStream::connect(controller_ip)?;
    main_connection.write_all(b"node_greeting")?;
    println!("Sent 'node_greeting' to controller");
    // Send `node_greeting` to controller through the main connection
    log("Sending node_greeting to controller".to_owned());
    main_connection.write_all(b"node_greeting")?;

    loop {
        // Wait for `node_tmp_connect` from the controller
        let mut buffer = [0; 1024];
        let bytes_read = main_connection.read(&mut buffer)?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);

        if message == "node_tmp_connect" {
            // Create another connection to the controller
            let tmp_connection = TcpStream::connect(controller_ip)?;
            log(
                "Received node_tmp_connect, creating new thread for handling this connection"
                    .to_owned(),
            );

            // Pass the tmp connection to a new thread
            thread::spawn(move || handle_tmp_connection(tmp_connection));
        }
    }

    // Ok(())
}
