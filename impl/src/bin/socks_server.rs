use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

use narad::logger::log;
use narad::socks_handler::handle_client_stream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    let listener = match TcpListener::bind("0.0.0.0:9999").await {
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

    loop {
        let (client_stream, addr) = match listener.accept().await {
            Ok((stream, addr)) => (stream, addr),
            Err(error) => {
                log(format!("Got error while creating connection: {}", error));
                continue;
            }
        };
        log(format!("New connection: {}", addr));

        let thread_auth_reqired = Arc::clone(&auth_reqired);
        let thread_username = Arc::clone(&username);
        let thread_password = Arc::clone(&password);

        tokio::spawn(async move {
            handle_client_stream(client_stream, thread_auth_reqired, thread_username, thread_password).await;
        });
    }
}
