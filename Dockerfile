FROM rust:latest

COPY . .
RUN cargo build --release

CMD ["./target/release/seneca-solver", "--xp-farm", "--duration-farm"]