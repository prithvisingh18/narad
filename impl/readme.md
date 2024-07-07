# Run

* Dependencies
    1. Cargo and rust should be installed.
    2. Install cross-rs and rustup for cross compilation. 

* Run in dev mode
    `cargo r --bin (controller|node|socks_server)`

* Building for release
    1. `cargo build --bin (controller|node|socks_server) --release` this will build for the target system
    2. `cross build  --bin (controller|node|socks_server) --release --target x86_64-unknown-linux-gnu`

