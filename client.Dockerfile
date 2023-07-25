# Builder
FROM rust:1.69.0 as builder

RUN apt-get update && \
   apt-get install -y  protobuf-compiler && \
   rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/nillion

COPY . .

RUN cargo build --release --package client

# Runner
FROM debian:12.0-slim

RUN apt-get update && \
   rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/nillion/target/release/client /usr/local/bin/client

CMD ["client"]
