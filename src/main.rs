mod reader;

use reader::handle_client_stream;
use std::net::TcpListener;

pub fn log(log: String) {
    println!("{}", log);
}

fn main() -> std::io::Result<()> {
    let listener = match TcpListener::bind("127.0.0.1:1080") {
        Ok(listener) => listener,
        Err(error) => {
            log(format!("Got error creating listener: {}", error));
            panic!("Exiting, could not create server.");
        }
    };

    // accept connections and process them serially
    for stream in listener.incoming() {
        let client_stream = match stream {
            Ok(connection) => connection,
            Err(error) => {
                log(format!("Got error while creating connection: {}", error));
                continue;
            }
        };
        log(format!(
            "New connection: {}",
            client_stream.peer_addr().unwrap()
        ));
        // Single threaded
        handle_client_stream(client_stream);
    }
    Ok(())
}
