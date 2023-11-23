# onote-rust-server
The backend Rust server for [ONote](https://github.com/rrecalo/ONote). Interfaces with ONote's MongoDB cluster to serve the client with quick, *rusty* calls.

Built to solidify my understanding of Rust, and to replace the horribly slow original NodeJS backend. Under very limited testing/benchmarking, this version of the server was up to ***5-6x faster*** than the *NodeJS version* - blazingly fast!

Hosted on an AWS EC2 instance with a reverse-proxy through NGINX

- Written in Rust with the help of *rust-analyzer*.
- Built in the [Axum](https://docs.rs/axum/latest/axum/) framework using popular crates including:
  - [tokio](https://tokio.rs/) (asynchronous runtime)
  - [serde](https://serde.rs/) (for serialization)
  - [tower](https://github.com/tower-rs/tower) (for cors)
- Lots of unwrap(), I'll fix it later™

