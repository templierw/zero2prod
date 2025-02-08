use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("failed to read configuration file");

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);

    let lst = TcpListener::bind(address)?;
    run(lst, connection_pool)?.await
}
