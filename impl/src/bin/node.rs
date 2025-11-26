use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use narad::logger::log;
use narad::relay::relay_data;

// Function to handle tmp_connect in a separate task
async fn handle_tmp_connection(mut tmp_connection: TcpStream) -> std::io::Result<()> {
    // Read target ip
    let mut buffer = [0; 1024];
    let bytes_read = tmp_connection.read(&mut buffer).await?;
    let target_ip_string = String::from_utf8_lossy(&buffer[..bytes_read]);
    let target_ip = match target_ip_string.parse::<std::net::IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            println!("Failed to parse IP address: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "target ip parsing failed",
            ));
        }
    };
    tmp_connection.write_all(b"ok").await?;

    // Read target port
    let bytes_read = tmp_connection.read(&mut buffer).await?;
    let target_port_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let target_port = match target_port_str.trim().parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            println!("Failed to parse port: {}", e);
            return Err(std::io::Error::new(
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
    let remote_connection = TcpStream::connect((target_ip, target_port)).await?;

    // Respond 'ok' to both messages
    tmp_connection.write_all(b"ok").await?;

    // Relay data between node and controller using `relay_data` function
    relay_data(tmp_connection, remote_connection).await;

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let controller_ip = &args[1];
    println!("Controller IP: {}", controller_ip);

    // Create the main connection to the controller
    let mut main_connection = TcpStream::connect(controller_ip).await?;
    main_connection.write_all(b"node_greeting").await?;
    println!("Sent 'node_greeting' to controller");
    // Send `node_greeting` to controller through the main connection
    log("Sending node_greeting to controller".to_owned());
    main_connection.write_all(b"node_greeting").await?;

    loop {
        // Wait for `node_tmp_connect` from the controller
        let mut buffer = [0; 1024];
        let bytes_read = main_connection.read(&mut buffer).await?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);

        if message == "node_tmp_connect" {
            // Create another connection to the controller
            let tmp_connection = TcpStream::connect(controller_ip).await?;
            log(
                "Received node_tmp_connect, creating new task for handling this connection"
                    .to_owned(),
            );

            // Pass the tmp connection to a new task
            tokio::spawn(async move {
                if let Err(e) = handle_tmp_connection(tmp_connection).await {
                    log(format!("Error handling tmp connection: {}", e));
                }
            });
        }
    }
}
