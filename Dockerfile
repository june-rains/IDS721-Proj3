FROM rust:1.63.0
WORKDIR /usr/src/app
COPY . .
RUN cargo install diesel_cli --no-default-features --features sqlite
RUN cargo build --release
CMD ["./target/release/rocket"]