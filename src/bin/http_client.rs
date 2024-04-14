use reqwest::blocking::Client;
// use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // print!("Enter a URL: ");
    // io::stdout().flush()?;

    // let mut url = String::new();
    // io::stdin().read_line(&mut url)?;
    // url = url.trim().to_string();
    let url = "http://localhost:1080";
    // let url = "https://duckduckgo.com/";
    // let url = "http://20.204.244.192:80";
    let response = client.get(url).send()?;
    let body = response.text()?;

    println!("Response Body:");
    println!("{}", body);

    Ok(())
}
