# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.66.1-slim-buster as cargo-build
WORKDIR /usr/src/myapp
COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release 
RUN rm -f target/release/deps/myapp*
COPY . .
RUN cargo build --release 

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:buster-slim
WORKDIR /apt/app/
COPY --from=cargo-build /usr/src/myapp/target/release/myapp .
CMD ["./myapp"]