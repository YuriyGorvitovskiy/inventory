FROM rust:1.90-bookworm AS builder
WORKDIR /app

COPY Cargo.toml ./Cargo.toml
COPY services/inventory-core/Cargo.toml ./services/inventory-core/Cargo.toml
COPY services/inventory-core/src ./services/inventory-core/src

RUN cargo build --release -p inventory-core

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 app
WORKDIR /home/app

COPY --from=builder /app/target/release/inventory-core /usr/local/bin/inventory-core
COPY models/inventory-core /opt/inventory/models/inventory-core

USER app
EXPOSE 8080
ENV PORT=8080
ENV MODEL_DIR=/opt/inventory/models/inventory-core
CMD ["inventory-core"]
