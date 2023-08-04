# Build Stage
FROM rust:alpine AS builder

RUN apk update && apk add lld clang libressl-dev musl-dev
WORKDIR /newsletter
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# Runtime Stage

FROM alpine:latest
WORKDIR /newsletter

COPY --from=builder /newsletter/target/release/newsletter newsletter
COPY configuration
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./newsletter"]