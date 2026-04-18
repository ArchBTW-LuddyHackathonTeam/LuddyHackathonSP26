use clap::Parser;
use luddy_hackathon_sp26::router::{self, AppState};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    /// setup flag lol (TODO: proper english)
    #[arg(short, long)]
    setup: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(std::env::var("DATABASE_URL").unwrap().as_str())
        .await
        .expect("There was an issue connecting to the database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("There was an issue running migrations");

    if args.setup {
        todo!("requires toml config")
    }

    let secret = Uuid::new_v4();
    println!("Admin Secret: {}", secret);

    let state = AppState { db: pool, secret };

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
