use std::io::ErrorKind;

use clap::Parser;
use luddy_hackathon_sp26::{
    config::{Config, LeaderboardConfig, ServerConfig},
    models::token::Token,
    router::{self, AppState},
};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
struct Args {
    /// Reset the administrator token
    #[arg(long)]
    reset_password: bool,
}

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
                server: ServerConfig { port: 3000 },
                leaderboard: LeaderboardConfig {
                    title: String::from("Example"),
                    sort_order: String::from("descending"),
                },
            };
            config
                .save()
                .await
                .expect("There was an error saving the newly created config.toml file");
        }
        Err(e) => panic!("There was an error loading the config.toml file: {}", e),
    };

    if !Token::any_exists(&pool)
        .await
        .expect("There was an issue fetching tokens from the database")
    {
        println!(
            "Admin Secret (new): {}",
            Token::new(&pool)
                .await
                .expect("There was an issue registering a new token")
        );
    } else if args.reset_password {
        Token::clear(&pool)
            .await
            .expect("There was an clearing old tokens");
        println!(
            "Admin Secret (new): {}",
            Token::new(&pool)
                .await
                .expect("There was an issue registering a new token")
        );
    }

    let state = AppState { db: pool, config };
    let server_config = state.config.server;

    let app = router::app(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", server_config.port))
        .await
        .expect("Couldn't create TCP listener");

    println!("Listening on localhost:3000");
    axum::serve(listener, app)
        .await
        .expect("There was an error serving the app")
}
