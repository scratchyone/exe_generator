FROM rustlang/rust:nightly

WORKDIR /usr/src/rustapp

RUN apt update && apt install -y mingw-w64
RUN rustup target add x86_64-pc-windows-gnu

COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release 
RUN rm -rf src/main.rs
COPY src/ ./src/
RUN chmod -R 777 .

CMD cargo run --release