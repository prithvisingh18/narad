use narad::logger::log;
use narad::socks_handler::handle_client_stream;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    log("Starting TCP server.".to_owned());
    let listener = match TcpListener::bind("127.0.0.1:1080") {
        Ok(listener) => listener,
        Err(error) => {
            log(format!("Got error creating listener: {}", error));
            panic!("Exiting, could not create server.");
        }
    };
    
    log("Waiting to accept connections.".to_owned());
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
