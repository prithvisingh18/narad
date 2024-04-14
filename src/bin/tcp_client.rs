use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    // Prompt the user for the server address and port
    // print!("Enter the server address and port (e.g., 127.0.0.1:8080): ");
    // io::stdout().flush()?;
    // let mut server_address = String::new();
    // io::stdin().read_line(&mut server_address)?;
    // server_address = server_address.trim().to_string();
    // let server_address = "https://duckduckgo.com/";
    // let server_address = "localhost:1080";
    let server_address = "20.204.244.192:80";

    // Establish a TCP connection to the server
    let mut stream = TcpStream::connect(server_address)?;
    println!("Connected to the server");

    // let message = "GET / HTTP/1.1
    //     accept: */*
    //     host: 20.204.244.192:80

    //     ";
    let message = "GET / HTTP/1.1\r\naccept: */*\r\nhost: 20.204.244.192:80\r\n\r\n";

    // Send the message to the server
    stream.write_all(message.as_bytes())?;
    // stream.write_all(b"\n")?;
    stream.flush()?;

    // Create a buffered reader for reading server responses
    let mut reader = BufReader::new(&stream);
    // loop {
    //     // Read the server's response
    //     let mut response = String::new();
    //     reader.read_line(&mut response)?;
    //     println!(
    //         "Server response: {} len {}",
    //         response.trim(),
    //         response.trim().len()
    //     );
    // }

    loop {
        // Read the server's response
        let mut response = String::new();
        let bytes_read = reader.read_line(&mut response)?;

        if bytes_read == 0 {
            // End of the stream reached
            println!("End of the stream reached. Closing the connection.");
            break;
        }

        println!(
            "Server response: {} len {}",
            response.trim(),
            response.trim().len()
        );
    }

    // loop {
    //     // Prompt the user for input
    //     // print!("Enter a message to send (or 'quit' to exit): ");
    //     // io::stdout().flush()?;

    //     // let mut message = String::new();
    //     // io::stdin().read_line(&mut message)?;
    //     // message = message.trim().to_string();

    //     // Check if the user wants to quit
    //     // if message.eq_ignore_ascii_case("quit") {
    //     //     break;
    //     // }

    //     let message = "GET / HTTP/1.1
    //     accept: */*
    //     host: 20.204.244.192:80

    //     ";

    //     // Send the message to the server
    //     stream.write_all(message.as_bytes())?;
    //     // stream.write_all(b"\n")?;
    //     stream.flush()?;

    //     // Create a buffered reader for reading server responses
    //     let mut reader = BufReader::new(&stream);
    //     // Read the server's response
    //     let mut response = String::new();
    //     reader.read_line(&mut response)?;
    //     println!("Server response: {}", response.trim());
    // }

    println!("Disconnected from the server");
    Ok(())
}
