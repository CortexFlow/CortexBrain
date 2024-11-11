FROM rust:1.82.0

WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler

COPY . .

RUN cargo build --release

