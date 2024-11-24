use tokio;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::sync::Arc;
use std::{env, thread};

use narad::logger::log;
use narad::async_socks_handler::handle_client_stream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    log("Starting socks server.".to_owned());
    let args: Vec<String> = env::args().collect();

    let mut auth_required = false;
    let mut username = "".to_owned();
    let mut password = "".to_owned();
    if args.len() > 1 {
        let auth = args[1].clone();
        log(format!("Auth arg -> {}", auth));
        let creds: Vec<&str> = auth.split(":").collect();
        username = String::from(creds[0]);
        password = String::from(creds[1]);
        auth_required = true;
    } else {
        log("No creds given, no auth will not be required.".to_owned());
    }

    let listener = match TcpListener::bind("0.0.0.0:9000").await {
        Ok(listener) => listener,
        Err(error) => {
            log(format!("Got error creating listener: {}", error));
            panic!("Exiting, could not create server.");
        }
    };

    log("Waiting to accept connections.".to_owned());
    let auth_required = Arc::new(auth_required);
    let username = Arc::new(username);
    let password = Arc::new(password);

    loop {
        // Accept incoming connections
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {:?}", addr);

        // Clone shared state for each connection
        let thread_auth_required = Arc::clone(&auth_required);
        let thread_username = Arc::clone(&username);
        let thread_password = Arc::clone(&password);

        // Spawn a new task for each connection
        tokio::spawn(async move {
            handle_client_stream(socket, thread_auth_required, thread_username, thread_password).await;
        });
    }

    // for stream in listener.accept().await {
    //     let client_stream = match stream {
    //         Ok(connection) => connection,
    //         Err(error) => {
    //             log(format!("Got error while creating connection: {}", error));
    //             continue;
    //         }
    //     };
    //     log(format!(
    //         "New connection: {}",
    //         client_stream.peer_addr().unwrap()
    //     ));
    //     let thread_auth_reqired = Arc::clone(&auth_reqired);
    //     let thread_username = Arc::clone(&username);
    //     let thread_password = Arc::clone(&password);
    //     // Single threaded
    //     thread::spawn(move || {
    //         handle_client_stream(client_stream, thread_auth_reqired, thread_username, thread_password);
    //     });
    // }
    Ok(())
}
