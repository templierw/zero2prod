use std::net::TcpListener;

use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let lst = TcpListener::bind("127.0.0.1:8000").expect("failed to bind");
    run(lst)?.await
}
