FROM rust:alpine

RUN apk update && apk add lld clang libressl-dev musl-dev

WORKDIR /newsletter

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release

ENTRYPOINT [ "./target/release/newsletter" ]