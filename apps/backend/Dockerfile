FROM debian:bookworm-slim
WORKDIR /app
COPY target/release/main .
ENV HOST=0.0.0.0
EXPOSE 8080
CMD ["./target/release/main"]
