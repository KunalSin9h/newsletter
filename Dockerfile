# Build Stage
# Cross-Compile to work with Alpine
# https://levelup.gitconnected.com/create-an-optimized-rust-alpine-docker-image-1940db638a6c
FROM rust:alpine AS builder

RUN apk update && apk add lld clang libressl-dev musl-dev
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /newsletter

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --target x86_64-unknown-linux-musl --release

# Runtime Stage

FROM alpine:latest

WORKDIR /newsletter

COPY --from=builder /newsletter/target/release/newsletter newsletter
COPY configuration configuration

ENV APP_ENVIRONMENT production

EXPOSE 5000

ENTRYPOINT ["./newsletter"]