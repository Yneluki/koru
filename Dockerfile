FROM rust:1.69.0 as chef
WORKDIR /app
RUN cargo install cargo-chef
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# Compute a lock-like file for project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
ARG KORU_FEATURES=production
# Build project dependencies
RUN cargo chef cook --release --features $KORU_FEATURES --no-default-features --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
# Build project
RUN cargo build --release --bin koru --features $KORU_FEATURES --no-default-features

FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Add TLS support, required to make https requests.
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/koru koru
COPY config config
ENV KORU_ENV prod
ENTRYPOINT ["./koru"]