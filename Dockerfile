# Build Stage
FROM lukemathwalker/cargo-chef:latest-rust-slim-bullseye AS chef

RUN apt-get update && apt-get install libssl-dev pkg-config lld clang  -y

WORKDIR /newsletter

# Create a lock file for installing dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Building stage
FROM chef AS builder 
COPY --from=planner /newsletter/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json


COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin newsletter

# Runtime Stage
FROM debian:bullseye-slim AS runtime

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /newsletter

COPY --from=builder /newsletter/target/release/newsletter newsletter
COPY configuration configuration

ENV APP_ENVIRONMENT production

EXPOSE 5000

ENTRYPOINT ["./newsletter"]