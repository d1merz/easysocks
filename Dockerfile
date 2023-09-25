FROM messense/rust-musl-cross:x86_64-musl as builder
WORKDIR /easysocks
COPY src/ src/
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /easysocks/target/x86_64-unknown-linux-musl/release/easysocks /easysocks
ENTRYPOINT ["/easysocks"]
