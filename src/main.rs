use std::{io::ErrorKind, sync::Arc};

use clap::Parser;
use luddy_hackathon_sp26::{
    config::{Config, LeaderboardConfig, LeaderboardSortOrder, ServerConfig},
    models::token::Token,
    router::{self, AppState},
};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;

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
                    sort_order: LeaderboardSortOrder::Descending,
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
        return;
    }

    let server_port = config.server.port;
    let state = AppState {
        db: pool,
        config: Arc::new(RwLock::new(config)),
    };

    let app = router::app(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", server_port))
        .await
        .expect("Couldn't create TCP listener");

    println!("Listening on localhost:{}", server_port);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("There was an error serving the app")
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to setup signal handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to setup signal handler");

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    }
}
