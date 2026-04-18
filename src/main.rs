use std::io::ErrorKind;

use clap::Parser;
use luddy_hackathon_sp26::{
    config::Config,
    router::{self, AppState},
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Parser)]
struct Args {}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            std::env::var("DATABASE_URL")
                .expect("The database url environment variable does not exist")
                .as_str(),
        )
        .await
        .expect("There was an issue connecting to the database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("There was an issue running migrations");

    let config: Config;

    match tokio::fs::read_to_string("config.toml").await {
        Ok(raw_config) => {
            config = toml::from_str(raw_config.as_str())
                .expect("There was an error parsing the config file")
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            config = Config {
                secret: Uuid::new_v4(),
            };
            config
                .save()
                .await
                .expect("There was an error saving the newly created config.toml file");
        }
        Err(e) => panic!("There was an error loading the config.toml file: {}", e),
    };

    println!("Admin Secret: {}", config.secret);

    let state = AppState { db: pool, config };

    let app = router::app(state);

    // TOOD: Grab from env or toml?
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Couldn't create TCP listener");

    println!("Listening on localhost:3000");
    axum::serve(listener, app)
        .await
        .expect("There was an error serving the app")
}
