FROM rust:1.82.0

WORKDIR /app

RUN apt-get update && apt-get install -y protobuf-compiler libseccomp-dev systemd

COPY . .

RUN cargo clean

RUN cargo build --release 

CMD ["./target/debug/core" ]

