FROM rustlang/rust:nightly

WORKDIR /usr/src/rustapp

COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release 
RUN rm -rf src/main.rs
COPY src/ ./src/

CMD cargo run --release