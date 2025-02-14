use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration file");

    let connection_pool = PgPool::connect_lazy(&config.database.connection_string())
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);

    let lst = TcpListener::bind(address)?;
    run(lst, connection_pool)?.await
}
