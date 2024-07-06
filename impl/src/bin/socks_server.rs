use std::net::TcpListener;
use std::sync::Arc;
use std::{env, thread};

use narad::logger::log;
use narad::socks_handler::handle_client_stream;

fn main() -> std::io::Result<()> {
    log("Starting socks server.".to_owned());
    let args: Vec<String> = env::args().collect();

    let mut auth_reqired = false;
    let mut username = "".to_owned();
    let mut password = "".to_owned();
    if args.len() > 1 {
        let auth = args[1].clone();
        log(format!("Auth arg -> {}", auth));
        let creds: Vec<&str> = auth.split(":").collect();
        username = String::from(creds[0]);
        password = String::from(creds[1]);
        auth_reqired = true;
    } else {
        log("No creds given, no auth will not be required.".to_owned());
    }

    let listener = match TcpListener::bind("0.0.0.0:9000") {
        Ok(listener) => listener,
        Err(error) => {
            log(format!("Got error creating listener: {}", error));
            panic!("Exiting, could not create server.");
        }
    };

    log("Waiting to accept connections.".to_owned());
    let auth_reqired = Arc::new(auth_reqired);
    let username = Arc::new(username);
    let password = Arc::new(password);

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
        let thread_auth_reqired = Arc::clone(&auth_reqired);
        let thread_username = Arc::clone(&username);
        let thread_password = Arc::clone(&password);
        // Single threaded
        thread::spawn(move || {
            handle_client_stream(client_stream, thread_auth_reqired, thread_username, thread_password);
        });
    }
    Ok(())
}
