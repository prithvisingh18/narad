use super::logger::log;
use tokio::net::TcpStream;

// Function to relay data between client and server sockets
// Using tokio's copy_bidirectional for efficient async bidirectional relay
pub async fn relay_data(mut client_socket: TcpStream, mut server_socket: TcpStream) {
    match tokio::io::copy_bidirectional(&mut client_socket, &mut server_socket).await {
        Ok((client_to_server, server_to_client)) => {
            log(format!(
                "Relay finished: {} bytes client->server, {} bytes server->client",
                client_to_server, server_to_client
            ));
        }
        Err(e) => {
            log(format!("Relay error: {}", e));
        }
    }
}
