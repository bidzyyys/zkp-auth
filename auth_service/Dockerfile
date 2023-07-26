# Builder
FROM rust:1.69.0 as builder

RUN apt-get update && \
   apt-get install -y  protobuf-compiler && \
   rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/nillion

COPY .. .

RUN cargo build --release --package auth_service

# Runner
FROM debian:12.0-slim

RUN apt-get update && \
   rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/nillion/target/release/auth_service /usr/local/bin/auth_service

CMD ["auth_service"]
