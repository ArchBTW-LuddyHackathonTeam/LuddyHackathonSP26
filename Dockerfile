FROM rust:slim
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy deps first (cached unless change)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source and build (only rebuilds on src code changes)
COPY . .
RUN cargo build --release
CMD ["cargo", "run", "--release"]
