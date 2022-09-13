extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use sqlx::PgPool;

use std::error::Error;

use std::env;

mod db;
mod endpoints;
mod model;

// Gonna need to handle TLS certs here when I deploy - lets look at NGINX
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenv::dotenv()?;
    pretty_env_logger::init();

    let conn_string = match env::var("DATABASE_URL") {
        Ok(db_url) => db_url,
        Err(_) => format!(
            "postgres://{}:{}@127.0.0.1:{}/{}",
            "postgres", "postgres", 5432, "postgres"
        ),
    };

    let pool = PgPool::connect(&conn_string).await;

    let o = match pool {
        Ok(p) => {
            info!("Connection established to Postgres DB");
            Some(p)
        }
        Err(e) => {
            panic!("Error connecting to db: {}", e);
        }
    };

    info!("Starting submission server");
    warp::serve(endpoints::end(o))
        .run(([0, 0, 0, 0], 8000))
        .await;
    Ok(())
}
