use std::net::TcpListener;
use std::thread;

use narad::logger::log;
use narad::socks_handler::handle_client_stream;

fn main() -> std::io::Result<()> {
    log("Starting socks server.".to_owned());
    let listener = match TcpListener::bind("127.0.0.1:9999") {
        Ok(listener) => listener,
        Err(error) => {
            log(format!("Got error creating listener: {}", error));
            panic!("Exiting, could not create server.");
        }
    };

    log("Waiting to accept connections.".to_owned());
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
        thread::spawn(|| {
            handle_client_stream(client_stream);
        });
    }
    Ok(())
}
