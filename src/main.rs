use clap::Parser;
use sqlx::postgres::PgPoolOptions;

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
}
