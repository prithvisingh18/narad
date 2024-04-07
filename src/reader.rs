use super::log;

use std::io::{Read, Write};
use std::net::TcpStream;

pub fn handle_client_stream(mut client_stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut message_count = 0;
    loop {
        // Read data from the stream
        match client_stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    log(format!("Connection ended got {} messages.", message_count));
                    break;
                }
                log(format!(
                    "Received request: {}, message number -> {}",
                    String::from_utf8_lossy(&buffer),
                    message_count
                ));

                // Simple response
                let response = "Recived.\n";
                match client_stream.write(response.as_bytes()) {
                    Ok(_) => {}
                    Err(error) => {
                        log(format!("Got error while responding to message: {}", error));
                        continue;
                    }
                };
                match client_stream.flush() {
                    Ok(_) => {}
                    Err(error) => {
                        log(format!(
                            "Got error while flushing the write stream: {}",
                            error
                        ));
                        continue;
                    }
                };
                message_count += 1;
            }
            Err(e) => log(format!("Failed to read from connection: {}", e)),
        }
    }
}
