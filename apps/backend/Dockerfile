FROM rust:1.96.0
WORKDIR /app
COPY . .
RUN cargo build --release
ENV HOST=0.0.0.0
EXPOSE 8080
CMD ["./target/release/main"]
