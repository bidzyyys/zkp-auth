# Builder
FROM rust:1.69.0 as builder

RUN apt-get update && \
   apt-get install -y  protobuf-compiler && \
   rm -rf /var/lib/apt/lists/* && \
   rustup default nightly

WORKDIR /usr/src/nillion

COPY . .

RUN cargo build --release --package router

# Runner
FROM debian:12.0-slim

RUN apt-get update && \
   rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/nillion/target/release/router /usr/local/bin/router

CMD ["router"]
